// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use wgpu::util::DeviceExt;
use bytemuck::{Pod, Zeroable};
use glam::Vec4;

use crate::gpu::context::Context;
use crate::gpu::MatrixStack;
use crate::rendering::texture::Texture;
use crate::r#abstract::*;
use crate::error::MoonWalkError;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct BrushUniform {
    view_proj: [[f32; 4]; 4],
    color: [f32; 4],
    params: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct BrushVertex {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub rotation: f32,
    pub opacity: f32,
}

pub struct PaintingSystem {
    pipeline: wgpu::RenderPipeline,
    eraser_pipeline: wgpu::RenderPipeline,
    uniform_layout: wgpu::BindGroupLayout,
    texture_layout: wgpu::BindGroupLayout,
    default_brush_texture: Texture,
}

impl PaintingSystem {
    pub fn new(ctx: &Context) -> Result<Self, MoonWalkError> {
        let shader_source = include_str!("brush.wgsl");

        let uniform_layout = BindGroup::new()
            .add_uniform(0, ShaderStage::Both)
            .build(ctx)?;

        let texture_layout = BindGroup::new()
            .add_texture(0, TextureType::Float)
            .add_sampler(1, SamplerType::Linear)
            .build(ctx)?;

        let pipeline = MoonPipeline::new(shader_source)
            .vertex_shader("vs_main")
            .fragment_shader("fs_main")
            .add_vertex_layout(
                VertexLayout::new()
                    .stride(24) // 2xFloat + 2xFloat + Float + Float = 24 байта
                    .step_mode(StepMode::Instance)
                    .add_attr(VertexAttr::new().format(Format::Float32x2).location(0).offset(0))
                    .add_attr(VertexAttr::new().format(Format::Float32x2).location(1).offset(8))
                    .add_attr(VertexAttr::new().format(Format::Float32).location(2).offset(16))
                    .add_attr(VertexAttr::new().format(Format::Float32).location(3).offset(20))
            )
            .blend(BlendMode::Alpha)
            .label("brush_pipeline")
            .build(ctx, wgpu::TextureFormat::Rgba8UnormSrgb, &[&uniform_layout, &texture_layout])?;

        // Отедльный пайплайн с другим блендмод для ластика
        let eraser_pipeline = MoonPipeline::new(shader_source)
            .vertex_shader("vs_main")
            .fragment_shader("fs_main")
            .add_vertex_layout(
                VertexLayout::new()
                    .stride(24)
                    .step_mode(StepMode::Instance)
                    .add_attr(VertexAttr::new().format(Format::Float32x2).location(0).offset(0))
                    .add_attr(VertexAttr::new().format(Format::Float32x2).location(1).offset(8))
                    .add_attr(VertexAttr::new().format(Format::Float32).location(2).offset(16))
                    .add_attr(VertexAttr::new().format(Format::Float32).location(3).offset(20))
            )
            .blend(BlendMode::Eraser)
            .label("eraser_pipeline")
            .build(ctx, wgpu::TextureFormat::Rgba8UnormSrgb, &[&uniform_layout, &texture_layout])?;

        let white_pixels = vec![255; 4 * 16 * 16];
        let default_brush = Texture::from_raw(ctx, &white_pixels, 16, 16, "Default Brush Tip")?;

        Ok(Self {
            pipeline: pipeline.pipeline.raw,
            eraser_pipeline: eraser_pipeline.pipeline.raw,
            uniform_layout,
            texture_layout,
            default_brush_texture: default_brush,
        })
    }

    pub fn draw_strokes(
        &mut self,
        ctx: &Context,
        target: &Texture,
        brush_tip: Option<&Texture>,
        instances: &[BrushVertex],
        color: Vec4,
        hardness: f32,
        is_eraser: bool,
    ) {
        if instances.is_empty() {
            return;
        }

        let width = target.texture.width();
        let height = target.texture.height();

        let mut matrix_stack = MatrixStack::new();
        matrix_stack.set_ortho(width as f32, height as f32);

        let uniform_data = BrushUniform {
            view_proj: matrix_stack.projection.to_cols_array_2d(),
            color: color.to_array(),
            params: [hardness, 0.0, 0.0, 0.0],
        };

        let uniform_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Brush uniforms"),
            contents: bytemuck::bytes_of(&uniform_data),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let uniform_bg = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.uniform_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: None,
        });

        let tip = brush_tip.unwrap_or(&self.default_brush_texture);
        let texture_bg = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&tip.view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&tip.sampler) },
            ],
            label: None,
        });

        let instance_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Brush instances"),
            contents: bytemuck::cast_slice(instances),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let mut encoder = ctx.create_encoder();
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Brush pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, 
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Просто кисть или ластик? Загадка века
            if is_eraser {
                pass.set_pipeline(&self.eraser_pipeline);
            } else {
                pass.set_pipeline(&self.pipeline);
            }

            pass.set_bind_group(0, &uniform_bg, &[]);
            pass.set_bind_group(1, &texture_bg, &[]);
            pass.set_vertex_buffer(0, instance_buffer.slice(..));
            
            pass.draw(0..6, 0..instances.len() as u32);
        }

        ctx.submit(encoder);
    }
}
