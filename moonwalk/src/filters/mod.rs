// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

pub mod color_matrix;
pub mod uniforms;
pub mod factory;

use wgpu::util::DeviceExt;
use bytemuck::bytes_of;

use crate::gpu::context::Context;
use crate::gpu::Buffer;
use crate::rendering::texture::Texture;
use crate::r#abstract::*;
use crate::error::MoonWalkError;

use self::uniforms::*;

pub struct FilterSystem {
    swap_texture: Option<Texture>,
    
    blur_pipeline: wgpu::RenderPipeline,
    color_pipeline: wgpu::RenderPipeline,
    advanced_pipeline: wgpu::RenderPipeline,
    
    uniform_layout: wgpu::BindGroupLayout,
    texture_layout: wgpu::BindGroupLayout,
    advanced_texture_layout: wgpu::BindGroupLayout,
    
    dummy_vbo: Buffer<DummyVertex>,
}

impl FilterSystem {
    pub fn new(ctx: &Context) -> Result<Self, MoonWalkError> {
        let dummy_vertices = [DummyVertex { _dummy: 0.0 }];
        let dummy_vbo = Buffer::vertex(ctx, &dummy_vertices);

        let uniform_layout = BindGroup::new()
            .add_uniform(0, ShaderStage::Fragment)
            .build(ctx)?;

        let texture_layout = BindGroup::new()
            .add_texture(0, TextureType::Float)
            .add_sampler(1, SamplerType::Linear)
            .build(ctx)?;

        let advanced_texture_layout = BindGroup::new()
            .add_texture(0, TextureType::Float)
            .add_sampler(1, SamplerType::Linear)
            .add_texture(2, TextureType::Float)
            .build(ctx)?;

        let blur_pipeline = factory::create_blur_pipeline(ctx, &uniform_layout, &texture_layout)?;
        let color_pipeline = factory::create_color_pipeline(ctx, &uniform_layout, &texture_layout)?;
        let advanced_pipeline = factory::create_advanced_pipeline(ctx, &uniform_layout, &advanced_texture_layout)?;

        Ok(Self {
            swap_texture: None,
            blur_pipeline,
            color_pipeline,
            advanced_pipeline,
            uniform_layout,
            texture_layout,
            advanced_texture_layout,
            dummy_vbo,
        })
    }

    pub fn apply_blur(&mut self, ctx: &Context, target_texture: &Texture, radius: f32, horizontal: bool) {
        let width = target_texture.texture.width();
        let height = target_texture.texture.height();
        
        self.ensure_swap_texture(ctx, width, height, target_texture.texture.format());
        let swap = self.swap_texture.as_ref().unwrap();

        let dir = if horizontal { [1.0, 0.0] } else { [0.0, 1.0] };
        
        let uniform_data = BlurUniform {
            direction: dir,
            radius,
            _pad: 0.0,
            resolution: [width as f32, height as f32],
        };

        self.execute_pass(
            ctx,
            &self.blur_pipeline,
            target_texture,
            swap,
            bytes_of(&uniform_data)
        );

        self.blit_back(ctx, target_texture, swap, width, height);
    }

    pub fn apply_color_matrix(
        &mut self,
        ctx: &Context,
        target_texture: &Texture,
        matrix: [[f32; 4]; 4],
        offset: [f32; 4]
    ) {
        let width = target_texture.texture.width();
        let height = target_texture.texture.height();
        
        self.ensure_swap_texture(ctx, width, height, target_texture.texture.format());
        let swap = self.swap_texture.as_ref().unwrap();

        let uniform_data = ColorMatrixUniform { matrix, offset };

        self.execute_pass(
            ctx,
            &self.color_pipeline,
            target_texture,
            swap,
            bytes_of(&uniform_data)
        );

        self.blit_back(ctx, target_texture, swap, width, height);
    }

    pub fn apply_chromakey(
        &mut self,
        ctx: &Context,
        target_texture: &Texture,
        key_color: [f32; 3],
        tolerance: f32
    ) {
        let width = target_texture.texture.width();
        let height = target_texture.texture.height();
        
        self.ensure_swap_texture(ctx, width, height, target_texture.texture.format());
        let swap = self.swap_texture.as_ref().unwrap();

        let uniform_data = AdvancedUniform {
            key_color,
            tolerance,
            params: [1.0, 0.0, 0.0, 0.0],
        };

        self.execute_advanced_pass(
            ctx,
            target_texture,
            target_texture,
            swap,
            bytes_of(&uniform_data)
        );

        self.blit_back(ctx, target_texture, swap, width, height);
    }

    pub fn apply_stencil(
        &mut self,
        ctx: &Context,
        target_texture: &Texture,
        mask_texture: &Texture,
        invert: bool
    ) {
        let width = target_texture.texture.width();
        let height = target_texture.texture.height();
        
        self.ensure_swap_texture(ctx, width, height, target_texture.texture.format());
        let swap = self.swap_texture.as_ref().unwrap();

        let uniform_data = AdvancedUniform {
            key_color: [0.0; 3],
            tolerance: 0.0,
            params: [2.0, if invert { 1.0 } else { 0.0 }, 0.0, 0.0],
        };

        self.execute_advanced_pass(
            ctx,
            target_texture,
            mask_texture,
            swap,
            bytes_of(&uniform_data)
        );

        self.blit_back(ctx, target_texture, swap, width, height);
    }

    fn ensure_swap_texture(&mut self, ctx: &Context, w: u32, h: u32, format: wgpu::TextureFormat) {
        let need_create = self.swap_texture.as_ref()
            .map_or(true, |t| t.texture.width() != w || t.texture.height() != h);

        if need_create {
            self.swap_texture = Some(Texture::create_render_target(ctx, w, h, format));
        }
    }

    fn blit_back(&self, ctx: &Context, target: &Texture, source: &Texture, width: u32, height: u32) {
        let mut encoder = ctx.create_encoder();
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &source.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &target.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d { width, height, depth_or_array_layers: 1 }
        );
        ctx.submit(encoder);
    }

    fn execute_pass(
        &self,
        ctx: &Context,
        pipeline: &wgpu::RenderPipeline,
        source: &Texture,
        dest: &Texture,
        uniform_bytes: &[u8]
    ) {
        let uniform_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Filter Uniform Buffer"),
            contents: uniform_bytes,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniform_bg = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Filter Uniform BG"),
            layout: &self.uniform_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding() }],
        });

        let texture_bg = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Filter Texture BG"),
            layout: &self.texture_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&source.view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&source.sampler) },
            ],
        });

        self.run_pipeline(ctx, pipeline, dest, &uniform_bg, &texture_bg);
    }

    fn execute_advanced_pass(
        &self,
        ctx: &Context,
        source: &Texture,
        mask: &Texture,
        dest: &Texture,
        uniform_bytes: &[u8]
    ) {
        let uniform_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Advanced Filter Uniform Buffer"),
            contents: uniform_bytes,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniform_bg = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Filter Uniform BG"),
            layout: &self.uniform_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding() }],
        });

        let texture_bg = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Advanced Texture BG"),
            layout: &self.advanced_texture_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&source.view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&source.sampler) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&mask.view) },
            ],
        });

        self.run_pipeline(ctx, &self.advanced_pipeline, dest, &uniform_bg, &texture_bg);
    }

    fn run_pipeline(
        &self, 
        ctx: &Context, 
        pipeline: &wgpu::RenderPipeline, 
        dest: &Texture, 
        bg0: &wgpu::BindGroup, 
        bg1: &wgpu::BindGroup
    ) {
        let mut encoder = ctx.create_encoder();
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Filter Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &dest.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, bg0, &[]);
            pass.set_bind_group(1, bg1, &[]);
            pass.set_vertex_buffer(0, self.dummy_vbo.raw.slice(..));
            pass.draw(0..3, 0..1);
        }

        ctx.submit(encoder);
    }
}
