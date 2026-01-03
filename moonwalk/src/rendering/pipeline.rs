// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use std::collections::HashMap;
use crate::gpu::Context;
use crate::objects::ShaderId;
use crate::error::MoonWalkError;
use crate::r#abstract::*;

pub struct ShaderStore {
    pipelines: HashMap<ShaderId, crate::gpu::Pipeline>,
    proj_bind_group: Option<wgpu::BindGroup>,
    proj_layout: wgpu::BindGroupLayout,
}

impl ShaderStore {
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

    pub fn compile_shader(
        &mut self, 
        ctx: &Context, 
        src: &str, 
        format: wgpu::TextureFormat
    ) -> Result<ShaderId, MoonWalkError> {
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
            .build(ctx, format, &[&self.proj_layout])?;
        
        let id = ShaderId(self.pipelines.len() as u32 + 100);
        self.pipelines.insert(id, pipeline.pipeline);
        
        Ok(id)
    }

    pub fn get_pipeline(&self, id: ShaderId) -> Option<&crate::gpu::Pipeline> {
        self.pipelines.get(&id)
    }

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

    pub fn get_proj_bind_group(&self) -> Option<&wgpu::BindGroup> {
        self.proj_bind_group.as_ref()
    }
}
