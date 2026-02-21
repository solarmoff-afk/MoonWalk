// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use moonwalk_backend::core::context::BackendContext;
use moonwalk_backend::pipeline::bind::RawBindGroupLayout;

use moonwalk_backend::pipeline::{
    BackendPipeline,
    PipelineResult,
    types::{
        BlendMode,
        StepMode
    }
};
use moonwalk_backend::pipeline::vertex::VertexLayout;

use crate::error::MoonWalkError;

pub fn create_blur_pipeline(
    context: &mut BackendContext,
    uniform_layout: &RawBindGroupLayout,
    texture_layout: &RawBindGroupLayout,
) -> Result<PipelineResult, MoonWalkError> {
    let format = context.get_format();
    let result = BackendPipeline::new(include_str!("shaders/blur.wgsl"))
        .vertex_shader("vs_main")
        .fragment_shader("fs_main")
        .add_vertex_layout(VertexLayout::new()
        .stride(0)
        .step_mode(StepMode::Vertex))
        .blend(BlendMode::None)
        .label("blur_filter")
        .build(context, format, &[uniform_layout, texture_layout])?;

    Ok(result)
}

pub fn create_color_pipeline(
    context: &mut BackendContext,
    uniform_layout: &RawBindGroupLayout,
    texture_layout: &RawBindGroupLayout,
) -> Result<PipelineResult, MoonWalkError> {
    let format = context.get_format();
    let result = BackendPipeline::new(include_str!("shaders/color_matrix.wgsl"))
        .vertex_shader("vs_main")
        .fragment_shader("fs_main")
        .add_vertex_layout(VertexLayout::new()
        .stride(0)
        .step_mode(StepMode::Vertex))
        .blend(BlendMode::None)
        .label("color_filter")
        .build(context, format, &[uniform_layout, texture_layout])?;

    Ok(result)
}

pub fn create_advanced_pipeline(
    context: &mut BackendContext,
    uniform_layout: &RawBindGroupLayout,
    texture_layout: &RawBindGroupLayout,
) -> Result<PipelineResult, MoonWalkError> {
    let format = context.get_format();
    let result = BackendPipeline::new(include_str!("shaders/advanced.wgsl"))
        .vertex_shader("vs_main")
        .fragment_shader("fs_main")
        .add_vertex_layout(VertexLayout::new()
        .stride(0)
        .step_mode(StepMode::Vertex))
        .blend(BlendMode::None)
        .label("advanced_filter")
        .build(context, format, &[uniform_layout, texture_layout])?;

    Ok(result)
}

pub fn create_liquid_glass_pipeline(
    context: &mut BackendContext,
    uniform_layout: &RawBindGroupLayout,
    texture_layout: &RawBindGroupLayout,
) -> Result<PipelineResult, MoonWalkError> {
    let format = context.get_format();
    let result = BackendPipeline::new(include_str!("shaders/liquid_glass.wgsl"))
        .vertex_shader("vs_main")
        .fragment_shader("fs_main")
        .add_vertex_layout(VertexLayout::new()
        .stride(0)
        .step_mode(StepMode::Vertex))
        .blend(BlendMode::None)
        .label("liquid_glass_filter")
        .build(context, format, &[uniform_layout, texture_layout])?;

    Ok(result)
}

pub fn create_mesh_gradient_pipeline(
    context: &mut BackendContext,
    uniform_layout: &RawBindGroupLayout,
    texture_layout: &RawBindGroupLayout,
) -> Result<PipelineResult, MoonWalkError> {
    let format = context.get_format();
    let result = BackendPipeline::new(include_str!("shaders/mesh_gradient.wgsl"))
        .vertex_shader("vs_main")
        .fragment_shader("fs_main")
        .add_vertex_layout(VertexLayout::new()
        .stride(0)
        .step_mode(StepMode::Vertex))
        .blend(BlendMode::None)
        .label("mesh_gradient_filter")
        .build(context, format, &[uniform_layout, texture_layout])?;

    Ok(result)
}
