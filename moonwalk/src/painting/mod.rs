// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use bytemuck::{Pod, Zeroable};
use glam::Vec4;

use moonwalk_backend::core::context::BackendContext;
use moonwalk_backend::error::MoonBackendError;
use moonwalk_backend::pipeline::{RawPipeline, BackendPipeline};
use moonwalk_backend::pipeline::vertex::{VertexAttr, VertexLayout};
use moonwalk_backend::pipeline::bind::{RawBindGroupLayout, BindGroup};
use moonwalk_backend::pipeline::types::{
    BlendMode, ShaderStage, TextureType, SamplerType, Format, StepMode
};
use moonwalk_backend::render::texture::BackendTexture;
use std::collections::HashMap;

use crate::gpu::MatrixStack;
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
    pipelines: HashMap<BlendMode, RawPipeline>,
    uniform_layout: RawBindGroupLayout,
    texture_layout: RawBindGroupLayout,
    default_brush_texture: BackendTexture,
}

impl PaintingSystem {
    pub fn new(context: &mut BackendContext) -> Result<Self, MoonWalkError> {
        let shader_source = include_str!("../shaders/brush.wgsl");

        let uniform_layout = BindGroup::new()
            .add_uniform(0, ShaderStage::Both)
            .build(context)?;

        let texture_layout = BindGroup::new()
            .add_texture(0, TextureType::Float)
            .add_sampler(1, SamplerType::Linear)
            .build(context)?;

        let mut pipelines = HashMap::new();

        let mut create_pipeline = |mode: BlendMode| -> Result<RawPipeline, MoonWalkError> {
            let actual_format = context.get_format();
            let pipeline = BackendPipeline::new(shader_source)
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
            
            match pipeline.pipeline {
                Some(raw) => Ok(raw),
                None => Err(MoonWalkError::ShaderError("Failed to create pipeline".to_string())),
            }
        };

        pipelines.insert(BlendMode::Alpha, create_pipeline(BlendMode::Alpha)?);
        pipelines.insert(BlendMode::Eraser, create_pipeline(BlendMode::Eraser)?);
        pipelines.insert(BlendMode::Additive, create_pipeline(BlendMode::Additive)?);
        pipelines.insert(BlendMode::Multiply, create_pipeline(BlendMode::Multiply)?);
        pipelines.insert(BlendMode::Screen, create_pipeline(BlendMode::Screen)?);

        let mut default_brush = BackendTexture::new(16, 16);
        default_brush.config.set_format(context.get_format());

        let white_pixels = vec![255; 4 * 16 * 16];
        default_brush.from_raw(context, &white_pixels, 16, 16)?;

        Ok(Self {
            pipelines,
            uniform_layout,
            texture_layout,
            default_brush_texture: default_brush,
        })
    }

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

        let uniform_buffer = BackendBuffer::<u8>::uniform_bytes(context, bytemuck::bytes_of(&uniform_data))?;

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

        let instance_buffer = BackendBuffer::<BrushVertex>::instance(
            context, 
            bytemuck::cast_slice(instances)
        )?;

        let mut encoder = BackendEncoder::new(context, "MoonWalk painting encoder")?;

        let mut pass = RenderPass::new(
            &mut encoder, target, None, "MoonWalk brush render pass"
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

        // После pass.draw RenderPass уже не нужен, чтобы боров чекер не бил ошибку
        // из encoder.submit_frame нужно дропнуть рендер пасс вручную
        drop(pass);

        encoder.submit_frame(context)?;

        Ok(())
    }
}