// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use std::collections::HashMap;

use moonwalk_backend::core::buffer::BackendBuffer;
use moonwalk_backend::core::context::BackendContext;
use moonwalk_backend::pipeline::{PipelineResult, RawPipeline};
use moonwalk_backend::pipeline::bind::{RawBindGroup, BindGroup, RawBindGroupLayout};
use moonwalk_backend::pipeline::types::ShaderStage;
use moonwalk_backend::pipeline::{
    BackendPipeline, FallbackStrategy,
    types::{BlendMode, CullMode, Format, SamplerType, StepMode, TextureType, Topology},
    vertex::{VertexAttr, VertexLayout, create_rect_instance_layout}
};
use moonwalk_backend::render::texture::BackendTextureFormat;

use crate::objects::ShaderId;
use crate::error::MoonWalkError;
use crate::rendering::state::GlobalUniform;

pub struct ShaderStore {
    pipelines: HashMap<ShaderId, RawPipeline>,
    proj_bind_group: Option<RawBindGroup>,
    proj_layout: RawBindGroupLayout,
    arena: Vec<PipelineResult>,
}

impl ShaderStore {
    pub fn new(context: &mut BackendContext) -> Result<Self, MoonWalkError> {
        let proj_layout = BindGroup::new()
            .add_uniform(0, ShaderStage::Vertex)
            .build(context)?;
        
        Ok(Self {
            pipelines: HashMap::new(),
            proj_bind_group: None,
            proj_layout,
            arena: Vec::new(),
        })
    }

    pub fn create_default_rect(
        &mut self,
        context: &mut BackendContext,
        format: BackendTextureFormat
    ) -> Result<ShaderId, MoonWalkError> {
        let shader_source = include_str!("../shaders/shape.wgsl");
        
        let uniform_layout = BindGroup::new()
            .add_uniform(0, ShaderStage::Vertex)
            .build(context)?;
        
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
            .add_bind_group(
                BindGroup::new()
                    .add_texture(0, TextureType::Float)
                    .add_sampler(1, SamplerType::Linear) 
            )
            .blend(BlendMode::Alpha)
            .cull(CullMode::None)
            .topology(Topology::TriangleList)
            .depth_test(false)
            .depth_write(false)
            .fallback_strategy(FallbackStrategy::Adaptive)
            .label("default_rect")
            .build(context, format, &[&uniform_layout, &texture_layout])?;
        
        let id = ShaderId(1);
        
        // [MAYBE]
        // Я понятия не имею это Arc или не Arc, я покопался в исходниках wgpu,
        // pub struct RenderPipeline { pub(crate) inner: dispatch::DispatchRenderPipeline, }
        // Тут вот такое есть DispatchRenderPipeline в текущей версии генерируется макросом,
        // а макрос действительно использует Arc::new, но меня смутило что только
        // для wgpu_core, для Webgpu и custom там уже не Arc, а Self::WebGPU(value), я так
        // поэтому надеюсь что клонирование не супер дорогое
        let raw = pipeline.get_raw()?.clone();
        
        self.pipelines.insert(id, raw);
        
        Ok(id)
    }

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
                    .add_uniform(0, ShaderStage::Both)
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

        // [MAYBE]
        let raw = pipeline.get_raw()?.clone();
        self.pipelines.insert(id, raw);
        
        Ok(id)
    }

    pub fn get_pipeline(&self, id: ShaderId) -> Option<&RawPipeline> {
        self.pipelines.get(&id)
    }

    pub fn update_projection(&mut self, context: &mut BackendContext, buffer: &BackendBuffer<GlobalUniform>) {
        let proj_bind_group = BindGroup::create_uniform_bind_group(
            &self.proj_layout,
            context,
            buffer,
            Some("Projection Bind Group")
        ).expect("Failed to create projection bind group");
        
        self.proj_bind_group = Some(proj_bind_group);
    }

    pub fn get_proj_bind_group(&self) -> Option<&RawBindGroup> {
        self.proj_bind_group.as_ref()
    }
}
