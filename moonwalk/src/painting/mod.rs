// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use bytemuck::{Pod, Zeroable};
use glam::Vec4;

#[cfg(feature = "modern")]
use moonwalk_backend::core::context::BackendContext;

#[cfg(feature = "modern")]
use moonwalk_backend::error::MoonBackendError;

#[cfg(feature = "modern")]
use moonwalk_backend::pipeline::RawPipeline;

#[cfg(feature = "modern")]
use moonwalk_backend::pipeline::vertex::VertexAttr;

use moonwalk_backend::pipeline::bind::{RawBindGroupLayout, BindGroup};

#[cfg(feature = "modern")]
use moonwalk_backend::pipeline::types::{
    BlendMode, ShaderStage, TextureType, SamplerType, Format, StepMode
};

#[cfg(feature = "modern")]
use moonwalk_backend::render::texture::BackendTexture;

use std::collections::HashMap;

#[cfg(not(feature = "modern"))]
use crate::gpu::context::Context;

use crate::gpu::MatrixStack;

#[cfg(not(feature = "modern"))]
use crate::rendering::texture::Texture;

#[cfg(not(feature = "modern"))]
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

#[cfg(feature = "modern")]
pub struct PaintingSystem {
    pipelines: HashMap<BlendMode, RawPipeline>,
    uniform_layout: RawBindGroupLayout,
    texture_layout: RawBindGroupLayout,
    default_brush_texture: BackendTexture,
}

#[cfg(not(feature = "modern"))]
pub struct PaintingSystem {
    pipelines: HashMap<BlendMode, wgpu::RenderPipeline>,
    uniform_layout: wgpu::BindGroupLayout,
    texture_layout: wgpu::BindGroupLayout,
    default_brush_texture: Texture,
}

impl PaintingSystem {
    #[cfg(feature = "modern")]
    pub fn new(context: &mut BackendContext) -> Result<Self, MoonWalkError> {
        let shader_source = include_str!("brush.wgsl");

        let uniform_layout = BindGroup::new()
            .add_uniform(0, ShaderStage::Both)
            .build(context)?;

        let texture_layout = BindGroup::new()
            .add_texture(0, TextureType::Float)
            .add_sampler(1, SamplerType::Linear)
            .build(context)?;

        let mut pipelines = HashMap::new();

        let create_pipeline = |mode: BlendMode| -> Result<RawPipeline, MoonWalkError> {
            let actual_format = context.get_format();
            let p = BackendPipeline::new(shader_source)
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
                .blend(mode)
                .label(&format!("brush_pipeline_{:?}", mode))
                .build(context, actual_format, &[&uniform_layout, &texture_layout])?;
            Ok(p.pipeline.raw)
        };

        pipelines.insert(BlendMode::Alpha, create_pipeline(BlendMode::Alpha)?);
        pipelines.insert(BlendMode::Eraser, create_pipeline(BlendMode::Eraser)?);
        pipelines.insert(BlendMode::Additive, create_pipeline(BlendMode::Additive)?);
        pipelines.insert(BlendMode::Multiply, create_pipeline(BlendMode::Multiply)?);
        pipelines.insert(BlendMode::Screen, create_pipeline(BlendMode::Screen)?);

        let mut default_brush = BackendTexture::new(16, 16);

        let white_pixels = vec![255; 4 * 16 * 16];
        default_brush.from_raw(context, &white_pixels, 16, 16)?;

        Ok(Self {
            pipelines,
            uniform_layout,
            texture_layout,
            default_brush_texture: default_brush,
        })
    }

    #[cfg(not(feature = "modern"))]
    pub fn new(ctx: &Context) -> Result<Self, MoonWalkError> {
        let shader_source = include_str!("brush.wgsl");

        let uniform_layout = BindGroup::new()
            .add_uniform(0, ShaderStage::Both)
            .build(ctx)?;

        let texture_layout = BindGroup::new()
            .add_texture(0, TextureType::Float)
            .add_sampler(1, SamplerType::Linear)
            .build(ctx)?;

        let mut pipelines = HashMap::new();

        let create_pipe = |mode: BlendMode| -> Result<wgpu::RenderPipeline, MoonWalkError> {
            let actual_format = ctx.config.format;
            let p = MoonPipeline::new(shader_source)
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
                .blend(mode)
                .label(&format!("brush_pipeline_{:?}", mode))
                .build(ctx, actual_format, &[&uniform_layout, &texture_layout])?;
            Ok(p.pipeline.raw)
        };

        pipelines.insert(BlendMode::Alpha, create_pipe(BlendMode::Alpha)?);
        pipelines.insert(BlendMode::Eraser, create_pipe(BlendMode::Eraser)?);
        pipelines.insert(BlendMode::Additive, create_pipe(BlendMode::Additive)?);
        pipelines.insert(BlendMode::Multiply, create_pipe(BlendMode::Multiply)?);
        pipelines.insert(BlendMode::Screen, create_pipe(BlendMode::Screen)?);

        let white_pixels = vec![255; 4 * 16 * 16];
        let default_brush = Texture::from_raw(ctx, &white_pixels, 16, 16, "Default brush tip")?;

        Ok(Self {
            pipelines,
            uniform_layout,
            texture_layout,
            default_brush_texture: default_brush,
        })
    }

    #[cfg(feature = "modern")]
    pub fn draw_strokes(
        &mut self,
        context: &mut BackendContext,
        target: &BackendTexture,
        brush_tip: Option<&BackendTexture>,
        instances: &[BrushVertex],
        color: Vec4,
        hardness: f32,
        blend_mode: BlendMode,
    ) -> Result<(), MoonWalkError> {
        use moonwalk_backend::{core::{buffer::BackendBuffer, encoder::BackendEncoder}, render::pass::RenderPass};

        if instances.is_empty() {
            return Ok(());
        }

        let width = target.width;
        let height = target.height;

        let mut matrix_stack = MatrixStack::new();
        matrix_stack.set_ortho(width as f32, height as f32);

        let uniform_data = BrushUniform {
            view_proj: matrix_stack.projection.to_cols_array_2d(),
            color: color.to_array(),
            params: [hardness, 0.0, 0.0, 0.0],
        };

        let uniform_buffer = BackendBuffer::uniform_bytes(context, bytemuck::bytes_of(&uniform_data))?;

        let uniform_bg = BindGroup::create_uniform_bind_group(
            &self.uniform_layout,
            context,
            &uniform_buffer,
            Some("Filter Uniform BG")
        )?;

        let tip = brush_tip.unwrap_or(&self.default_brush_texture);

        let texture_bg = BindGroup::create_texture_bind_group(
            &self.texture_layout,
            context,
            &[(tip, 0)],
            &[(tip, 1)],
            Some("Filter Texture BG")
        )?;

        let instance_buffer = BackendBuffer::instance(
            context, bytemuck::cast_slice(instances)
        )?;

        let mut encoder = BackendEncoder::new(context, "MoonWalk painting encoder")?;

        let mut pass = RenderPass::new(
            &mut encoder, texture, None, "MoonWalk brush render pass".to_string()
        )?;

        // [HACK]
        // Нужно удалить unwrap и заменить на полноценный Result
        let pipeline = self.pipelines.get(&blend_mode)
            .or_else(|| self.pipelines.get(&BlendMode::Alpha))
            .expect("HACK: Remove me please");

        pass.set_pipeline(pipeline);

        pass.set_bind_group(0, &uniform_bg);
        pass.set_bind_group(1, &texture_bg);
        pass.set_vertex_buffer(0, &instance_buffer);
        pass.draw(instances.len() as u32);

        encoder.submit_frame(context)?;

        Ok(())
    }

    #[cfg(not(feature = "modern"))]
    pub fn draw_strokes(
        &mut self,
        ctx: &Context,
        target: &Texture,
        brush_tip: Option<&Texture>,
        instances: &[BrushVertex],
        color: Vec4,
        hardness: f32,
        blend_mode: BlendMode,
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

            let pipeline = self.pipelines.get(&blend_mode)
                .or_else(|| self.pipelines.get(&BlendMode::Alpha))
                .unwrap();

            pass.set_pipeline(pipeline);

            pass.set_bind_group(0, &uniform_bg, &[]);
            pass.set_bind_group(1, &texture_bg, &[]);
            pass.set_vertex_buffer(0, instance_buffer.slice(..));
            
            pass.draw(0..6, 0..instances.len() as u32);
        }

        ctx.submit(encoder);
    }
}