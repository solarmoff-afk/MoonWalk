// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

pub mod svg;

use wgpu::util::DeviceExt;
use lyon::math::point;
use lyon::path::Path;
use lyon::tessellation::*;
use bytemuck::{Pod, Zeroable};

use crate::MoonWalkError;
use crate::gpu::context::Context;
use crate::rendering::texture::Texture;
use crate::r#abstract::*;

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
    bind_group: Option<wgpu::BindGroup>,
}

impl VectorSystem {
    pub fn new(ctx: &Context) -> Result<Self, MoonWalkError> {
        let shader_source = include_str!("path.wgsl");
        
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
            .add_bind_group(
                BindGroup::new()
                    .add_uniform(0, ShaderStage::Both)
            )
            .blend(BlendMode::Alpha)
            .cull(CullMode::None)
            .topology(Topology::TriangleList)
            .depth_test(false)
            .depth_write(false)
            .label("vector_path")
            .build(ctx, wgpu::TextureFormat::Rgba8UnormSrgb, &[])?;
        
        Ok(Self {
            pipeline: pipeline.pipeline.raw,
            bind_group: None,
        })
    }

    pub fn render(
        &mut self,
        ctx: &Context,
        vertices: &[VectorVertex],
        indices: &[u16],
        width: u32,
        height: u32,
        color: [f32; 4],
        target: &Texture,
    ) {
        if vertices.is_empty() || indices.is_empty() {
            return;
        }

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

        self.bind_group = Some(ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.get_bind_group_layout(ctx),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: None,
        }));

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
            if let Some(bind_group) = &self.bind_group {
                pass.set_bind_group(0, bind_group, &[]);
            }
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
        }

        ctx.submit(encoder);
    }

    pub fn render_to_texture(
        &mut self,
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

    fn get_bind_group_layout(&self, ctx: &Context) -> wgpu::BindGroupLayout {
        BindGroup::new()
            .add_uniform(0, ShaderStage::Both)
            .build(ctx)
            .expect("Failed to create vector bind group layout")
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
