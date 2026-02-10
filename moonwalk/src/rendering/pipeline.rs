// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use std::collections::HashMap;

#[cfg(feature = "modern")]
use moonwalk_backend::core::buffer::BackendBuffer;

#[cfg(feature = "modern")]
use moonwalk_backend::core::context::BackendContext;

#[cfg(feature = "modern")]
use moonwalk_backend::pipeline::RawPipeline;

#[cfg(feature = "modern")]
use moonwalk_backend::pipeline::bind::{RawBindGroup, BindGroup, RawBindGroupLayout};

#[cfg(feature = "modern")]
use moonwalk_backend::pipeline::types::ShaderStage;

#[cfg(feature = "modern")]
use moonwalk_backend::pipeline::{
    BackendPipeline, FallbackStrategy,
    types::{BlendMode, CullMode, Format, SamplerType, StepMode, TextureType, Topology},
    vertex::{VertexAttr, VertexLayout, create_rect_instance_layout}
};

#[cfg(feature = "modern")]
use moonwalk_backend::render::texture::BackendTextureFormat;

#[cfg(not(feature = "modern"))]
use crate::gpu::Context;

use crate::objects::ShaderId;
use crate::error::MoonWalkError;
use crate::r#abstract::*;

#[cfg(feature = "modern")]
pub struct ShaderStore<'a> {
    pipelines: HashMap<ShaderId, &'a RawPipeline>,
    proj_bind_group: Option<RawBindGroup>,
    proj_layout: RawBindGroupLayout,
}

#[cfg(not(feature = "modern"))]
pub struct ShaderStore {
    pipelines: HashMap<ShaderId, crate::gpu::Pipeline>,
    proj_bind_group: Option<wgpu::BindGroup>,
    proj_layout: wgpu::BindGroupLayout,
}

impl ShaderStore<'_> {
     #[cfg(feature = "modern")]
    pub fn new(context: &mut BackendContext) -> Result<Self, MoonWalkError> {
        let proj_layout = BindGroup::new()
            .add_uniform(0, ShaderStage::Vertex)
            .build(context)?;
        
        Ok(Self {
            pipelines: HashMap::new(),
            proj_bind_group: None,
            proj_layout,
        })
    }

    #[cfg(not(feature = "modern"))]
    pub fn new(ctx: &Context) -> Result<Self, MoonWalkError> {
        let proj_layout = BindGroup::new()
            .add_uniform(0, ShaderStage::Vertex)
            .build(ctx)?;
        
        Ok(Self {
            pipelines: HashMap::new(),
            proj_bind_group: None,
            proj_layout,
        })
    }

    #[cfg(feature = "modern")]
    pub fn create_default_rect(
        &mut self,
        context: &mut BackendContext,
        format: BackendTextureFormat
    ) -> Result<ShaderId, MoonWalkError> {
        let shader_source = include_str!("../shaders/shape.wgsl");
        
        let texture_layout = BindGroup::new()
            .add_texture(0, TextureType::Float)
            .add_sampler(1, SamplerType::Linear)
            .build(context)?;
        
        let pipeline = BackendPipeline::new(shader_source)
            .vertex_shader("vs_main")
            .fragment_shader("fs_main")
            .add_vertex_layout(
                VertexLayout::new()
                    .stride(8)
                    .step_mode(StepMode::Vertex)
                    .add_attr(
                        VertexAttr::new()
                            .format(Format::Float32x2)
                            .location(0)
                            .offset(0)
                    )
            )
            .add_vertex_layout(create_rect_instance_layout())
            .add_bind_group(
                BindGroup::new()
                    .add_uniform(0, ShaderStage::Vertex)
            )
            .blend(BlendMode::Alpha)
            .cull(CullMode::None)
            .topology(Topology::TriangleList)
            .depth_test(false)
            .depth_write(false)
            .fallback_strategy(FallbackStrategy::Adaptive)
            .label("default_rect")
            .build(context, format, &[&texture_layout])?;
        
        let id = ShaderId(1);
        self.pipelines.insert(id, pipeline.get_raw()?);
        
        Ok(id)
    }

    #[cfg(not(feature = "modern"))]
    pub fn create_default_rect(&mut self, ctx: &Context, format: wgpu::TextureFormat) -> Result<ShaderId, MoonWalkError> {
        let shader_source = include_str!("../shaders/shape.wgsl");
        
        let texture_layout = BindGroup::new()
            .add_texture(0, TextureType::Float)
            .add_sampler(1, SamplerType::Linear)
            .build(ctx)?;
        
        let pipeline = MoonPipeline::new(shader_source)
            .vertex_shader("vs_main")
            .fragment_shader("fs_main")
            .add_vertex_layout(
                VertexLayout::new()
                    .stride(8)
                    .step_mode(StepMode::Vertex)
                    .add_attr(
                        VertexAttr::new()
                            .format(Format::Float32x2)
                            .location(0)
                            .offset(0)
                    )
            )
            .add_vertex_layout(MoonPipeline::create_rect_instance_layout())
            .add_bind_group(
                BindGroup::new()
                    .add_uniform(0, ShaderStage::Vertex)
            )
            .blend(BlendMode::Alpha)
            .cull(CullMode::None)
            .topology(Topology::TriangleList)
            .depth_test(false)
            .depth_write(false)
            .fallback_strategy(FallbackStrategy::Adaptive)
            .label("default_rect")
            .build(ctx, format, &[&texture_layout])?;
        
        let id = ShaderId(1);
        self.pipelines.insert(id, pipeline.pipeline);
        
        Ok(id)
    }

    #[cfg(feature = "modern")]
    pub fn compile_shader(
        &mut self, 
        context: &mut BackendContext,
        src: &str,
    ) -> Result<ShaderId, MoonWalkError> {
        let actual_format = context.get_format();

        let pipeline = BackendPipeline::new(src)
            .add_vertex_layout(
                VertexLayout::new()
                    .stride(60)
                    .step_mode(StepMode::Vertex)
                    .add_attr(
                        VertexAttr::new()
                            .format(Format::Float32x3)
                            .location(0)
                            .offset(0)
                    )
                    .add_attr(
                        VertexAttr::new()
                            .format(Format::Float32x4)
                            .location(1)
                            .offset(12)
                    )
                    .add_attr(
                        VertexAttr::new()
                            .format(Format::Float32x2)
                            .location(2)
                            .offset(28)
                    )
                    .add_attr(
                        VertexAttr::new()
                            .format(Format::Float32x2)
                            .location(3)
                            .offset(36)
                    )
                    .add_attr(
                        VertexAttr::new()
                            .format(Format::Float32x4)
                            .location(4)
                            .offset(44)
                    )
            )
            .add_bind_group(
                BindGroup::new()
                    .add_uniform(0, ShaderStage::Vertex)
            )
            .blend(BlendMode::Alpha)
            .cull(CullMode::Back)
            .topology(Topology::TriangleList)
            .depth_test(true)
            .depth_write(true)
            .fallback_strategy(FallbackStrategy::None)
            .label("custom_shader")
            .build(context, actual_format, &[&self.proj_layout])?;
        
        let id = ShaderId(self.pipelines.len() as u32 + 100);
        self.pipelines.insert(id, pipeline.get_raw()?);
        
        Ok(id)
    }

    #[cfg(not(feature = "modern"))]
    pub fn compile_shader(
        &mut self, 
        ctx: &Context, 
        src: &str, 
        _format: wgpu::TextureFormat
    ) -> Result<ShaderId, MoonWalkError> {
        let actual_format = ctx.config.format;

        let pipeline = MoonPipeline::new(src)
            .add_vertex_layout(
                VertexLayout::new()
                    .stride(60)
                    .step_mode(StepMode::Vertex)
                    .add_attr(
                        VertexAttr::new()
                            .format(Format::Float32x3)
                            .location(0)
                            .offset(0)
                    )
                    .add_attr(
                        VertexAttr::new()
                            .format(Format::Float32x4)
                            .location(1)
                            .offset(12)
                    )
                    .add_attr(
                        VertexAttr::new()
                            .format(Format::Float32x2)
                            .location(2)
                            .offset(28)
                    )
                    .add_attr(
                        VertexAttr::new()
                            .format(Format::Float32x2)
                            .location(3)
                            .offset(36)
                    )
                    .add_attr(
                        VertexAttr::new()
                            .format(Format::Float32x4)
                            .location(4)
                            .offset(44)
                    )
            )
            .add_bind_group(
                BindGroup::new()
                    .add_uniform(0, ShaderStage::Vertex)
            )
            .blend(BlendMode::Alpha)
            .cull(CullMode::Back)
            .topology(Topology::TriangleList)
            .depth_test(true)
            .depth_write(true)
            .fallback_strategy(FallbackStrategy::None)
            .label("custom_shader")
            .build(ctx, actual_format, &[&self.proj_layout])?;
        
        let id = ShaderId(self.pipelines.len() as u32 + 100);
        self.pipelines.insert(id, pipeline.pipeline);
        
        Ok(id)
    }

    #[cfg(feature = "modern")]
    pub fn get_pipeline(&self, id: ShaderId) -> Option<&RawPipeline> {
        self.pipelines.get(&id).map(|p| *p)
    }

    #[cfg(not(feature = "modern"))]
    pub fn get_pipeline(&self, id: ShaderId) -> Option<&crate::gpu::Pipeline> {
        self.pipelines.get(&id)
    }

    #[cfg(feature = "modern")]
    pub fn update_projection(&mut self, context: &mut BackendContext, buffer: &BackendBuffer<[u8; 64]>) {
        let proj_bind_group = BindGroup::create_uniform_bind_group(
            &self.proj_layout,
            context,
            buffer,
            Some("Projection Bind Group")
        ).expect("Failed to create projection bind group");
        
        self.proj_bind_group = Some(proj_bind_group);
    }

    #[cfg(not(feature = "modern"))]
    pub fn update_projection(&mut self, ctx: &Context, buffer: &wgpu::Buffer) {
        self.proj_bind_group = Some(ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.proj_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("Projection Bind Group"),
        }));
    }

    #[cfg(feature = "modern")]
    pub fn get_proj_bind_group(&self) -> Option<&RawBindGroup> {
        self.proj_bind_group.as_ref()
    }

    #[cfg(not(feature = "modern"))]
    pub fn get_proj_bind_group(&self) -> Option<&wgpu::BindGroup> {
        self.proj_bind_group.as_ref()
    }
}
