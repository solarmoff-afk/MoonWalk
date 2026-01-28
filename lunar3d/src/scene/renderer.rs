// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

#![allow(unused)]

use std::collections::HashMap;
use glam::{Mat3, Mat4, Quat, EulerRot};
use bytemuck::{bytes_of, cast_slice};
use moonwalk::MoonWalk;
use moonwalk::rendering::custom::{CustomPaint, MoonRenderPass, MoonBuffer};
use moonwalk::BindResource;

use crate::factory::LunarFactory;
use crate::internal::store::ObjectStore;
use crate::resources::InstanceRaw;
use crate::core::types::GlobalUniform;
use crate::core::config::MAX_SHADOWS;

pub struct SceneRenderer {
    pub(crate) paint: CustomPaint,
    pub(crate) global_buf: MoonBuffer,
    global_bg: Option<moonwalk::rendering::custom::MoonBindGroup>,
    current_shadow_map_id: Option<u32>,
    material_bg_cache: HashMap<(u32, u32, u32), moonwalk::rendering::custom::MoonBindGroup>,
    instance_data_cache: Vec<InstanceRaw>,
    snapshot_id: u32,
    width: u32,
    height: u32,
}

struct BatchState {
    pipe_key: String,
    mesh_id: usize,
    mat_key: (u32, u32, u32),
    start_instance: u32,
    count: u32,
}

impl SceneRenderer {
    pub fn new(
        mw: &mut MoonWalk, 
        width: u32, 
        height: u32
    ) -> Self {
        let mut paint = mw.new_custom_paint(width, height, "LunarScene");
        
        paint.start_frame(&mw.renderer.context);
        let _ = paint.render_pass(MoonRenderPass::new().set_clear_color(Some(glam::Vec4::ZERO)));
        paint.submit_frame(&mw.renderer.context);
        
        let snapshot_id = paint.snapshot(mw);

        Self {
            paint,
            global_buf: mw.create_uniform_buffer(&[0; 4096]),
            global_bg: None,
            current_shadow_map_id: None,
            material_bg_cache: HashMap::new(),
            instance_data_cache: Vec::with_capacity(1000),
            snapshot_id,
            width, 
            height,
        }
    }

    pub fn update_instances(
        &mut self, 
        mw: &MoonWalk, 
        store: &mut ObjectStore
    ) -> usize {
        self.instance_data_cache.clear();
        
        for i in 0..store.positions.len() {
            if !store.alive[i] { 
                continue; 
            }
            
            let rotation = Quat::from_euler(
                EulerRot::XYZ, 
                store.rotations[i].x, 
                store.rotations[i].y, 
                store.rotations[i].z
            );

            let t = Mat4::from_scale_rotation_translation(
                store.scales[i], 
                rotation, 
                store.positions[i]
            );
            
            let nm = Mat3::from_mat4(t);
            let c0 = nm.col(0); 
            let c1 = nm.col(1); 
            let c2 = nm.col(2);

            self.instance_data_cache.push(InstanceRaw {
                model: t.to_cols_array_2d(),
                normal_mat_0: [c0.x, c0.y, c0.z, 0.0],
                normal_mat_1: [c1.x, c1.y, c1.z, 0.0],
                normal_mat_2: [c2.x, c2.y, c2.z, 0.0],
                color: store.colors[i].to_array(),
                metallic: store.metallic[i],
                roughness: store.roughness[i],
                unlit: if store.unlit[i] { 1.0 } else { 0.0 },
            });
        }
        
        if self.instance_data_cache.len() > store.capacity {
            store.capacity = (self.instance_data_cache.len() as f32 * 1.5) as usize;
            let size = store.capacity * std::mem::size_of::<InstanceRaw>();
            store.buffer = mw.create_vertex_buffer(&vec![0u8; size]);
        }
        
        if !self.instance_data_cache.is_empty() {
            mw.update_buffer(&store.buffer, cast_slice(&self.instance_data_cache));
        }
        
        self.instance_data_cache.len()
    }

    pub fn render_main_pass(
        &mut self, 
        mw: &mut MoonWalk, 
        factory: &LunarFactory, 
        store: &ObjectStore,
        shadow_map_id: u32
    ) -> u32 {
        if self.global_bg.is_none() || self.current_shadow_map_id != Some(shadow_map_id) {
             let bg = mw.create_bind_group(&factory.global_layout, &[
                BindResource::Uniform(&self.global_buf),
                BindResource::Texture(shadow_map_id),
                BindResource::Sampler(shadow_map_id)
            ]).unwrap();
            
             self.global_bg = Some(bg);
            self.current_shadow_map_id = Some(shadow_map_id);
        }

        // Прогрев кэша материалов
        for i in 0..store.positions.len() {
            if !store.alive[i] {
                continue;
            }

            let alb = store.albedo_ids[i].unwrap_or(factory.default_white);
            let nrm = store.normal_ids[i].unwrap_or(factory.default_normal);
            let mr = factory.default_white;
            let key = (alb, nrm, mr);

            if !self.material_bg_cache.contains_key(&key) {
                let flags = crate::core::types::MaterialFlags {
                    use_albedo_map: 1, 
                    use_normal_map: if nrm != factory.default_normal {
                        1
                    } else {
                        0
                    },

                    use_mr_map: 0, 
                    _pad: 0
                };

                let buf = mw.create_uniform_buffer(bytes_of(&flags));
                let bg = mw.create_bind_group(&factory.material_layout, &[
                    BindResource::Texture(alb), 
                    BindResource::Sampler(alb),
                    BindResource::Texture(nrm), 
                    BindResource::Texture(mr),
                    BindResource::Uniform(&buf)
                ]).unwrap();

                self.material_bg_cache.insert(key, bg);
            }
        }

        let mut current_batch: Option<BatchState> = None;
        let instance_size = std::mem::size_of::<InstanceRaw>() as u64;
        
        let mut gpu_instance_index = 0;

        self.paint.start_frame(&mw.renderer.context);
        
        if let Some(mut pass) = self.paint.render_pass(MoonRenderPass::new()
            .set_clear_color(Some(glam::Vec4::ZERO))
            .set_clear_depth(true))
        {
            if let Some(bg) = &self.global_bg {
                pass.set_bind_group(0, bg);
            }

            macro_rules! draw_batch {
                ($batch:expr) => {
                    if let Some(pipe) = factory.pipelines.get(&$batch.pipe_key) {
                        pass.set_pipeline(pipe);
                        
                        if let Some(mesh) = factory.meshes.get($batch.mesh_id) {
                            pass.set_vertex_buffer(0, &mesh.vertex_buffer, 0, None);
                            pass.set_index_buffer(&mesh.index_buffer, 0, None);
                            
                            if let Some(bg) = self.material_bg_cache.get(&$batch.mat_key) {
                                pass.set_bind_group(1, bg);
                            }

                            let offset = ($batch.start_instance as u64) * instance_size;
                            let size = ($batch.count as u64) * instance_size;
                            
                            pass.set_vertex_buffer(1, &store.buffer, offset, Some(size));
                            pass.draw_indexed(0..mesh.index_count, 0, 0..$batch.count);
                        }
                    }
                };
            }

            for i in 0..store.positions.len() {
                if !store.alive[i] { 
                    continue; 
                }

                let pipe_key = &store.layer_names[i];
                let mesh_id = store.mesh_ids[i];
                let alb = store.albedo_ids[i].unwrap_or(factory.default_white);
                let nrm = store.normal_ids[i].unwrap_or(factory.default_normal);
                let mr = factory.default_white;
                let mat_key = (alb, nrm, mr);

                let mut matches = false;
                if let Some(batch) = &current_batch {
                    if &batch.pipe_key == pipe_key && batch.mesh_id == mesh_id && batch.mat_key == mat_key {
                        matches = true;
                    }
                }

                if matches {
                    if let Some(batch) = &mut current_batch {
                        batch.count += 1;
                    }
                } else {
                    if let Some(batch) = &current_batch {
                        draw_batch!(batch);
                    }

                    current_batch = Some(BatchState {
                        pipe_key: pipe_key.clone(),
                        mesh_id,
                        mat_key,
                        start_instance: gpu_instance_index,
                        count: 1,
                    });
                }
                
                gpu_instance_index += 1;
            }

            if let Some(batch) = &current_batch {
                draw_batch!(batch);
            }
        }
        
        self.paint.submit_frame(&mw.renderer.context);
        
        self.paint.update_snapshot(mw, self.snapshot_id);
        self.snapshot_id
    }

    pub fn resize(&mut self, mw: &mut MoonWalk, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        self.width = width;
        self.height = height;

        // Для того чтобы обновить размер поверхности нужно удалить старую
        // текстуру снапшота и создать новый custom paint, сделать снапшот
        // и перенастроить рендер сцены на новую текстуру

        self.paint = mw.new_custom_paint(width, height, "LunarScene");
        
        mw.remove_texture(self.snapshot_id);
        self.snapshot_id = self.paint.snapshot(mw);
    }
}
