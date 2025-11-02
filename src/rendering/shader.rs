use std::collections::HashMap;
use std::sync::Arc;
use crate::error::MoonWalkError;
use crate::objects::ShaderId;

const RECT_WGSL: &str = include_str!("rect.wgsl");
const TEXT_WGSL: &str = include_str!("text.wgsl");

pub struct ShaderStore {
    device: Arc<wgpu::Device>,
    next_id: u32,
    pipelines: HashMap<ShaderId, wgpu::RenderPipeline>,
    glyph_bind_group_layout: wgpu::BindGroupLayout,
}

impl ShaderStore {
    pub fn new(device: Arc<wgpu::Device>) -> Self {
        let glyph_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Glyph Texture Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        Self {
            device,
            next_id: 1,
            pipelines: HashMap::new(),
            glyph_bind_group_layout,
        }
    }
    
    pub fn get_pipeline(&self, shader_id: ShaderId) -> Option<&wgpu::RenderPipeline> {
        self.pipelines.get(&shader_id)
    }

    pub fn create_default_rect_shader(&mut self) -> Result<ShaderId, MoonWalkError> {
        self.compile_shader(RECT_WGSL)
    }

    pub fn create_default_text_shader(&mut self) -> Result<ShaderId, MoonWalkError> {
        self.compile_shader(TEXT_WGSL)
    }

    pub fn compile_shader(&mut self, src: &str) -> Result<ShaderId, MoonWalkError> {
        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Custom Shader"),
            source: wgpu::ShaderSource::Wgsl(src.into()),
        });
        
        let id = ShaderId::from(self.next_id);
        self.next_id += 1;
        
        let pipeline = self.create_pipeline(&shader, id);
        self.pipelines.insert(id, pipeline);

        Ok(id)
    }
    
    fn create_pipeline(&self, shader_module: &wgpu::ShaderModule, shader_id: ShaderId) -> wgpu::RenderPipeline {
        let proj_bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("Projection Bind Group Layout"),
        });

        let mut bind_group_layouts = vec![&proj_bind_group_layout];
        
        if shader_id == ShaderId(2) {
             bind_group_layouts.push(&self.glyph_bind_group_layout);
        }

        let layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &bind_group_layouts,
            push_constant_ranges: &[],
        });
        
        let (vertex_entry, vertex_buffers) = if shader_id == ShaderId(2) {
            ("vs_text_main", &[
                wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 9]>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4, 2 => Float32x2],
                }
            ])
        } else {
            ("vs_rect_main", &[
                wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 15]>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4, 2 => Float32x2, 3 => Float32x2, 4 => Float32x4],
                }
            ])
        };

        let fragment_entry = if shader_id == ShaderId(2) { "fs_text_main" } else { "fs_rect_main" };

        self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: shader_module,
                entry_point: vertex_entry,
                buffers: vertex_buffers,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader_module,
                entry_point: fragment_entry,
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        })
    }
}