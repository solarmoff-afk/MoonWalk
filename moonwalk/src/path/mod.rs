// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

pub mod svg;

use wgpu::util::DeviceExt;
use lyon::math::point;
use lyon::path::Path;
use lyon::tessellation::*;
use bytemuck::{Pod, Zeroable};

use crate::gpu::context::Context;
use crate::rendering::texture::Texture;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct VectorUniform {
    view_proj: [[f32; 4]; 4],
    color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct VectorVertex {
    pub position: [f32; 2],
}

/// Система для рендеринга векторной графики (векторных путей) в текстуры
pub struct VectorSystem {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl VectorSystem {
    pub fn new(ctx: &Context) -> Self {
        let bind_group_layout = ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Vector Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,

                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },

                count: None,
            }],
        });

        let pipeline_layout = ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Vector Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Vector Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("path.wgsl").into()),
        });

        let pipeline = ctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Vector Pipeline"),
            layout: Some(&pipeline_layout),

            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),

                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<VectorVertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,

                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 0,
                    }],
                }],

                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },

            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),

                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],

                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw, 
                cull_mode: None,
                ..Default::default()
            },

            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            bind_group_layout,
        }
    }

    pub fn render(
        &self,
        ctx: &Context,
        vertices: &[VectorVertex],
        indices: &[u16],
        width: u32,
        height: u32,
        color: [f32; 4],
        target: &Texture,
    ) {
        let vertex_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vector VBO"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vector IBO"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let mut matrix_stack = crate::gpu::MatrixStack::new();
        matrix_stack.set_ortho(width as f32, height as f32);
        
        let uniform_data = VectorUniform {
            view_proj: matrix_stack.projection.to_cols_array_2d(),
            color,
        };

        let uniform_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vector Uniforms"),
            contents: bytemuck::bytes_of(&uniform_data),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.bind_group_layout,
            
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],

            label: None,
        });

        let mut encoder = ctx.create_encoder();
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Vector Pass"),
                
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],

                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
        }

        ctx.submit(encoder);
    }

    pub fn render_to_texture(
        &self,
        ctx: &Context,
        vertices: &[VectorVertex],
        indices: &[u16],
        width: u32,
        height: u32,
        color: [f32; 4],
    ) -> Texture {
        let texture = Texture::create_render_target(
            ctx, 
            width, 
            height, 
            wgpu::TextureFormat::Rgba8UnormSrgb
        );

        self.render(ctx, vertices, indices, width, height, color, &texture);

        texture
    }
}

/// Обертка над билдером из lyon для удобного апи
pub struct PathBuilder {
    builder: lyon::path::Builder,
    color: [f32; 4],
    is_stroke: bool,
    stroke_width: f32,
}

impl PathBuilder {
    pub fn new() -> Self {
        Self {
            builder: Path::builder(),
            color: [1.0, 1.0, 1.0, 1.0],
            is_stroke: false,
            stroke_width: 1.0,
        }
    }

    pub fn set_color(&mut self, color: glam::Vec4) {
        self.color = color.to_array();
    }

    pub fn set_stroke(&mut self, width: f32) {
        self.is_stroke = true;
        self.stroke_width = width;
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.builder.begin(point(x, y));
    }

    pub fn line_to(&mut self, x: f32, y: f32) {
        self.builder.line_to(point(x, y));
    }

    pub fn quadratic_bezier_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.builder.quadratic_bezier_to(point(cx, cy), point(x, y));
    }
    
    pub fn cubic_bezier_to(&mut self, ctrl1_x: f32, ctrl1_y: f32, ctrl2_x: f32, ctrl2_y: f32, x: f32, y: f32) {
         self.builder.cubic_bezier_to(point(ctrl1_x, ctrl1_y), point(ctrl2_x, ctrl2_y), point(x, y));
    }
    
    pub fn close(&mut self) {
        self.builder.close();
    }

    /// Завершает построение, тесселирует и рендерит в текстуру. Возвращает айди
    /// новой текстуры
    pub fn tessellate(self, mw: &mut crate::MoonWalk, width: u32, height: u32) -> u32 {
        let path = self.builder.build();
        
        let mut geometry: VertexBuffers<VectorVertex, u16> = VertexBuffers::new();
        
        if self.is_stroke {
            let mut tessellator = StrokeTessellator::new();
            let options = StrokeOptions::default().with_line_width(self.stroke_width);
            
            let _ = tessellator.tessellate_path(
                &path,
                &options,
                &mut BuffersBuilder::new(&mut geometry, |vertex: StrokeVertex| {
                    VectorVertex { position: [vertex.position().x, vertex.position().y] }
                }),
            );
        } else {
            let mut tessellator = FillTessellator::new();
            let options = FillOptions::default();
            
            let _ = tessellator.tessellate_path(
                &path,
                &options,
                &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
                    VectorVertex {
                        position: [vertex.position().x, vertex.position().y]
                    }
                }),
            );
        }

        let texture = mw.renderer.vector_system.render_to_texture(
            &mw.renderer.context, 
            &geometry.vertices, 
            &geometry.indices, 
            width, 
            height, 
            self.color
        );
        
        mw.renderer.register_texture(texture)
    }

    pub fn tessellate_to(self, mw: &mut crate::MoonWalk, texture_id: u32, width: u32, height: u32) {
        let path = self.builder.build();
        
        let mut geometry: VertexBuffers<VectorVertex, u16> = VertexBuffers::new();
        
        if self.is_stroke {
            let mut tessellator = StrokeTessellator::new();
            let options = StrokeOptions::default().with_line_width(self.stroke_width);
            
            let _ = tessellator.tessellate_path(
                &path,
                &options,
                &mut BuffersBuilder::new(&mut geometry, |vertex: StrokeVertex| {
                    VectorVertex { position: [vertex.position().x, vertex.position().y] }
                }),
            );
        } else {
            let mut tessellator = FillTessellator::new();
            let options = FillOptions::default();
            
            let _ = tessellator.tessellate_path(
                &path,
                &options,
                &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
                    VectorVertex { position: [vertex.position().x, vertex.position().y] }
                }),
            );
        }

        if let Some(texture) = mw.renderer.state.textures.get(&texture_id) {
            mw.renderer.vector_system.render(
                &mw.renderer.context, 
                &geometry.vertices, 
                &geometry.indices, 
                width, 
                height, 
                self.color,
                texture
            );
        }
    }

    pub fn get_internal_builder(&mut self) -> &mut lyon::path::Builder {
        &mut self.builder
    }
}
