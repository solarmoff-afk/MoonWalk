// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use wgpu;

use crate::gpu::Pipeline;
use crate::gpu::context::Context;
use crate::gpu::pipeline::PipelineBuilder;
use crate::rendering::pipeline::ShaderStore;

/// Для фаллбека нужен отдельный пайплайн и соотвественно функция для его создания
/// принимает ShaderStore, gpu контекст и формат текстуры
pub fn create_split_pipeline(
    store: &ShaderStore,
    ctx: &Context,
    format: wgpu::TextureFormat
) -> Pipeline {
    let layout_a = wgpu::VertexBufferLayout {
        array_stride: 32,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &[
            // PosSize (локация 1), uv (локация 2) и extra (локация 5)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 0, 
                shader_location: 1
            },

            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Unorm16x4,
                offset: 16,
                shader_location: 2
            },

            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 24,
                shader_location: 5
            },
        ],
    };

    let layout_b = wgpu::VertexBufferLayout {
        array_stride: 32,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &[
            // Radii (локация 3), gradient (локация 4), color2 (локация 6),
            // color (локация 7), type (локация 8) и эффекты (локация 9)
            wgpu::VertexAttribute { 
                format: wgpu::VertexFormat::Uint16x4,
                offset: 0,
                shader_location: 3
            },

            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Snorm16x4,
                offset: 8,
                shader_location: 4
            },
            
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Uint32,
                offset: 16,
                shader_location: 6
            },

            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Uint32,
                offset: 20,
                shader_location: 7
            },

            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Uint32,
                offset: 24,
                shader_location: 8
            },

            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Uint16x2,
                offset: 28,
                shader_location: 9
            },
        ],
    };

    let texture_layout = ShaderStore::get_texture_layout(&ctx.device);

    PipelineBuilder::new(ctx, include_str!("../shaders/shape.wgsl"))
        .add_layout(ShaderStore::get_vertex_layout()) 
        .add_layout(layout_a)
        .add_layout(layout_b)
        .build(format, &[&store.proj_layout, &texture_layout])
}