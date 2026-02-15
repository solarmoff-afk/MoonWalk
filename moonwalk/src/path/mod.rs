// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

#![allow(dead_code)]

pub mod svg;

use moonwalk_backend::core::context::BackendContext;
use moonwalk_backend::pipeline::RawPipeline;
use moonwalk_backend::pipeline::bind::RawBindGroup;
use moonwalk_backend::render::texture::BackendTexture;

use lyon::math::point;
use lyon::path::Path;
use lyon::tessellation::*;
use bytemuck::{Pod, Zeroable};

use crate::MoonWalkError;
use crate::{perf_start, perf_end};

/// Настройка концов линий
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineCap {
    /// Обрубленный конец (стандарт)
    #[default]
    Butt,
    /// Скругленный конец
    Round,
    /// Квадратный выступ
    Square,
}

/// Настройка соединения линий
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineJoin {
    /// Острый угол (стандарт)
    #[default]
    Miter,
    /// Скругленный угол
    Round,
    /// Срезанный угол (фаска)
    Bevel,
}

/// Правило заливки фигур
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FillRule {
    /// Заливка определяется направлением линий (стандарт SVG)
    #[default]
    NonZero,
    /// Заливка определяется пересечением (четное/нечетное) что удобно для дырок
    EvenOdd,
}

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
    pipeline: RawPipeline,
    bind_group: Option<RawBindGroup>,
}

impl VectorSystem {
    pub fn new(context: &mut BackendContext) -> Result<Self, MoonWalkError> {
        use moonwalk_backend::pipeline::{BackendPipeline, bind::BindGroup, types::{BlendMode, CullMode, Format, ShaderStage, StepMode, Topology}, vertex::{VertexAttr, VertexLayout}};

        let shader_source = include_str!("../shaders/path.wgsl");
        let texture_format = context.get_format();
        
        let bind_group_layout = BindGroup::new()
            .add_uniform(0, ShaderStage::Both)
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
            .build(context, texture_format, &[&bind_group_layout])?;
        
        Ok(Self {
            // Паники здесь никогда не будет, так как прямо выше идёт вызов метода
            // build и обработка Result из него, так что pipeline.pipeline
            // точно существует
            pipeline: pipeline.pipeline.expect("[IRE]: New vector pipeline"),

            bind_group: None,
        })
    }

    pub fn render(
        &mut self,
        context: &mut BackendContext,
        vertices: &[VectorVertex],
        indices: &[u16],
        width: u32,
        height: u32,
        color: [f32; 4],
        target: &BackendTexture,
    ) -> Result<(), MoonWalkError> {
        use moonwalk_backend::{core::{buffer::BackendBuffer, encoder::BackendEncoder}, pipeline::{bind::BindGroup, types::ShaderStage}, render::pass::RenderPass};

        if vertices.is_empty() || indices.is_empty() {
            return Ok(());
        }

        perf_start!("[VECTOR]: Create vector vertex buffer");
            let vertex_buffer = BackendBuffer::vertex(context, vertices)?;
        perf_end!("[VECTOR]: Create vector vertex buffer");
        
        // [HACK]
        // Перевод u16 в u32 чтобы сохранить легаси сигнатуру
        
        perf_start!("[VECTOR]: Convert u16 indices to u32");
            let u32_indices: Vec<u32> = indices.iter().map(|&i| i as u32).collect();
            let index_buffer = BackendBuffer::<u32>::index(context, &u32_indices)?;
        perf_end!("[VECTOR]: Convert u16 indices to u32");

        let mut matrix_stack = crate::gpu::MatrixStack::new();
        matrix_stack.set_ortho(width as f32, height as f32);
        
        let uniform_data = VectorUniform {
            view_proj: matrix_stack.projection.to_cols_array_2d(),
            color,
        };

        let uniform_buffer = BackendBuffer::uniform(context, &uniform_data)?;

        // Создаём bind group
       
        perf_start!("[VECTOR]: Create bind group");
            let bind_group_layout = BindGroup::new()
                .add_uniform(0, ShaderStage::Both)
                .build(context)?;

            let bind_group = BindGroup::create_uniform_bind_group(
                &bind_group_layout,
                context,
                &uniform_buffer,
                None,
            )?;
        perf_end!("[VECTOR]: Create bind group");

        perf_start!("[VECTOR]: Create encoder and render pass on render");
            let mut encoder = BackendEncoder::new(context, "Vector encoder")?;
        
            let mut pass = RenderPass::new(
                &mut encoder,
                target,
                Some(color.into()),
                "Vector render pass",
            )?;
        perf_end!("[VECTOR]: Create encoder and render pass on render");

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind_group);
        pass.set_vertex_buffer(0, &vertex_buffer);
        pass.set_index_buffer(&index_buffer);
        pass.draw_indexed(indices.len() as u32);

        drop(pass);

        encoder.submit_frame(context)?;
        Ok(())
    }

    pub fn render_to_texture(
        &mut self,
        context: &mut BackendContext,
        vertices: &[VectorVertex],
        indices: &[u16],
        width: u32,
        height: u32,
        color: [f32; 4],
    ) -> Result<BackendTexture, MoonWalkError> {
        let mut texture = BackendTexture::new(width, height);
        texture.config.set_format(context.get_format());
        texture.create_render_target(context, width, height)?;

        self.render(context, vertices, indices, width, height, color, &texture)?;

        Ok(texture)
    }
}

/// Обертка над билдером из lyon для удобного апи
pub struct PathBuilder {
    builder: lyon::path::Builder,
    color: [f32; 4],
    is_stroke: bool,
    stroke_options: StrokeOptions,
    fill_options: FillOptions,
}

impl PathBuilder {
    pub fn new() -> Self {
        Self {
            builder: Path::builder(),
            color: [1.0, 1.0, 1.0, 1.0],
            is_stroke: false,
            stroke_options: StrokeOptions::default()
                .with_line_cap(lyon::tessellation::LineCap::Round)
                .with_line_join(lyon::tessellation::LineJoin::Round)
                .with_tolerance(0.1),
            fill_options: FillOptions::default()
                .with_tolerance(0.1),
        }
    }

    pub fn set_line_cap(&mut self, cap: LineCap) {
        let lyon_cap = match cap {
            LineCap::Butt => lyon::tessellation::LineCap::Butt,
            LineCap::Round => lyon::tessellation::LineCap::Round,
            LineCap::Square => lyon::tessellation::LineCap::Square,
        };

        self.stroke_options = self.stroke_options.with_line_cap(lyon_cap);
    }

    pub fn set_line_join(&mut self, join: LineJoin) {
        let lyon_join = match join {
            LineJoin::Miter => lyon::tessellation::LineJoin::Miter,
            LineJoin::Round => lyon::tessellation::LineJoin::Round,
            LineJoin::Bevel => lyon::tessellation::LineJoin::Bevel,
        };

        self.stroke_options = self.stroke_options.with_line_join(lyon_join);
    }

    pub fn set_fill_rule(&mut self, rule: FillRule) {
        let lyon_rule = match rule {
            FillRule::NonZero => lyon::tessellation::FillRule::NonZero,
            FillRule::EvenOdd => lyon::tessellation::FillRule::EvenOdd,
        };

        self.fill_options = self.fill_options.with_fill_rule(lyon_rule);
    }

    pub fn set_tolerance(&mut self, tolerance: f32) {
        self.stroke_options = self.stroke_options.with_tolerance(tolerance);
        self.fill_options = self.fill_options.with_tolerance(tolerance);
    }

    pub fn set_color(&mut self, color: glam::Vec4) {
        self.color = color.to_array();
    }

    pub fn set_stroke(&mut self, width: f32) {
        self.is_stroke = true;
        self.stroke_options = self.stroke_options.with_line_width(width);
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

            let _ = tessellator.tessellate_path(
                &path,
                &self.stroke_options,
                &mut BuffersBuilder::new(&mut geometry, |vertex: StrokeVertex| {
                    VectorVertex {
                        position: [
                            vertex.position().x,
                            vertex.position().y
                        ]
                    }
                }),
            );
        } else {
            let mut tessellator = FillTessellator::new();

            let _ = tessellator.tessellate_path(
                &path,
                &self.fill_options,
                &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
                    VectorVertex {
                        position: [vertex.position().x, vertex.position().y]
                    }
                }),
            );
        }

        let texture = mw.renderer.vector_system.render_to_texture(
            &mut mw.renderer.context, 
            &geometry.vertices, 
            &geometry.indices, 
            width, 
            height, 
            self.color
        );
        
        // [HACK]
        // Убрать expect
        mw.renderer.register_texture(texture.expect("Texture not created"))
    }

    pub fn tessellate_to(self, mw: &mut crate::MoonWalk, texture_id: u32, width: u32, height: u32) {
        let path = self.builder.build();
        
        let mut geometry: VertexBuffers<VectorVertex, u16> = VertexBuffers::new();
        
        if self.is_stroke {
            let mut tessellator = StrokeTessellator::new();

            let _ = tessellator.tessellate_path(
                &path,
                &self.stroke_options,
                &mut BuffersBuilder::new(&mut geometry, |vertex: StrokeVertex| {
                    VectorVertex { position: [vertex.position().x, vertex.position().y] }
                }),
            );
        } else {
            let mut tessellator = FillTessellator::new();

            let _ = tessellator.tessellate_path(
                &path,
                &self.fill_options,
                &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
                    VectorVertex { position: [vertex.position().x, vertex.position().y] }
                }),
            );
        }

        if let Some(texture) = mw.renderer.state.textures.get(&texture_id) {
            mw.renderer.vector_system.render(
                &mut mw.renderer.context, 
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