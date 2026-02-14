// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

pub mod color_matrix;
pub mod uniforms;
pub mod factory;

use bytemuck::{bytes_of, Pod};

#[cfg(feature = "modern")]
use moonwalk_backend::core::context::BackendContext;

#[cfg(feature = "modern")]
use moonwalk_backend::render::texture::BackendTexture;

#[cfg(feature = "modern")]
use moonwalk_backend::core::buffer::BackendBuffer;

#[cfg(feature = "modern")]
use moonwalk_backend::pipeline::PipelineResult;

#[cfg(feature = "modern")]
use moonwalk_backend::pipeline::bind::{BindGroup, RawBindGroupLayout, RawBindGroup};

#[cfg(feature = "modern")]
use moonwalk_backend::pipeline::types::{ShaderStage, TextureType, SamplerType};

#[cfg(feature = "modern")]
use moonwalk_backend::core::encoder::BackendEncoder;

#[cfg(feature = "modern")]
use moonwalk_backend::render::texture::BackendTextureFormat;

#[cfg(not(feature = "modern"))]
use crate::gpu::context::Context;

#[cfg(not(feature = "modern"))]
use crate::gpu::Buffer;

#[cfg(not(feature = "modern"))]
use crate::rendering::texture::Texture;

use crate::r#abstract::*;
use crate::error::MoonWalkError;

use self::uniforms::*;

#[cfg(feature = "modern")]
pub struct FilterSystem {
    swap_texture: Option<BackendTexture>,

    blur_pipeline: PipelineResult,
    color_pipeline: PipelineResult,
    advanced_pipeline: PipelineResult,
    
    uniform_layout: RawBindGroupLayout,
    texture_layout: RawBindGroupLayout,
    advanced_texture_layout: RawBindGroupLayout,

    dummy_vbo: BackendBuffer<DummyVertex>,
}

#[cfg(not(feature = "modern"))]
pub struct FilterSystem {
    swap_texture: Option<BackendTexture>,
 
    blur_pipeline: wgpu::RenderPipeline,
    color_pipeline: wgpu::RenderPipeline,
    advanced_pipeline: wgpu::RenderPipeline,

    uniform_layout: wgpu::BindGroupLayout,
    texture_layout: wgpu::BindGroupLayout,
    advanced_texture_layout: wgpu::BindGroupLayout,

    dummy_vbo: Buffer<DummyVertex>,
}

impl FilterSystem {
    #[cfg(feature = "modern")]
    pub fn new(context: &mut BackendContext) -> Result<Self, MoonWalkError> {
        let dummy_vertices = [DummyVertex { _dummy: 0.0 }];
        let dummy_vbo = BackendBuffer::vertex(context, &dummy_vertices);

        let uniform_layout = BindGroup::new()
            .add_uniform(0, ShaderStage::Fragment)
            .build(context)?;

        let texture_layout = BindGroup::new()
            .add_texture(0, TextureType::Float)
            .add_sampler(1, SamplerType::Linear)
            .build(context)?;

        let advanced_texture_layout = BindGroup::new()
            .add_texture(0, TextureType::Float)
            .add_sampler(1, SamplerType::Linear)
            .add_texture(2, TextureType::Float)
            .build(context)?;

        let blur_pipeline = factory::create_blur_pipeline(context, &uniform_layout, &texture_layout)?;
        let color_pipeline = factory::create_color_pipeline(context, &uniform_layout, &texture_layout)?;
        let advanced_pipeline = factory::create_advanced_pipeline(context, &uniform_layout, &advanced_texture_layout)?;
        
        Ok(Self {
            swap_texture: None,
            blur_pipeline,
            color_pipeline,
            advanced_pipeline,
            uniform_layout,
            texture_layout,
            advanced_texture_layout,
            dummy_vbo: dummy_vbo?,
        })
    }

    #[cfg(not(feature = "modern"))]
    pub fn new(context: &Context) -> Result<Self, MoonWalkError> {
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

    #[cfg(feature = "modern")]
    pub fn apply_blur(
        &mut self,
        context: &mut BackendContext,
        target_texture: &BackendTexture,
        radius: f32,
        horizontal: bool
    ) -> Result<(), MoonWalkError> {
        let width = target_texture.width;
        let height = target_texture.height;
        
        self.ensure_swap_texture(context, width, height, target_texture.config.get_format());
        let swap = self.swap_texture.as_ref()
            .ok_or(MoonWalkError::TextureLoading("Failed to get filters system swap texture ref".to_string()))?;

        let dir = if horizontal {
            [1.0, 0.0]
        } else {
            [0.0, 1.0]
        };
        
        let uniform_data = BlurUniform {
            direction: dir,
            radius,
            _pad: 0.0,
            resolution: [width as f32, height as f32],
        };

        self.execute_pass(
            context,
            &self.blur_pipeline,
            target_texture,
            swap,
            &uniform_data,
        );

        self.blit_back(context, target_texture, swap, width, height)?;

        Ok(())
    }

    #[cfg(not(feature = "modern"))]
    pub fn apply_blur(&mut self, ctx: &Context, target_texture: &Texture, radius: f32, horizontal: bool) {
        let width = target_texture.texture.width();
        let height = target_texture.texture.height();
        
        self.ensure_swap_texture(ctx, width, height, target_texture.texture.format());
        let swap = self.swap_texture.as_ref().unwrap();

        let dir = if horizontal {
            [1.0, 0.0]
        } else {
            [0.0, 1.0]
        };
        
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

    #[cfg(feature = "modern")]
    pub fn apply_color_matrix(
        &mut self,
        context: &mut BackendContext,
        target_texture: &mut BackendTexture,
        matrix: [[f32; 4]; 4],
        offset: [f32; 4]
    )  -> Result<(), MoonWalkError> {
        let width = target_texture.width;
        let height = target_texture.height;
        
        self.ensure_swap_texture(context, width, height, target_texture.config.get_format());
        let swap = self.swap_texture.as_ref().unwrap();

        let uniform_data = ColorMatrixUniform {
            matrix, offset
        };

        self.execute_pass(
            context,
            &self.color_pipeline,
            target_texture,
            swap,
            &uniform_data,
        );

        self.blit_back(context, target_texture, swap, width, height)?;

        Ok(())
    }

    #[cfg(not(feature = "modern"))]
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

    #[cfg(feature = "modern")]
    pub fn apply_chromakey(
        &mut self,
        context: &mut BackendContext,
        target_texture: &mut BackendTexture,
        key_color: [f32; 3],
        tolerance: f32
    ) -> Result<(), MoonWalkError> {
        let width = target_texture.width;
        let height = target_texture.height;
        
        self.ensure_swap_texture(context, width, height, target_texture.config.get_format());
        let swap = self.swap_texture.as_ref().unwrap();

        let uniform_data = AdvancedUniform {
            key_color,
            tolerance,
            params: [1.0, 0.0, 0.0, 0.0],
        };

        self.execute_advanced_pass(
            context,
            target_texture,
            target_texture,
            swap,
            &uniform_data,
        );

        self.blit_back(context, target_texture, swap, width, height)?;

        Ok(())
    }

    #[cfg(not(feature = "modern"))]
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

    #[cfg(feature = "modern")]
    pub fn apply_stencil(
        &mut self,
        context: &mut BackendContext,
        target_texture: &BackendTexture,
        mask_texture: &BackendTexture,
        invert: bool
    ) -> Result<(), MoonWalkError> {
        let width = target_texture.width;
        let height = target_texture.height;
        
        self.ensure_swap_texture(context, width, height, target_texture.config.get_format());
        let swap = self.swap_texture.as_ref().unwrap();

        let uniform_data = AdvancedUniform {
            key_color: [0.0; 3],
            tolerance: 0.0,
            params: [2.0, if invert {
                1.0
            } else {
                0.0
            }, 0.0, 0.0],
        };

        self.execute_advanced_pass(
            context,
            target_texture,
            mask_texture,
            swap,
            &uniform_data,
        );

        self.blit_back(context, target_texture, swap, width, height)?;
    
        Ok(())
    }

    #[cfg(not(feature = "modern"))]
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

    #[cfg(feature = "modern")]
    fn ensure_swap_texture(&mut self, context: &mut BackendContext, w: u32, h: u32, format: BackendTextureFormat) {
        let need_create = self.swap_texture.as_ref()
            .map_or(true, |t| t.width != w || t.height != h);

        if need_create {
            let mut texture = BackendTexture::new(w, h);
            texture.config.set_format(format);
            texture.create_render_target(context, w, h);

            self.swap_texture = Some(texture);
        }
    }

    #[cfg(not(feature = "modern"))]
    fn ensure_swap_texture(&mut self, ctx: &Context, w: u32, h: u32, format: wgpu::TextureFormat) {
        let need_create = self.swap_texture.as_ref()
            .map_or(true, |t| t.texture.width() != w || t.texture.height() != h);

        if need_create {
            self.swap_texture = Some(Texture::create_render_target(ctx, w, h, format));
        }
    }

    #[cfg(feature = "modern")]
    fn blit_back(
        &self,
        context: &mut BackendContext,
        target: &BackendTexture,
        source: &BackendTexture,
        width: u32,
        height: u32
    ) -> Result<(), MoonWalkError> {
        let mut encoder = BackendEncoder::new(
            context, "MoonWalk encoder blit"
        )?;

        let source_raw = match source.get_raw() {
            Some(raw) => raw,
            None => return Err(MoonWalkError::BackendError("Texture is empty".to_string())),
        };

        let target_raw = match target.get_raw() {
            Some(raw) => raw,
            None => return Err(MoonWalkError::BackendError("Texture is empty".to_string())),
        };
        
        encoder.copy_texture_to_texture(0, 0, width, height, source_raw, target_raw);
        encoder.submit_frame(context);

        Ok(())
    }

    #[cfg(not(feature = "modern"))]
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

            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1
            }
        );
        
        ctx.submit(encoder);
    }

    #[cfg(feature = "modern")]
    fn execute_pass<T: Pod>(
        &self,
        context: &mut BackendContext,
        pipeline: &PipelineResult,
        source: &BackendTexture,
        dest: &BackendTexture,
        uniform_data: &T,
    ) -> Result<(), MoonWalkError> {
        let uniform_buffer = BackendBuffer::<T>::uniform_bytes(
            context, 
            bytemuck::bytes_of(uniform_data)
        )?;

        let uniform_bg = BindGroup::create_uniform_bind_group(
            &self.uniform_layout,
            context,
            &uniform_buffer,
            Some("Filter Uniform BG")
        )?;

        let texture_bg = BindGroup::create_texture_bind_group(
            &self.texture_layout,
            context,
            &[(source, 0)],
            &[(source, 1)],
            Some("Filter Texture BG")
        )?;

        self.run_pipeline(context, pipeline, dest, &uniform_bg, &texture_bg)?;
    
        Ok(())
    }

    #[cfg(not(feature = "modern"))]
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

    #[cfg(feature = "modern")]
    fn execute_advanced_pass<T: Pod>(
        &self,
        context: &mut BackendContext,
        source: &BackendTexture,
        mask: &BackendTexture,
        dest: &BackendTexture,
        uniform_data: &T,
    ) -> Result<(), MoonWalkError> {
        let uniform_buffer = BackendBuffer::<T>::uniform_bytes(
            context, 
            bytemuck::bytes_of(uniform_data)
        )?;

        let uniform_bg = BindGroup::create_uniform_bind_group(
            &self.uniform_layout,
            context,
            &uniform_buffer,
            Some("Filter Uniform BG")
        )?;

        let texture_bg = BindGroup::create_texture_bind_group(
            &self.advanced_texture_layout,
            context,
            &[(source, 0), (mask, 2)],
            &[(source, 1)],
            Some("Advanced Texture BG")
        )?;

        self.run_pipeline(context, &self.advanced_pipeline, dest, &uniform_bg, &texture_bg)?;
    
        Ok(())
    }

    #[cfg(not(feature = "modern"))]
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

    #[cfg(feature = "modern")]
    fn run_pipeline(
        &self, 
        context: &mut BackendContext, 
        pipeline: &PipelineResult, 
        dest: &BackendTexture, 
        bg0: &RawBindGroup, 
        bg1: &RawBindGroup,
    ) -> Result<(), MoonWalkError> {
        use glam::Vec4;
        use moonwalk_backend::render::pass::RenderPass;

        let mut encoder = BackendEncoder::new(context, "MoonWalk filters encoder")?;
        let mut pass = RenderPass::new(
            &mut encoder,
            dest,

            // Прозрачный цвет для заливки чтобы если фильтр применился к png картинке
            // это wgpu не перекрыл бы прозрачный фон своим каким-то цветом
            Some(Vec4::ZERO),
            "Filter render pass"
        )?;

        pass.set_pipeline(pipeline.get_raw()?);
        pass.set_bind_group(0, bg0);
        pass.set_bind_group(1, bg1);
        pass.set_vertex_buffer(0, &self.dummy_vbo);
        
        // Отрисовка !!! Можно выкидывать в мусор, работа завершена
        pass.draw(3);

        // Боров чекер хочет чтобы я выкинул на свалку экземпляр прохода
        // чтобы вызвать метод submit_frame который работает с &mut self
        // BackendEncoder, поэтому нужно сделать дроп чтобы освободить
        // заимствование, pass.draw вызывается выше
        drop(pass);

        encoder.submit_frame(context);

        Ok(())
    }

    #[cfg(not(feature = "modern"))]
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
