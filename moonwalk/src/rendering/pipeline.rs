// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use std::collections::HashMap;

use crate::gpu::{Context, Pipeline, PipelineBuilder};
use crate::objects::ShaderId;
use crate::error::MoonWalkError;

pub struct ShaderStore {
    pipelines: HashMap<ShaderId, Pipeline>,
    pub proj_layout: wgpu::BindGroupLayout,
}

impl ShaderStore {
    pub fn new(ctx: &Context) -> Self {
        let proj_layout = ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Projection Layout"),
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
        });

        Self {
            pipelines: HashMap::new(),
            proj_layout,
        }
    }

    pub fn create_default_rect(&mut self, ctx: &Context, format: wgpu::TextureFormat) -> Result<ShaderId, MoonWalkError> {
        let use_split = crate::fallback::check_fallback(ctx);

        let pipeline = if !use_split {
            let instance_layout = wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<crate::rendering::vertex::ObjectInstance>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &[
                    // Pos + Size (vec4<f32>) 16 байт
                    // Смещение 0
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset: 0,
                        shader_location: 1
                    },

                    // UV (vec4<u16>) 8 байт
                    // Смещение 16
                    wgpu::VertexAttribute { 
                        format: wgpu::VertexFormat::Unorm16x4, 
                        offset: 16,
                        shader_location: 2
                    },

                    // Radii (vec4<u16>) 8 байт
                    // Смещение 16 + 8 = 24
                    wgpu::VertexAttribute { 
                        format: wgpu::VertexFormat::Uint16x4, 
                        offset: 24,
                        shader_location: 3
                    },
                    
                    // Gradient (Snorm16x4) 8 байт
                    // Смещение 24 + 8 = 32
                    wgpu::VertexAttribute { 
                        format: wgpu::VertexFormat::Snorm16x4,
                        offset: 32,
                        shader_location: 4
                    },
                    
                    // Extra Z + Rot (vec2<f32>) 8 байт
                    // Смещение 32 + 8 = 40
                    wgpu::VertexAttribute { 
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 40,
                        shader_location: 5
                    },
                    
                    // Color2 (u32) 4 байта
                    // Смещение 40 + 8 = 48
                    wgpu::VertexAttribute { 
                        format: wgpu::VertexFormat::Uint32,
                        offset: 48, 
                        shader_location: 6
                    },

                    // Color (u32) 4 байта
                    // Смещение 48 + 4 = 52
                    wgpu::VertexAttribute { 
                        format: wgpu::VertexFormat::Uint32,
                        offset: 52, 
                        shader_location: 7
                    },

                    // Type id (u32) 4 байта
                    // Смещение 52 + 4 = 56
                    wgpu::VertexAttribute { 
                        format: wgpu::VertexFormat::Uint32,
                        offset: 56, 
                        shader_location: 8
                    },
                ],
            };

            let texture_layout = Self::get_texture_layout(&ctx.device);

            PipelineBuilder::new(ctx, include_str!("../shaders/shape.wgsl"))
                .add_layout(Self::get_vertex_layout())
                .add_layout(instance_layout)
                .build(format, &[&self.proj_layout, &texture_layout])
        } else {
            crate::fallback::pipeline::create_split_pipeline(self, ctx, format)
        };

        let id = ShaderId(1);
        self.pipelines.insert(id, pipeline);
        Ok(id)
    }

    pub fn compile_shader(&mut self, ctx: &Context, src: &str, format: wgpu::TextureFormat) -> Result<ShaderId, MoonWalkError> {
        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<[f32; 15]>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![
                0 => Float32x3,
                1 => Float32x4,
                2 => Float32x2,
                3 => Float32x2,
                4 => Float32x4 
            ],
        };

        let pipeline = PipelineBuilder::new(ctx, src)
            .add_layout(vertex_layout)
            .build(format, &[&self.proj_layout]);
            
        let id = ShaderId(self.pipelines.len() as u32 + 100);
        self.pipelines.insert(id, pipeline);
        
        Ok(id)
    }

    pub fn get_pipeline(&self, id: ShaderId) -> Option<&Pipeline> {
        self.pipelines.get(&id)
    }

    pub fn get_proj_bind_group(&self, ctx: &Context, buffer: &wgpu::Buffer) -> wgpu::BindGroup {
        ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.proj_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("Projection Bind Group"),
        })
    }

    pub fn get_texture_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float {
                            filterable: true
                        },
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
        })
    }

    pub fn get_vertex_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<crate::rendering::vertex::QuadVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0
                },
            ],
        }
    }
}