// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

pub mod color_matrix;

use wgpu::util::DeviceExt;
use bytemuck::{Pod, Zeroable};

use crate::gpu::context::Context;
use crate::gpu::Buffer;
use crate::rendering::texture::Texture;
use crate::r#abstract::*;
use crate::error::MoonWalkError;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct DummyVertex {
    _dummy: f32,
}


#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct BlurUniform {
    direction: [f32; 2],
    radius: f32,
    _pad: f32,
    resolution: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct ColorMatrixUniform {
    matrix: [[f32; 4]; 4],
    offset: [f32; 4],
}

pub struct FilterSystem {
    swap_texture: Option<Texture>,
    
    blur_pipeline: wgpu::RenderPipeline,
    color_pipeline: wgpu::RenderPipeline,
    
    uniform_layout: wgpu::BindGroupLayout,
    texture_layout: wgpu::BindGroupLayout,
    dummy_vbo: Buffer<DummyVertex>,
}

impl FilterSystem {
    pub fn new(ctx: &Context) -> Result<Self, MoonWalkError> {
        let dummy_vertices = [DummyVertex {
            _dummy: 0.0
        }];

        let dummy_vbo = Buffer::vertex(ctx, &dummy_vertices);

        let uniform_layout = BindGroup::new()
            .add_uniform(0, ShaderStage::Fragment)
            .build(ctx)?;

        let texture_layout = BindGroup::new()
            .add_texture(0, TextureType::Float)
            .add_sampler(1, SamplerType::Linear)
            .build(ctx)?;

        let blur_pipeline = MoonPipeline::new(include_str!("shaders/blur.wgsl"))
            .vertex_shader("vs_main")
            .fragment_shader("fs_main")
            .add_vertex_layout(
                VertexLayout::new()
                    .stride(0)
                    .step_mode(StepMode::Vertex)
            )
            .add_bind_group(
                BindGroup::new()
                    .add_uniform(0, ShaderStage::Fragment)
            ) 
            .add_bind_group(
                BindGroup::new()
                    .add_texture(0, TextureType::Float)
                    .add_sampler(1, SamplerType::Linear)
            )
            .blend(BlendMode::None)
            .cull(CullMode::None)
            .topology(Topology::TriangleList)
            .depth_test(false)
            .depth_write(false)
            .label("blur_filter")
            .build(ctx, wgpu::TextureFormat::Rgba8UnormSrgb, &[])?;

        let color_pipeline = MoonPipeline::new(include_str!("shaders/color_matrix.wgsl"))
            .vertex_shader("vs_main")
            .fragment_shader("fs_main")
            .add_vertex_layout(
                VertexLayout::new()
                    .stride(0)
                    .step_mode(StepMode::Vertex)
            )
            .add_bind_group(
                BindGroup::new()
                    .add_uniform(0, ShaderStage::Fragment)
            )
            .add_bind_group(
                BindGroup::new()
                    .add_texture(0, TextureType::Float)
                    .add_sampler(1, SamplerType::Linear)
            )
            .blend(BlendMode::None)
            .cull(CullMode::None)
            .topology(Topology::TriangleList)
            .depth_test(false)
            .depth_write(false)
            .label("color_filter")
            .build(ctx, wgpu::TextureFormat::Rgba8UnormSrgb, &[])?;

        Ok(Self {
            swap_texture: None,
            blur_pipeline: blur_pipeline.pipeline.raw,
            color_pipeline: color_pipeline.pipeline.raw,
            uniform_layout,
            texture_layout,
            dummy_vbo,
        })
    }

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
            bytemuck::bytes_of(&uniform_data)
        );

        let mut encoder = ctx.create_encoder();
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &swap.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },

            wgpu::TexelCopyTextureInfo {
                texture: &target_texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },

            wgpu::Extent3d {
                width, height,
                depth_or_array_layers: 1
            }
        );

        ctx.submit(encoder);
    }

    fn ensure_swap_texture(&mut self, ctx: &Context, w: u32, h: u32, format: wgpu::TextureFormat) {
        let need_create = self.swap_texture.as_ref()
            .map_or(true, |t| t.texture.width() != w || t.texture.height() != h);

        if need_create {
            self.swap_texture = Some(Texture::create_render_target(ctx, w, h, format));
        }
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
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let texture_bg = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Filter Texture BG"),
            layout: &self.texture_layout,

            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&source.view),
                },

                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&source.sampler),
                },
            ],
        });

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
            pass.set_bind_group(0, &uniform_bg, &[]);
            pass.set_bind_group(1, &texture_bg, &[]);

            // [HACK]
            // Взятие слайса напрямую из raw буфера
            pass.set_vertex_buffer(0, self.dummy_vbo.raw.slice(..));

            pass.draw(0..3, 0..1);
        }
        
        ctx.submit(encoder);
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

        let uniform_data = ColorMatrixUniform {
            matrix,
            offset,
        };

        self.execute_pass(
            ctx,
            &self.color_pipeline,
            target_texture,
            swap,
            bytemuck::bytes_of(&uniform_data)
        );

        let mut encoder = ctx.create_encoder();
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &swap.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },

            wgpu::TexelCopyTextureInfo {
                texture: &target_texture.texture,
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
}
