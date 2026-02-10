// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

#[cfg(feature = "modern")]
use moonwalk_backend::core::context::BackendContext;

#[cfg(feature = "modern")]
use moonwalk_backend::pipeline::bind::{BindGroup, RawBindGroupLayout};

#[cfg(feature = "modern")]
use moonwalk_backend::pipeline::{BackendPipeline, PipelineResult, types::{BlendMode, StepMode}};

#[cfg(feature = "modern")]
use moonwalk_backend::pipeline::vertex::VertexLayout;

#[cfg(feature = "modern")]
use moonwalk_backend::render::texture::BackendTextureFormat;

#[cfg(not(feature = "modern"))]
use crate::gpu::context::Context;

use crate::r#abstract::*;
use crate::error::MoonWalkError;

#[cfg(feature = "modern")]
pub fn create_blur_pipeline(
    context: &mut BackendContext,
    uniform_layout: &RawBindGroupLayout,
    texture_layout: &RawBindGroupLayout,
) -> Result<PipelineResult, MoonWalkError> {
    let result = BackendPipeline::new(include_str!("shaders/blur.wgsl"))
        .vertex_shader("vs_main")
        .fragment_shader("fs_main")
        .add_vertex_layout(VertexLayout::new()
        .stride(0)
        .step_mode(StepMode::Vertex))
        .blend(BlendMode::None)
        .label("blur_filter")
        .build(context, BackendTextureFormat::Rgba8UnormSrgb, &[uniform_layout, texture_layout])?;

    Ok(result)
}

#[cfg(not(feature = "modern"))]
pub fn create_blur_pipeline(
    ctx: &Context,
    uniform_layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
) -> Result<wgpu::RenderPipeline, MoonWalkError> {
    let actual_format = ctx.config.format;
    let result = MoonPipeline::new(include_str!("shaders/blur.wgsl"))
        .vertex_shader("vs_main")
        .fragment_shader("fs_main")
        .add_vertex_layout(VertexLayout::new().stride(0).step_mode(StepMode::Vertex))
        .blend(BlendMode::None)
        .label("blur_filter")
        .build(ctx, actual_format, &[uniform_layout, texture_layout])?;

    Ok(result.pipeline.raw)
}

#[cfg(feature = "modern")]
pub fn create_color_pipeline(
    context: &mut BackendContext,
    uniform_layout: &RawBindGroupLayout,
    texture_layout: &RawBindGroupLayout,
) -> Result<PipelineResult, MoonWalkError> {
    let result = BackendPipeline::new(include_str!("shaders/color_matrix.wgsl"))
        .vertex_shader("vs_main")
        .fragment_shader("fs_main")
        .add_vertex_layout(VertexLayout::new()
        .stride(0)
        .step_mode(StepMode::Vertex))
        .blend(BlendMode::None)
        .label("color_filter")
        .build(context, BackendTextureFormat::Rgba8UnormSrgb, &[uniform_layout, texture_layout])?;

    Ok(result)
}

#[cfg(not(feature = "modern"))]
pub fn create_color_pipeline(
    ctx: &Context,
    uniform_layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
) -> Result<wgpu::RenderPipeline, MoonWalkError> {
    let actual_format = ctx.config.format;
    let result = MoonPipeline::new(include_str!("shaders/color_matrix.wgsl"))
        .vertex_shader("vs_main")
        .fragment_shader("fs_main")
        .add_vertex_layout(VertexLayout::new().stride(0).step_mode(StepMode::Vertex))
        .blend(BlendMode::None)
        .label("color_filter")
        .build(ctx, actual_format, &[uniform_layout, texture_layout])?;

    Ok(result.pipeline.raw)
}

#[cfg(feature = "modern")]
pub fn create_advanced_pipeline(
    context: &mut BackendContext,
    uniform_layout: &RawBindGroupLayout,
    texture_layout: &RawBindGroupLayout,
) -> Result<PipelineResult, MoonWalkError> {
    let result = BackendPipeline::new(include_str!("shaders/advanced.wgsl"))
        .vertex_shader("vs_main")
        .fragment_shader("fs_main")
        .add_vertex_layout(VertexLayout::new()
        .stride(0)
        .step_mode(StepMode::Vertex))
        .blend(BlendMode::None)
        .label("advanced_filter")
        .build(context, BackendTextureFormat::Rgba8UnormSrgb, &[uniform_layout, texture_layout])?;

    Ok(result)
}

#[cfg(not(feature = "modern"))]
pub fn create_advanced_pipeline(
    ctx: &Context,
    uniform_layout: &wgpu::BindGroupLayout,
    advanced_layout: &wgpu::BindGroupLayout,
) -> Result<wgpu::RenderPipeline, MoonWalkError> {
    let actual_format = ctx.config.format;
    let result = MoonPipeline::new(include_str!("shaders/advanced.wgsl"))
        .vertex_shader("vs_main")
        .fragment_shader("fs_main")
        .add_vertex_layout(VertexLayout::new().stride(0).step_mode(StepMode::Vertex))
        .blend(BlendMode::None)
        .label("advanced_filter")
        .build(ctx, actual_format, &[uniform_layout, advanced_layout])?;

    Ok(result.pipeline.raw)
}
