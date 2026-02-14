// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use glam::{Vec2, Vec4};

#[cfg(feature = "modern")]
use moonwalk_backend::core::context::BackendContext;

#[cfg(feature = "modern")]
use moonwalk_backend::core::encoder::BackendEncoder;

#[cfg(feature = "modern")]
use moonwalk_backend::core::buffer::BackendBuffer;

#[cfg(feature = "modern")]
use moonwalk_backend::pipeline::bind::RawBindGroup;

#[cfg(feature = "modern")]
use moonwalk_backend::render::texture::BackendTexture;

#[cfg(feature = "modern")]
use moonwalk_backend::render::pass::RenderPass;

#[cfg(feature = "modern")]
use moonwalk_backend::error::MoonBackendError;

#[cfg(feature = "modern")]
use crate::error::MoonWalkError;

#[cfg(not(feature = "modern"))]
use crate::gpu::context::Context;

#[cfg(not(feature = "modern"))]
use crate::gpu::Buffer;

use crate::gpu::MatrixStack;

use crate::objects::store::ObjectStore;
use crate::objects::ObjectId;
use crate::batching::shapes::uber::UberBatch;
use crate::rendering::snapshot::ClippedSnapshot;
use crate::rendering::state::GlobalUniform;

#[cfg(not(feature = "modern"))]
use crate::rendering::texture::Texture;

use crate::textware::FontId;
use crate::MoonWalk;
use crate::FontAsset;
use crate::TextAlign;

#[cfg(feature = "modern")]
pub struct RenderContainer {
    pub store: ObjectStore,
    pub batch: UberBatch,
    pub proj_bind_group: RawBindGroup,
    pub target: BackendTexture,
    pub width: u32,
    pub height: u32,
}

#[cfg(not(feature = "modern"))]
pub struct RenderContainer {
    pub store: ObjectStore,
    pub batch: UberBatch,
    pub proj_bind_group: wgpu::BindGroup,
    pub target: Texture,
    pub width: u32,
    pub height: u32,
}

#[cfg(not(feature = "modern"))]
impl RenderContainer {
    pub fn new(ctx: &Context, width: u32, height: u32) -> Self {
        let target = crate::rendering::texture::Texture::create_empty(
            ctx, 
            width, 
            height, 
            ctx.config.format,
            "Container Target"
        );

        let mut matrix_stack = MatrixStack::new();
        matrix_stack.set_ortho(width as f32, height as f32);
        
        let uniform_data = GlobalUniform {
            view_proj: matrix_stack.projection.to_cols_array_2d(),
        };
        let uniform_buffer = Buffer::uniform(ctx, &uniform_data);
        
        let proj_layout = ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Container Proj Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let proj_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &proj_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.raw.as_entire_binding(),
            }],

            label: Some("Container Proj Bind Group"),
        });

        Self {
            store: ObjectStore::new(),
            batch: UberBatch::new(ctx),
            proj_bind_group,
            target,
            width,
            height,
        }
    }
}

#[cfg(feature = "modern")]
impl RenderContainer {
    pub fn new(context: &mut BackendContext, width: u32, height: u32) -> Result<Self, MoonWalkError> {
        use moonwalk_backend::pipeline::bind::BindGroup;
        use moonwalk_backend::pipeline::types::ShaderStage;

        let format = context.get_format(); 

        let mut target = BackendTexture::new(width, height);
        target.config.set_format(format);
        target.create_render_target(context, width, height)?;

        let mut matrix_stack = MatrixStack::new();
        matrix_stack.set_ortho(width as f32, height as f32);
        
        let uniform_data = GlobalUniform {
            view_proj: matrix_stack.projection.to_cols_array_2d(),
        };

        let uniform_buffer = BackendBuffer::uniform(
            context, &uniform_data
        )?;
        
        let proj_layout = BindGroup::new()
            .add_uniform(0, ShaderStage::Vertex)
            .build(context)?;

        let proj_bind_group = BindGroup::create_uniform_bind_group(
            &proj_layout, context, &uniform_buffer, Some("Container Proj Bind Group")
        )?;

        Ok(Self {
            store: ObjectStore::new(),
            batch: UberBatch::new(context)?,
            proj_bind_group,
            target,
            width,
            height,
        })
    }
}

impl RenderContainer {
    pub fn new_rect(&mut self) -> ObjectId {
        self.store.new_rect()
    }

    #[inline]
    #[deprecated(note = "Use set_position instead")]
    pub fn config_position(&mut self, id: ObjectId, pos: Vec2) {
        self.store.config_position(id, pos);
    }

    #[inline]
    pub fn set_position(&mut self, id: ObjectId, pos: Vec2) {
        self.store.config_position(id, pos);
    }

    #[inline]
    pub fn get_position(&self, id: ObjectId) -> Vec2 {
        self.store.get_position(id)
    }

    #[inline]
    #[deprecated(note = "Use set_size instead")]
    pub fn config_size(&mut self, id: ObjectId, size: Vec2) {
        self.store.config_size(id, size);
    }

    #[inline]
    pub fn set_size(&mut self, id: ObjectId, size: Vec2) {
        self.store.config_size(id, size);
    }

    #[inline]
    pub fn get_size(&self, id: ObjectId) -> Vec2 {
        self.store.get_size(id)
    }

    #[inline]
    #[deprecated(note = "Use set_color instead")]
    pub fn config_color(&mut self, id: ObjectId, color: Vec4) {
        self.store.config_color(id, color);
    }

    #[inline]
    pub fn set_color(&mut self, id: ObjectId, color: Vec4) {
        self.store.config_color(id, color);
    }

    #[inline]
    pub fn get_color(&self, id: ObjectId) -> Vec4 {
        self.store.get_color(id)
    }

    #[inline]
    #[deprecated(note = "Use set_color2 instead")]
    pub fn config_color2(&mut self, id: ObjectId, color: Vec4) {
        self.store.config_color2(id, color);
    }

    #[inline]
    pub fn set_color2(&mut self, id: ObjectId, color: Vec4) {
        self.store.config_color2(id, color);
    }

    #[inline]
    pub fn get_color2(&self, id: ObjectId) -> Vec4 {
        self.store.get_color2(id)
    }
    
    #[inline]
    #[deprecated(note = "Use set_rotation instead")]
    pub fn config_rotation(&mut self, id: ObjectId, rad: f32) {
        self.store.config_rotation(id, rad);
    }

    #[inline]
    pub fn set_rotation(&mut self, id: ObjectId, rad: f32) {
        self.store.config_rotation(id, rad);
    }

    #[inline]
    pub fn get_rotation(&self, id: ObjectId) -> f32 {
        self.store.get_rotation(id)
    }

    #[inline]
    #[deprecated(note = "Use set_z_index instead")]
    pub fn config_z_index(&mut self, id: ObjectId, z: f32) {
        self.store.config_z_index(id, z);
    }

    #[inline]
    pub fn set_z_index(&mut self, id: ObjectId, z: f32) {
        self.store.config_z_index(id, z);
    }

    #[inline]
    pub fn get_z_index(&self, id: ObjectId) -> f32 {
        self.store.get_z_index(id)
    }

    #[inline]
    #[deprecated(note = "Use set_uv instead")]
    pub fn config_uv(&mut self, id: ObjectId, uv: [f32; 4]) {
        self.store.config_uv(id, uv);
    }

    #[inline]
    pub fn set_uv(&mut self, id: ObjectId, uv: [f32; 4]) {
        self.store.config_uv(id, uv);
    }

    #[inline]
    pub fn set_rounded(&mut self, id: ObjectId, radii: Vec4) {
        self.store.set_rounded(id, radii);
    }

    #[inline]
    pub fn get_rounded(&self, id: ObjectId) -> Vec4 {
        self.store.get_rounded(id)
    }

    #[inline]
    #[deprecated(note = "Use set_texture instead")]
    pub fn config_texture(&mut self, id: ObjectId, texture_id: u32) {
        self.store.config_texture(id, texture_id);
    }

    #[inline]
    pub fn set_texture(&mut self, id: ObjectId, texture_id: u32) {
        self.store.config_texture(id, texture_id);
    }

    #[inline]
    #[deprecated(note = "Use set_gradient_data instead")]
    pub fn config_gradient_data(&mut self, id: ObjectId, gradient_data: [f32; 4]) {
        self.store.config_gradient_data(id, gradient_data);
    }

    #[inline]
    pub fn set_gradient_data(&mut self, id: ObjectId, gradient_data: [f32; 4]) {
        self.store.config_gradient_data(id, gradient_data);
    }

    pub fn new_text(&mut self, content: &str, font: FontAsset, size: f32) -> crate::objects::ObjectId {
        let internal_id = FontId(font.0);
        self.store.new_text(content.to_string(), internal_id, size)
    }

    #[inline]
    pub fn set_text(&mut self, id: crate::objects::ObjectId, content: &str) {
        self.store.set_text(id, content.to_string());
    }

    #[inline]
    pub fn get_text(&self, id: crate::objects::ObjectId) -> String {
        self.store.get_text(id).to_string()
    }

     #[inline]
    pub fn set_font_size(&mut self, id: crate::objects::ObjectId, size: f32) {
        self.store.set_font_size(id, size);
    }

    #[inline]
    pub fn get_font_size(&self, id: crate::objects::ObjectId) -> f32 {
        self.store.get_font_size(id)
    }

    #[inline]
    pub fn set_text_size(&mut self, id: crate::objects::ObjectId, w: f32, h: f32) {
        self.store.set_text_bounds(id, w, h);
    }

    #[inline]
    pub fn get_text_size(&self, id: crate::objects::ObjectId) -> Vec2 {
        self.store.get_text_bounds(id)
    }

    #[inline]
    pub fn set_text_align(&mut self, id: crate::objects::ObjectId, align: TextAlign) {
        let val = match align {
            TextAlign::Left => 0,
            TextAlign::Center => 1,
            TextAlign::Right => 2,
            TextAlign::Justified => 3,
        };
        self.store.set_text_align(id, val);
    }

    #[inline]
    pub fn get_text_align(&self, id: crate::objects::ObjectId) -> TextAlign {
        match self.store.get_text_align(id) {
            0 => TextAlign::Left,
            1 => TextAlign::Center,
            2 => TextAlign::Right,
            3 => TextAlign::Justified,
            _ => TextAlign::Left,
        }
    }

    pub fn measure_text(&mut self, mw: &mut MoonWalk, text: &str, font: FontAsset, size: f32, max_width: f32) -> Vec2 {
        let (w, h) = mw.renderer.text_engine.measure_text(
            text, 
            crate::textware::FontId(font.0), 
            size, 
            max_width
        );
        Vec2::new(w, h)
    }
    
    pub fn remove(&mut self, id: ObjectId) {
        self.store.remove(id);
    }

    #[inline]
    pub fn set_effect(&mut self, id: crate::objects::ObjectId, border_width: f32, box_shadow: f32) {
        self.store.config_effect_data(id, [border_width, box_shadow]);
    }

    #[cfg(feature = "modern")]
    pub fn draw(&mut self, mw: &mut MoonWalk, clear_color: Option<Vec4>) -> Result<(), MoonWalkError> {
        let renderer = &mut mw.renderer;
        let context = &mut renderer.context;
        let text_engine = &mut renderer.text_engine;
        
        self.batch.prepare(context, &self.store, text_engine);

        text_engine.prepare(context);
        let atlas_bg = text_engine.get_bind_group()?;
        
        let clear_color = clear_color.map(|c| Vec4::new(c.x, c.y, c.z, c.w));

        let mut encoder = BackendEncoder::new(context, "Render Container Encoder")?;

        let mut pass = RenderPass::new(
            &mut encoder,
            &self.target,
            clear_color,
            "Render Container Pass".to_string()
        )?;

        if let Some(pipeline) = renderer.state.shaders.get_pipeline(renderer.state.rect_shader) {
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &self.proj_bind_group);
            
            self.batch.render(
                &mut pass, 
                &renderer.state.white_texture, 
                &renderer.state.textures,
                Some(&atlas_bg),
            );
        }

        drop(pass);

        encoder.submit_frame(context)?;

        Ok(())
    }

    #[cfg(feature = "modern")]
    pub fn snapshot(&mut self, mw: &mut MoonWalk, x: u32, y: u32, w: u32, h: u32) -> u32 {
        // [HACK] [UNWRAP]
        // Для того чтобы не ломать api тут используется expect и unwrap везде,
        // потом желательно заменить на Result
        // Result<u32, MoonWalkError>

        let renderer = &mut mw.renderer;
        let format = renderer.context.get_format();

        let mut snapshot_region = ClippedSnapshot::new(
            Vec2::new(x as f32, y as f32),
            Vec2::new(w as f32, h as f32),
        );

        snapshot_region.clip_snapshot(Vec2::new(
            self.width as f32,
            self.height as f32
        ));
        
        let mut snapshot_texture = BackendTexture::new(snapshot_region.size.x as u32, snapshot_region.size.y as u32);
        snapshot_texture.config.set_format(format);
        snapshot_texture.create_render_target(&mut renderer.context, snapshot_region.size.x as u32, snapshot_region.size.y as u32)
            .expect("DELETE THIS PLEASE");

        let id = renderer.state.add_texture(snapshot_texture);
        let target_tex = renderer.state.textures.get(&id).unwrap();
        
        let mut encoder = BackendEncoder::new(&mut renderer.context, "Snapshot Encoder")
            .expect("DELETE THIS PLEASE");

        encoder.copy_texture_to_texture(
            snapshot_region.position.x as u32,
            snapshot_region.position.y as u32,
            snapshot_region.size.x as u32,
            snapshot_region.size.y as u32,
            &self.target.get_raw().unwrap(),
            &target_tex.get_raw().unwrap()
        ).expect("DELETE THIS PLEASE");

        encoder.submit_frame(&mut renderer.context).expect("DELETE THIS PLEASE");

        // Ok(id)
        id
    }

    #[cfg(feature = "modern")]
    pub fn update_snapshot(
        &mut self,
        mw: &mut MoonWalk,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        id: u32
    ) {
        // [HACK] [UNWRAP]
        // Для того чтобы не ломать api тут используется expect и unwrap везде,
        // потом желательно заменить на Result
        // Result<(), MoonWalkError>

        let renderer = &mut mw.renderer;
        
        let mut snapshot_region = ClippedSnapshot::new(
            Vec2::new(x as f32, y as f32),
            Vec2::new(w as f32, h as f32),
        );

        snapshot_region.clip_snapshot(Vec2::new(
            self.width as f32,
            self.height as f32
        ));

        let target_tex = renderer.state.textures.get(&id).unwrap();
        
        let mut encoder = BackendEncoder::new(&mut renderer.context, "Update Snapshot Encoder")
            .expect("DELETE THIS PLEASE");

        encoder.copy_texture_to_texture(
            snapshot_region.position.x as u32,
            snapshot_region.position.y as u32,
            snapshot_region.size.x as u32,
            snapshot_region.size.y as u32,
            &self.target.get_raw().unwrap(),
            &target_tex.get_raw().unwrap()
        ).expect("DELETE THIS PLEASE");

        encoder.submit_frame(&mut renderer.context).expect("DELETE THIS PLEASE");

        // Ok(())
    }

    #[cfg(not(feature = "modern"))]
    pub fn draw(&mut self, mw: &mut MoonWalk, clear_color: Option<Vec4>) {
        let renderer = &mut mw.renderer;
        let ctx = &renderer.context;
        let text_engine = &mut renderer.text_engine;
        
        self.batch.prepare(ctx, &self.store, text_engine);

        text_engine.prepare(&ctx.queue);
        let atlas_bg = text_engine.get_bind_group();
        
        let wgpu_clear_color = clear_color.map(|c| wgpu::Color {
            r: c.x as f64,
            g: c.y as f64,
            b: c.z as f64,
            a: c.w as f64,
        });

        let mut encoder = ctx.create_encoder();
        let view = &self.target.view;
        {
            let mut pass = crate::gpu::RenderPass::new(
                &mut encoder,
                view,
                wgpu_clear_color
            );
            
            if let Some(pipeline) = renderer.state.shaders.get_pipeline(renderer.state.rect_shader) {
                pass.set_pipeline(pipeline);
                pass.set_bind_group(0, &self.proj_bind_group);
                
                self.batch.render(
                    &mut pass, 
                    &renderer.state.white_texture, 
                    &renderer.state.textures,
                    Some(&atlas_bg),
                );
            }
        }
        
        ctx.submit(encoder);
    }

    #[cfg(not(feature = "modern"))]
    pub fn snapshot(&mut self, mw: &mut MoonWalk, x: u32, y: u32, w: u32, h: u32) -> u32 {
        let renderer = &mut mw.renderer;

        let mut snapshot_region = ClippedSnapshot::new(
            Vec2::new(x as f32, y as f32),
            Vec2::new(w as f32, h as f32),
        );

        snapshot_region.clip_snapshot(Vec2::new(
            self.width as f32,
            self.height as f32
        ));
        
        let result = Texture::create_render_target(
            &renderer.context,
            snapshot_region.size.x as u32,
            snapshot_region.size.y as u32,
            self.target.texture.format()
        );
        
        let id = renderer.state.add_texture(result);
        let target_tex = renderer.state.textures.get(&id).unwrap();
        
        let mut encoder = renderer.context.create_encoder();
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.target.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: snapshot_region.position.x as u32,
                    y: snapshot_region.position.y as u32,
                    z: 0
                },
                aspect: wgpu::TextureAspect::All,
            },

            wgpu::TexelCopyTextureInfo {
                texture: &target_tex.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },

            wgpu::Extent3d {
                width: snapshot_region.size.x as u32,
                height: snapshot_region.size.y as u32,
                depth_or_array_layers: 1
            }
        );
        
        renderer.context.submit(encoder);
        
        id
    }

    #[cfg(not(feature = "modern"))]
    pub fn update_snapshot(&mut self, mw: &mut MoonWalk, x: u32, y: u32, w: u32, h: u32, id: u32) {
        let renderer = &mut mw.renderer;
        
        let mut snapshot_region = ClippedSnapshot::new(
            Vec2::new(x as f32, y as f32),
            Vec2::new(w as f32, h as f32),
        );

        snapshot_region.clip_snapshot(Vec2::new(
            self.width as f32,
            self.height as f32
        ));

        let target_tex = renderer.state.textures.get(&id).unwrap();
        
        let mut encoder = renderer.context.create_encoder();
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.target.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: snapshot_region.position.x as u32,
                    y: snapshot_region.position.y as u32,
                    z: 0
                },
                aspect: wgpu::TextureAspect::All,
            },

            wgpu::TexelCopyTextureInfo {
                texture: &target_tex.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },

            wgpu::Extent3d {
                width: snapshot_region.size.x as u32,
                height: snapshot_region.size.y as u32,
                depth_or_array_layers: 1
            }
        );
        
        renderer.context.submit(encoder);
    }
}
