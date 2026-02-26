// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use std::borrow::Cow;

use crate::core::context::BackendContext;

#[derive(Debug, Clone)]
pub struct Pipeline {
    pub raw: wgpu::RenderPipeline,
}

pub struct PipelineBuilder<'a> {
    shader_src: &'a str,
    vertex_entry: &'a str,
    fragment_entry: &'a str,
    vertex_layouts: Vec<wgpu::VertexBufferLayout<'a>>,
    topology: wgpu::PrimitiveTopology,
    polygon_mode: wgpu::PolygonMode,
    cull_mode: Option<wgpu::Face>,
    front_face: wgpu::FrontFace,
    blend: Option<wgpu::BlendState>,
    depth_stencil: Option<wgpu::DepthStencilState>,
}

impl<'a> PipelineBuilder<'a> {
    pub fn new(ctx: &'a Context, shader_src: &'a str) -> Self {
        Self {
            ctx,
            shader_src,
            vertex_entry: "vs_main",
            fragment_entry: "fs_main",
            vertex_layouts: Vec::new(),
            topology: wgpu::PrimitiveTopology::TriangleList,
            polygon_mode: wgpu::PolygonMode::Fill,
            cull_mode: None,
            front_face: wgpu::FrontFace::Ccw,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            depth_stencil: None,
        }
    }

    pub fn vertex_entry(mut self, entry: &'a str) -> Self {
        self.vertex_entry = entry;
        self
    }
    
    pub fn fragment_entry(mut self, entry: &'a str) -> Self {
        self.fragment_entry = entry;
        self
    }

    pub fn add_layout(mut self, layout: wgpu::VertexBufferLayout<'a>) -> Self {
        self.vertex_layouts.push(layout);
        self
    }
    
    pub fn with_topology(mut self, topology: wgpu::PrimitiveTopology) -> Self {
        self.topology = topology;
        self
    }
    
    pub fn with_polygon_mode(mut self, mode: wgpu::PolygonMode) -> Self {
        self.polygon_mode = mode;
        self
    }
    
    pub fn with_cull_mode(mut self, mode: Option<wgpu::Face>) -> Self {
        self.cull_mode = mode;
        self
    }
    
    pub fn with_front_face(mut self, front_face: wgpu::FrontFace) -> Self {
        self.front_face = front_face;
        self
    }
    
    pub fn blend_state(mut self, blend: wgpu::BlendState) -> Self {
        self.blend = Some(blend);
        self
    }
    
    pub fn no_blend(mut self) -> Self {
        self.blend = None;
        self
    }

    pub fn depth_stencil_state(mut self, state: wgpu::DepthStencilState) -> Self {
        self.depth_stencil = Some(state);
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
                entry_point: Some(self.vertex_entry),
                buffers: &self.vertex_layouts,
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some(self.fragment_entry),
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
                front_face: self.front_face,
                cull_mode: self.cull_mode,
                polygon_mode: self.polygon_mode,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: self.depth_stencil,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Pipeline { raw }
    }
}