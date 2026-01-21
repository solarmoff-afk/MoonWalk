// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

pub mod types;
pub mod loader;
pub mod pipeline;
pub mod batch;
pub mod render;

use std::collections::HashMap;
use glam::{Vec3, Vec4};
use bytemuck::{bytes_of, cast_slice};

use crate::MoonWalk;
use crate::rendering::custom::*;
use crate::public::custom::BindResource;
use crate::r#abstract::*;

pub use self::types::*;

pub struct Scene3D {
    pipeline_pbr: CustomPipeline,
    pipeline_phong: CustomPipeline,
    shadow_pipeline: CustomPipeline,
    custom_pipeline_override: Option<CustomPipeline>,
    global_layout: MoonBindGroupLayout,
    shadow_global_layout: MoonBindGroupLayout,
    material_layout: MoonBindGroupLayout,
    global_buf: MoonBuffer,
    shadow_buf: MoonBuffer,
    global_bg: Option<MoonBindGroup>,
    shadow_bg: Option<MoonBindGroup>,
    instance_buffer: MoonBuffer,
    instance_capacity: usize,

    pub meshes: Vec<MeshData>,
    pub inst_mesh_ids: Vec<usize>,
    pub inst_positions: Vec<Vec3>,
    pub inst_rotations: Vec<Vec3>,
    pub inst_scales: Vec<Vec3>,
    pub inst_alive: Vec<bool>,
    pub inst_free_slots: Vec<usize>,
    pub inst_parents: Vec<Option<usize>>,
    pub inst_unlit: Vec<bool>,
    pub inst_mat_base_color: Vec<Vec4>,
    pub inst_mat_metallic: Vec<f32>,
    pub inst_mat_roughness: Vec<f32>,
    pub inst_mat_albedo_id: Vec<Option<u32>>,
    pub inst_mat_normal_id: Vec<Option<u32>>,
    pub inst_mat_mr_id: Vec<Option<u32>>,
    pub lights_data: Vec<LightRaw>,
    pub lights_active: Vec<bool>,
    pub lights_free_slots: Vec<usize>,
    pub camera_pos: Vec3,
    pub camera_target: Vec3,
    pub ambient_color: Vec3,
    
    paint: CustomPaint,
    shadow_paint: CustomPaint,
    snapshot_id: u32,
    shadow_map_id: u32,
    
    pub shadow_map_size: u32,
    pub shadow_ortho_size: f32,
    pub lighting_model: LightingModel,
    pub shadow_quality: ShadowQuality,
    
    pub default_white: u32,
    pub default_normal: u32,
    material_bg_cache: HashMap<(u32, u32, u32), MoonBindGroup>,
    
    width: u32,
    height: u32,
}

impl Scene3D {
    /// Стартовый метод для создания 3д сцены. Он скрывает от пользователя
    /// работу с кастом пеинтом и позволяет создавать 3д на MoonWalk. Стоит
    /// осознавать что мунволк не является 3д движком, а 3д в нём дополнение
    /// к основному 2д функционалу
    pub fn new(mw: &mut MoonWalk, width: u32, height: u32) -> Self {
        let (global_layout, shadow_global_layout, material_layout) = pipeline::create_layouts(mw);
        let (pipeline_pbr, pipeline_phong, shadow_pipeline) = pipeline::create_pipelines(
            mw, &global_layout, &shadow_global_layout, &material_layout
        );
        
        let mut paint = mw.new_custom_paint(width, height, "Scene3D Main");
        
        paint.start_frame(&mw.renderer.context);
        
        let _ = paint.render_pass(MoonRenderPass::new().set_clear_color(Some(Vec4::ZERO)));
        paint.submit_frame(&mw.renderer.context);
        
        let snapshot_id = paint.snapshot(mw);
        
        // Стандартная карта теней 2048 пикселей в ширину и в высоту столкьо же
        let shadow_map_size = 2048;

        let mut shadow_paint = mw.new_custom_paint(shadow_map_size, shadow_map_size, "Shadow Map");
        let dummy = crate::rendering::texture::Texture::create_depth_texture(&mw.renderer.context, 1, 1, "Dummy");
        let real_depth = std::mem::replace(&mut shadow_paint.depth, dummy);
        let shadow_map_id = mw.renderer.state.add_texture(real_depth);

        let norm_px = vec![128, 128, 255, 255]; 
        let default_normal = mw.renderer.state.add_texture(crate::rendering::texture::Texture::from_raw(
            &mw.renderer.context, &norm_px, 1, 1, "DefNorm").unwrap());
        
        let white_px = vec![255, 255, 255, 255];

        let default_white = mw.renderer.state.add_texture(crate::rendering::texture::Texture::from_raw(
            &mw.renderer.context, &white_px, 1, 1, "DefWhite").unwrap());

        let global_buf = mw.create_uniform_buffer(&[0; 512]); 
        let shadow_buf = mw.create_uniform_buffer(&[0; 64]);

        let instance_capacity = 100;
        let instance_buffer = mw.create_vertex_buffer(&vec![0u8; instance_capacity * std::mem::size_of::<InstanceRaw>()]);

        Self {
            pipeline_pbr,
            pipeline_phong,
            shadow_pipeline,
            custom_pipeline_override: None,
            global_layout,
            shadow_global_layout,
            material_layout,
            global_buf,
            shadow_buf,
            global_bg: None,
            shadow_bg: None,
            instance_buffer,
            instance_capacity,
            meshes: Vec::new(),
            inst_mesh_ids: Vec::new(),
            inst_positions: Vec::new(),
            inst_rotations: Vec::new(),
            inst_scales: Vec::new(),
            inst_alive: Vec::new(),
            inst_free_slots: Vec::new(),
            inst_parents: Vec::new(),
            inst_unlit: Vec::new(),
            inst_mat_base_color: Vec::new(),
            inst_mat_metallic: Vec::new(),
            inst_mat_roughness: Vec::new(),
            inst_mat_albedo_id: Vec::new(),
            inst_mat_normal_id: Vec::new(),
            inst_mat_mr_id: Vec::new(),
            lights_data: Vec::new(),
            lights_active: Vec::new(),
            lights_free_slots: Vec::new(),
            
            // Позиция камеры
            camera_pos: Vec3::new(0.0, 2.0, 5.0),
            
            camera_target: Vec3::ZERO,
            ambient_color: Vec3::splat(0.03),
            paint,
            shadow_paint,
            snapshot_id,
            shadow_map_id,
            width,
            height,
            default_normal,
            default_white,
            material_bg_cache: HashMap::new(),
            shadow_map_size, shadow_ortho_size: 20.0,
            lighting_model: LightingModel::Pbr,
            shadow_quality: ShadowQuality::High,
        }
    }
    

    /// Этот метод устанавливает цвет света который будет по всей сцене независимо
    /// от наличия источников света на ней и его удаления от объекта. Принимает
    /// цвет в формате vec3 (rgb) из крейта glam
    pub fn set_ambient_light(&mut self, color: Vec3) {
        self.ambient_color = color;
    }
    
    /// Этот метод позволяет объекту игнорировать источники света
    pub fn set_unlit(&mut self, id: InstanceId, unlit: bool) {
        if id.0 < self.inst_unlit.len() {
            self.inst_unlit[id.0] = unlit;
        }
    }
    
    /// Этот метод делает второй объект родителем первого, что позволяет
    /// двум объектам двигаться вместе создавая иерархию объектов
    pub fn set_parent(&mut self, child: InstanceId, parent: InstanceId) {
        if child.0 < self.inst_parents.len() && child.0 != parent.0 {
            self.inst_parents[child.0] = Some(parent.0);
        }
    }
    
    /// Этот метод убирает родителя у ребёнка, но сам родитель остаётся,
    /// просто убирается их связь
    pub fn clear_parent(&mut self, child: InstanceId) {
        if child.0 < self.inst_parents.len() {
            self.inst_parents[child.0] = None;
        }
    }
    
    /// Этот метод позволяет установить свой собственный пайплайн, для этого
    /// нужно создать экземпляр MoonPipeline, передать MoonWalk первым аргументом,
    /// а MoonPipeline втором. Главное не забыть настроить пайплайн. Это позволяет
    /// использовать собственные шейдеры в 3d рендеринге
    pub fn set_custom_pipeline(&mut self, mw: &MoonWalk, desc: MoonPipeline) -> Result<(), crate::MoonWalkError> {
        let pipe = mw.compile_pipeline(desc, &[&self.global_layout, &self.material_layout])?;
        self.custom_pipeline_override = Some(pipe);
        self.lighting_model = LightingModel::Custom;
        Ok(())
    }
    
    /// Этот метод устанавливает качество теней либо полностью их отключает. Нужно
    /// передать экземпляр MoonWalk и качество теней из перечисления ShadowQuality
    pub fn set_shadow_quality(&mut self, mw: &mut MoonWalk, quality: ShadowQuality) {
        if self.shadow_quality == quality {
            return;
        }

        self.shadow_quality = quality;
        
        if quality == ShadowQuality::Off {
            return;
        }
        
        let size = match quality {
            ShadowQuality::Low => 512,
            ShadowQuality::Medium => 1024,
            ShadowQuality::High => 2048,
            ShadowQuality::Ultra => 4096,
            ShadowQuality::Off => 1,
        };
        
        let mut new_shadow_paint = mw.new_custom_paint(size, size, "Shadow Map Resized");
        let _ = mw.renderer.state.textures.remove(&self.shadow_map_id);
        
        let dummy = crate::rendering::texture::Texture::create_depth_texture(&mw.renderer.context, 1, 1, "Dummy");
        let new_depth = std::mem::replace(&mut new_shadow_paint.depth, dummy);
        
        self.shadow_map_id = mw.renderer.state.add_texture(new_depth);
        self.shadow_paint = new_shadow_paint;
        self.shadow_map_size = size;
        self.global_bg = None;
    }

    /// Этот метод позволяет загрузить obj 3д модель, этот формат очень популярен
    /// и явлется текстовым. Загрузка происходит через передачу байтов
    pub fn load_obj(&mut self, mw: &MoonWalk, obj_bytes: &[u8]) -> Vec<MeshId> {
        let loaded_meshes = loader::load_obj(mw, obj_bytes);
        self.register_meshes(loaded_meshes)
    }
    
    /// Этот метод позволяет загрузить 3д модель формата gltf из файла на диске
    pub fn load_gltf(&mut self, mw: &mut MoonWalk, path: &str) -> Vec<MeshId> {
        let loaded_meshes = loader::load_gltf(mw, path);
        self.register_meshes(loaded_meshes)
    }

    /// Этот метод регистрирует мэш
    fn register_meshes(&mut self, new_meshes: Vec<MeshData>) -> Vec<MeshId> {
        let mut ids = Vec::new();
        let start_id = self.meshes.len();
        
        for (i, mesh) in new_meshes.into_iter().enumerate() {
            self.meshes.push(mesh);
            ids.push(MeshId(start_id + i));
        }
        ids
    }

    /// Этот метод создаёт новый объект со стандартными настройками
    pub fn new_instance(&mut self, mesh_id: MeshId) -> InstanceId {
        if let Some(idx) = self.inst_free_slots.pop() {
            self.inst_alive[idx] = true;
            self.inst_mesh_ids[idx] = mesh_id.0;
            self.inst_positions[idx] = Vec3::ZERO;
            self.inst_rotations[idx] = Vec3::ZERO;
            self.inst_scales[idx] = Vec3::ONE;
            
            self.inst_mat_base_color[idx] = Vec4::ONE;
            self.inst_mat_metallic[idx] = 0.0;
            self.inst_mat_roughness[idx] = 0.5;
            self.inst_mat_albedo_id[idx] = None;
            self.inst_mat_normal_id[idx] = None;
            self.inst_mat_mr_id[idx] = None;
            
            self.inst_parents[idx] = None;
            self.inst_unlit[idx] = false;
            
            return InstanceId(idx);
        }

        let idx = self.inst_positions.len();
        self.inst_alive.push(true);
        self.inst_mesh_ids.push(mesh_id.0);
        self.inst_positions.push(Vec3::ZERO);
        self.inst_rotations.push(Vec3::ZERO);
        self.inst_scales.push(Vec3::ONE);
        
        self.inst_mat_base_color.push(Vec4::ONE);
        self.inst_mat_metallic.push(0.0);
        self.inst_mat_roughness.push(0.5);
        self.inst_mat_albedo_id.push(None);
        self.inst_mat_normal_id.push(None);
        self.inst_mat_mr_id.push(None);
        
        self.inst_parents.push(None);
        self.inst_unlit.push(false);

        InstanceId(idx)
    }

    /// Этот метод удаляет объект, помечая его мёртвым. Здесь работает такая
    /// же система реинкарнации как и в 2д рендеринге в ObjectStore
    pub fn remove_instance(&mut self, id: InstanceId) {
        if id.0 < self.inst_alive.len() {
            if self.inst_alive[id.0] {
                self.inst_alive[id.0] = false;
                self.inst_free_slots.push(id.0);
            }
        }
    }

    /// Устанавливает позицию объекта, принимает vec3 из glam
    #[inline]
    pub fn set_position(&mut self, id: InstanceId, pos: Vec3) {
        if id.0 < self.inst_positions.len() {
            self.inst_positions[id.0] = pos;
        }
    }

    /// Устанавливает вращение объекта, принимает vec3 из glam
    #[inline]
    pub fn set_rotation(&mut self, id: InstanceId, rot: Vec3) {
        if id.0 < self.inst_rotations.len() {
            self.inst_rotations[id.0] = rot;
        }
    }

    /// Устанавливает масштаб объекта, принимает vec3 из glam
    #[inline]
    pub fn set_scale(&mut self, id: InstanceId, scale: Vec3) {
        if id.0 < self.inst_scales.len() {
            self.inst_scales[id.0] = scale;
        }
    }

    /// Устанавливает цвет объекта, принимает vec3 из glam
    #[inline]
    pub fn set_color(&mut self, id: InstanceId, color: Vec4) {
        if id.0 < self.inst_mat_base_color.len() {
            self.inst_mat_base_color[id.0] = color;
        }
    }

    /// [WAIT DOC]
    #[inline]
    pub fn set_metallic(&mut self, id: InstanceId, val: f32) {
        if id.0 < self.inst_mat_metallic.len() {
            self.inst_mat_metallic[id.0] = val;
        }
    }

    /// [WAIT DOC]
    #[inline]
    pub fn set_roughness(&mut self, id: InstanceId, val: f32) {
        if id.0 < self.inst_mat_roughness.len() {
            self.inst_mat_roughness[id.0] = val;
        }
    }

    /// Устанавливает текстуру объекту, тут используются обычные текстуры которые
    /// общие с 3д и 2д графикой
    #[inline]
    pub fn set_texture(&mut self, id: InstanceId, tex_id: u32) {
        if id.0 < self.inst_mat_albedo_id.len() {
            self.inst_mat_albedo_id[id.0] = Some(tex_id);
        }
    }
    
    /// Этот метод устанавливает карту нормалей. Она используется для имитации
    /// объёма у простых моделей, благодаря тому что освещение учитывает виртуальные
    /// нормали из карты. Текстуру для карты нормалей лучше не создавать самому,
    /// чаще всего фотореалистичные текстуры поставляются с картой нормалей в
    /// комплекте
    #[inline]
    pub fn set_normal_map(&mut self, id: InstanceId, tex_id: u32) {
        if id.0 < self.inst_mat_normal_id.len() {
            self.inst_mat_normal_id[id.0] = Some(tex_id);
        }
    }
    
    /// [WAIT DOC]
    #[inline]
    pub fn set_mr_map(&mut self, id: InstanceId, tex_id: u32) {
        if id.0 < self.inst_mat_mr_id.len() {
            self.inst_mat_mr_id[id.0] = Some(tex_id);
        }
    }

    /// Получает позицию объекта по его айди, возвращает vec3 из glam
    #[inline]
    pub fn get_position(&self, id: InstanceId) -> Vec3 {
        if id.0 < self.inst_positions.len() {
            self.inst_positions[id.0]
        } else {
            Vec3::ZERO
        }
    }

    /// Получает позицию объекта по его айди, возвращает vec3 из glam
    #[inline]
    pub fn get_rotation(&self, id: InstanceId) -> Vec3 {
        if id.0 < self.inst_positions.len() {
            self.inst_rotations[id.0]
        } else {
            Vec3::ZERO
        }
    }

    /// Получает масштаб объекта по его айди, возвращает vec3 из glam
    #[inline]
    pub fn get_scale(&self, id: InstanceId) -> Vec3 {
        if id.0 < self.inst_positions.len() {
            self.inst_scales[id.0]
        } else {
            Vec3::ZERO
        }
    }
    
    /// Создаёт новый источник света
    pub fn new_light(&mut self) -> LightId {
        if let Some(idx) = self.lights_free_slots.pop() {
            self.lights_active[idx] = true;
            self.lights_data[idx] = LightRaw::default();
            return LightId(idx);
        }
        
        let idx = self.lights_data.len();
        if idx >= MAX_LIGHTS {
            eprintln!("MoonWalk Warning: Max lights limit ({}) reached", MAX_LIGHTS);
            return LightId(0);
        }
        
        self.lights_active.push(true);
        self.lights_data.push(LightRaw::default());
        LightId(idx)
    }
    
    /// Удаляет источник света. Тут также действует система реинкарнации
    pub fn remove_light(&mut self, id: LightId) {
        if id.0 < self.lights_active.len() {
            self.lights_active[id.0] = false;
            self.lights_free_slots.push(id.0);
        }
    }

    /// Этот метод устанавливает позицию для источника света, vec3 из glam
    #[inline]
    pub fn set_light_position(&mut self, id: LightId, pos: Vec3) {
        if id.0 < self.lights_data.len() {
            self.lights_data[id.0].position = pos.to_array();
        }
    }

    /// Этот метод устанавливает цвет для источника света, vec3 из glam
    #[inline]
    pub fn set_light_color(&mut self, id: LightId, color: Vec3) {
        if id.0 < self.lights_data.len() {
            self.lights_data[id.0].color = color.to_array();
        }
    }

    /// Этот метод устанавливает интенсивность источника света
    #[inline]
    pub fn set_light_intensity(&mut self, id: LightId, intensity: f32) {
        if id.0 < self.lights_data.len() { self.lights_data[id.0].intensity = intensity; }
    }
    
    /// Этот метод устанавливает тип освещения из перечисоления LightingModel
    /// PBR выглядит реалистичнее, но тяжелее для устройства
    /// Phong выглядит проще, но легче для устройства
    pub fn set_lighting_model(&mut self, model: LightingModel) {
        self.lighting_model = model;
    }

    /// Этот метод рендерит 3д сцену
    pub fn render(&mut self, mw: &mut MoonWalk) -> u32 {
        let (global_u, shadow_u) = render::update_uniforms(mw, &self, self.width, self.height);
        
        mw.update_buffer(&self.global_buf, bytes_of(&global_u));
        mw.update_buffer(&self.shadow_buf, bytes_of(&shadow_u));
        
        let (batches, offsets, all_data) = batch::prepare_batches(&self);
        
        if all_data.len() > self.instance_capacity {
            self.instance_capacity = (all_data.len() * 2).max(1024);
            let size = self.instance_capacity * std::mem::size_of::<InstanceRaw>();
            self.instance_buffer = mw.create_vertex_buffer(&vec![0u8; size]);
        }

        mw.update_buffer(&self.instance_buffer, cast_slice(&all_data));

        let commands = render::prepare_draw_commands(
            mw, &batches, &offsets, &self.material_layout, 
            &mut self.material_bg_cache, self.default_normal, self.default_white
        );

        if self.shadow_quality != ShadowQuality::Off {
            let shadow_depth_tex = mw.renderer.state.textures.remove(&self.shadow_map_id).expect("Shadow map missing");
            let old_dummy = std::mem::replace(&mut self.shadow_paint.depth, shadow_depth_tex);
            
            if self.shadow_bg.is_none() {
                self.shadow_bg = Some(mw.create_bind_group(&self.shadow_global_layout, &[BindResource::Uniform(&self.shadow_buf)]).unwrap());
            }

            struct ShadowDrawItem {
                vb: MoonBuffer,
                ib: MoonBuffer,
                count: u32,
                off: u64,
                size: u64
            }
            let mut shadow_items = Vec::with_capacity(commands.len());
            
            for cmd in &commands {
                 if let Some(mesh) = self.meshes.get(cmd.mesh_id) {
                     let size = cmd.count as u64 * std::mem::size_of::<InstanceRaw>() as u64;
                     let offset = cmd.offset as u64 * std::mem::size_of::<InstanceRaw>() as u64;
                     shadow_items.push(ShadowDrawItem {
                         vb: mesh.vertex_buffer.clone(),
                         ib: mesh.index_buffer.clone(),
                         count: mesh.index_count,
                         off: offset,
                         size
                     });
                 }
            }

            let pipe = self.shadow_pipeline.clone();
            let bg = self.shadow_bg.clone();
            let inst_buf = self.instance_buffer.clone();

            self.shadow_paint.start_frame(&mw.renderer.context);
            if let Some(mut pass) = self.shadow_paint.render_pass(MoonRenderPass::new().set_clear_depth(true)) {
                pass.set_pipeline(&pipe);
                if let Some(b) = &bg { pass.set_bind_group(0, b); }

                for item in &shadow_items {
                    pass.set_vertex_buffer(0, &item.vb, 0, None);
                    pass.set_vertex_buffer(1, &inst_buf, item.off, Some(item.size));
                    pass.set_index_buffer(&item.ib, 0, None);
                    
                    let inst_count = item.size as u32 / std::mem::size_of::<InstanceRaw>() as u32;
                    pass.draw_indexed(0..item.count, 0, 0..inst_count);
                }
            }

            self.shadow_paint.submit_frame(&mw.renderer.context);
            
            let rendered_shadow_tex = std::mem::replace(&mut self.shadow_paint.depth, old_dummy);
            mw.renderer.state.textures.insert(self.shadow_map_id, rendered_shadow_tex);
        }

        if self.global_bg.is_none() {
            self.global_bg = Some(mw.create_bind_group(&self.global_layout, &[
                BindResource::Uniform(&self.global_buf),
                BindResource::Texture(self.shadow_map_id),
                BindResource::Sampler(self.shadow_map_id)
            ]).unwrap());
        }

        struct MainDrawItem {
            vb: MoonBuffer,
            ib: MoonBuffer,
            bg: MoonBindGroup,
            count: u32,
            off: u64,
            size: u64
        }

        let mut main_items = Vec::with_capacity(commands.len());

        for cmd in commands {
             if let Some(mesh) = self.meshes.get(cmd.mesh_id) {
                 let size = cmd.count as u64 * std::mem::size_of::<InstanceRaw>() as u64;
                 let offset = cmd.offset as u64 * std::mem::size_of::<InstanceRaw>() as u64;
                 main_items.push(MainDrawItem {
                     vb: mesh.vertex_buffer.clone(),
                     ib: mesh.index_buffer.clone(),
                     bg: cmd.bind_group.clone(),
                     count: mesh.index_count,
                     off: offset,
                     size
                 });
             }
        }
        
        let pipeline = match self.lighting_model {
            LightingModel::Pbr => self.pipeline_pbr.clone(),
            LightingModel::Phong => self.pipeline_phong.clone(),
            LightingModel::Custom => self.custom_pipeline_override.as_ref().unwrap_or(&self.pipeline_pbr).clone(),
        };
        let global_bg = self.global_bg.clone();
        let inst_buf = self.instance_buffer.clone();

        self.paint.start_frame(&mw.renderer.context);
        
        if let Some(mut pass) = self.paint.render_pass(MoonRenderPass::new().set_clear_color(Some(glam::Vec4::ZERO)).set_clear_depth(true)) {
            pass.set_pipeline(&pipeline);
            if let Some(bg) = &global_bg { pass.set_bind_group(0, bg); }
            
            for item in &main_items {
                pass.set_bind_group(1, &item.bg);
                pass.set_vertex_buffer(0, &item.vb, 0, None);
                pass.set_vertex_buffer(1, &inst_buf, item.off, Some(item.size));
                pass.set_index_buffer(&item.ib, 0, None);
                
                let inst_count = item.size as u32 / std::mem::size_of::<InstanceRaw>() as u32;
                pass.draw_indexed(0..item.count, 0, 0..inst_count);
            }
        }

        self.paint.submit_frame(&mw.renderer.context);

        self.paint.update_snapshot(mw, self.snapshot_id);
        self.snapshot_id
    }
}
