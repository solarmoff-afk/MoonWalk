// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use std::collections::HashMap;

use crate::easy_gpu::{Context, Pipeline, PipelineBuilder};
use crate::objects::ShaderId;
use crate::error::MoonWalkError;

pub struct ShaderStore {
    pipelines: HashMap<ShaderId, Pipeline>,
    pub proj_layout: wgpu::BindGroupLayout,
    // pub glyph_layout: wgpu::BindGroupLayout,
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

        // let glyph_layout = ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //     label: Some("Glyph Layout"),
        //     entries: &[
        //         wgpu::BindGroupLayoutEntry {
        //             binding: 0,
        //             visibility: wgpu::ShaderStages::FRAGMENT,
        //             ty: wgpu::BindingType::Texture {
        //                 multisampled: false,
        //                 view_dimension: wgpu::TextureViewDimension::D2,
        //                 sample_type: wgpu::TextureSampleType::Float { filterable: true },
        //             },
        //             count: None,
        //         },
        //         wgpu::BindGroupLayoutEntry {
        //             binding: 1,
        //             visibility: wgpu::ShaderStages::FRAGMENT,
        //             ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        //             count: None,
        //         },
        //     ],
        // });

        Self {
            pipelines: HashMap::new(),
            proj_layout,
            // glyph_layout,
        }
    }

    pub fn create_default_rect(&mut self, ctx: &Context, format: wgpu::TextureFormat) -> Result<ShaderId, MoonWalkError> {
        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<crate::rendering::vertex::QuadVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // @location(0) position: vec2<f32>
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x2, offset: 0, shader_location: 0 },
            ],
        };

        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<crate::rendering::vertex::RectInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // Pos + ыize (vec4<f32>) 16 байт
                wgpu::VertexAttribute { 
                    format: wgpu::VertexFormat::Float32x4, 
                    offset: 0,  
                    shader_location: 1 
                },

                // Radii (vec4<f32>) 16 байт
                // Смещение 16
                wgpu::VertexAttribute { 
                    format: wgpu::VertexFormat::Float32x4, 
                    offset: 16, 
                    shader_location: 2 
                },
                
                // Extra: Z + Rot (vec2<f32>) 8 байт
                // Смещение 16 + 16 = 32
                wgpu::VertexAttribute { 
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 32, 
                    shader_location: 3 
                },
                
                // Color (u32) 4 байта
                // Смещение 32 + 8 = 40
                wgpu::VertexAttribute { 
                    format: wgpu::VertexFormat::Uint32,
                    offset: 40, 
                    shader_location: 4 
                },
            ],
        };

        let pipeline = PipelineBuilder::new(ctx, include_str!("../shaders/rect.wgsl"))
            .add_layout(vertex_layout)
            .add_layout(instance_layout)
            .build(format, &[&self.proj_layout]);

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
}