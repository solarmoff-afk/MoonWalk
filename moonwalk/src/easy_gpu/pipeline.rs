// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use std::borrow::Cow;

use crate::easy_gpu::context::Context;

pub struct Pipeline {
    pub raw: wgpu::RenderPipeline,
}

pub struct PipelineBuilder<'a> {
    ctx: &'a Context,
    shader_src: &'a str,
    vertex_layouts: Vec<wgpu::VertexBufferLayout<'a>>,
    topology: wgpu::PrimitiveTopology,
    blend: Option<wgpu::BlendState>,
    stencil: Option<wgpu::StencilState>,
}

impl<'a> PipelineBuilder<'a> {
    pub fn new(ctx: &'a Context, shader_src: &'a str) -> Self {
        Self {
            ctx,
            shader_src,
            vertex_layouts: Vec::new(),
            topology: wgpu::PrimitiveTopology::TriangleList,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            stencil: None,
        }
    }

    pub fn add_layout(mut self, layout: wgpu::VertexBufferLayout<'a>) -> Self {
        self.vertex_layouts.push(layout);
        self
    }
    
    pub fn with_topology(mut self, topology: wgpu::PrimitiveTopology) -> Self {
        self.topology = topology;
        self
    }
    
    pub fn with_stencil(mut self, stencil: wgpu::StencilState) -> Self {
        self.stencil = Some(stencil);
        self
    }

    pub fn no_blend(mut self) -> Self {
        self.blend = None;
        self
    }

    pub fn build(self, target_format: wgpu::TextureFormat, bind_group_layouts: &[&wgpu::BindGroupLayout]) -> Pipeline {
        let shader = self.ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader Module"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(self.shader_src)),
        });

        let layout = self.ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts,
            push_constant_ranges: &[],
        });

        let raw = self.ctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &self.vertex_layouts,
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: self.blend,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: self.topology,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: self.stencil.map(|s| wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: s,
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Pipeline { raw }
    }
}