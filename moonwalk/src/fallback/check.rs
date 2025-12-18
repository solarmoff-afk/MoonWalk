// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use std::sync::OnceLock;
use wgpu;

use crate::gpu::context::Context;
use crate::debug_println;

// Проверка лимитов драйвера может вызываться несколько раз, поэтому нужна
// глобальная переменная для кэширования результата (Чтобы не создавать пайплайн
// каждый раз)
static FALLBACK_STATUS: OnceLock<bool> = OnceLock::new();

/// Эта функция возвращает статус. Если true то нужно использлвать fallback пайплайн
/// и деление структуры инстанса. Кэширует результат и во второй (Либо третий и т.д.) раз
/// возвразает сохранённый в памяти результат 
pub fn is_fallback_required(ctx: &Context) -> bool {
    *FALLBACK_STATUS.get_or_init(|| {
        perform_hardware_check(ctx)
    })
}

fn perform_hardware_check(ctx: &Context) -> bool {
    let device = &ctx.device;

    // [MAYBE]
    // По быстрому тестированию на одном устройстве max_vertex_buffer_array_stride
    // возвразает 2048 когда реальный лимит ~86 байт. Ему доверять особо нельзя,
    // но возможно он покажет правильное значение и сэкономит время. В норме он
    // показывает число из спецификации вулкана поэтому если он покажет 32 (либо другое)
    // то это будет тревожный звоночек. Если что-то пойдёт не так можно удалить этот
    // блок

    let max_stride = device.limits().max_vertex_buffer_array_stride;
    if max_stride < 64 {
        debug_println!("Max vertex buffer: {}", max_stride);
        return true;
    }

    // Если не получилось (вероятнее всего) можно попробовать создать тестовый
    // пайплайн где 64 байта на вершину и если он упадёт, то драйвер не поддерживает
    // 64 байтовый режим
    let result = pollster::block_on(async {
        device.push_error_scope(wgpu::ErrorFilter::Validation);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Limit Check Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/empty.wgsl").into()),
        });

        let _ = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Stride Limit Test"),
            layout: None,
            cache: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset: 0,
                        shader_location: 0,
                    }],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let error = device.pop_error_scope().await;
        
        if let Some(e) = error {
            debug_println!("Hardware check failed, stride 64 not supported, use fallback: {:?}", e);
            true
        } else {
            debug_println!("64 byte stride supported, use fast mode");
            false
        }
    });

    result
}