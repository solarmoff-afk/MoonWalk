// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use bytemuck::bytes_of;
use moonwalk::MoonWalk;
use moonwalk::rendering::custom::{CustomPaint, MoonRenderPass, CustomPipeline, MoonBuffer};
use moonwalk::BindResource;

use crate::core::types::ShadowQuality;
use crate::core::config::{MAX_SHADOWS, SHADOW_MAP_SIZE};
use crate::factory::LunarFactory;
use crate::internal::store::ObjectStore;
use crate::resources::InstanceRaw;

pub struct ShadowSystem {
    paint: CustomPaint,
    pipeline: CustomPipeline,
    map_id: u32,
    uniform_buf: MoonBuffer,
    pub quality: ShadowQuality,
    pub paused: bool,
    pub ortho_size: f32,
}

impl ShadowSystem {
    pub fn new(mw: &mut MoonWalk, factory: &LunarFactory) -> Self {
        let size = SHADOW_MAP_SIZE;
        let mut paint = mw.new_custom_paint(size, size, "ShadowMap");
        
        let dummy = moonwalk::rendering::texture::Texture::create_depth_texture(
            &mw.renderer.context, 1, 1, "Dummy"
        );

        let depth = std::mem::replace(&mut paint.depth, dummy);
        let map_id = mw.renderer.state.add_texture(depth);
        
        let pipeline = factory.create_shadow_pipeline(mw, include_str!("../shaders/shadow.wgsl"));
        let uniform_buf = mw.create_uniform_buffer(&[0; 256]);

        Self {
            paint,
            pipeline,
            map_id,
            uniform_buf,
            quality: ShadowQuality::High, // Высокое качество по умолчанию
            paused: false,
            ortho_size: 40.0,
        }
    }

    pub fn get_texture_id(&self) -> u32 {
        self.map_id
    }

    pub fn set_quality(&mut self, mw: &mut MoonWalk, quality: ShadowQuality) {
        if self.quality == quality {
            return;
        }

        self.quality = quality;

        let size = match quality {
            ShadowQuality::Off => 1,
            ShadowQuality::Low => 1024,
            ShadowQuality::Medium => 2048,
            ShadowQuality::High => 4096,
            ShadowQuality::Ultra => 8192,
        };

        let _ = mw.renderer.state.textures.remove(&self.map_id);
        let mut new_paint = mw.new_custom_paint(size, size, "ShadowMapResized");
        
        let dummy = moonwalk::rendering::texture::Texture::create_depth_texture(
            &mw.renderer.context, 1, 1, "Dummy"
        );

        let new_depth = std::mem::replace(&mut new_paint.depth, dummy);
        
        self.map_id = mw.renderer.state.add_texture(new_depth);
        self.paint = new_paint;
    }

    pub fn render(
        &mut self, 
        mw: &mut MoonWalk, 
        factory: &LunarFactory, 
        store: &ObjectStore,
        matrices: &[[[f32; 4]; 4]; MAX_SHADOWS],
        active_lights_count: usize
    ) {
        if self.quality == ShadowQuality::Off || self.paused {
            return;
        }

        if active_lights_count == 0 {
            return;
        }

        let shadow_tex = mw.renderer.state.textures.remove(&self.map_id)
            .expect("Shadow map missing");
            
        let old_dummy = std::mem::replace(&mut self.paint.depth, shadow_tex);

        let bg = mw.create_bind_group(&factory.shadow_global_layout, &[
            BindResource::Uniform(&self.uniform_buf)
        ]).unwrap();

        self.paint.start_frame(&mw.renderer.context);

        let passes_to_render = active_lights_count.min(MAX_SHADOWS);

        for i in 0..passes_to_render {
            let (offset, scale) = match i {
                0 => ([0.0, 0.0], 0.5f32),
                1 => ([0.5, 0.0], 0.5f32),
                2 => ([0.0, 0.5], 0.5f32),
                _ => ([0.5, 0.5], 0.5f32),
            };

            let u = crate::core::types::ShadowUniform {
                light_view_proj: matrices[i],
                atlas_offset: offset,
                atlas_scale: scale,
                _pad: 0.0,
            };
            
            mw.update_buffer(&self.uniform_buf, bytes_of(&u));

            let clear = if i == 0 {
                true
            } else {
               false
            };

            if let Some(mut pass) = self.paint.render_pass(MoonRenderPass::new()
                .set_clear_depth(clear))
            {
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &bg);

                let instance_size = std::mem::size_of::<InstanceRaw>() as u64;
                
                for k in 0..store.positions.len() {
                    if !store.alive[k] {
                        continue;
                    }
                    
                    let mesh_id = store.mesh_ids[k];
                    
                    if let Some(mesh) = factory.meshes.get(mesh_id) {
                        pass.set_vertex_buffer(0, &mesh.vertex_buffer, 0, None);
                        pass.set_index_buffer(&mesh.index_buffer, 0, None);
                        pass.set_vertex_buffer(1, &store.buffer, (k as u64) * instance_size, Some(instance_size));
                        pass.draw_indexed(0..mesh.index_count, 0, 0..1);
                    }
                }
            }
        }

        self.paint.submit_frame(&mw.renderer.context);
        
        let rendered_tex = std::mem::replace(&mut self.paint.depth, old_dummy);
        mw.renderer.state.textures.insert(self.map_id, rendered_tex);
    }
}
