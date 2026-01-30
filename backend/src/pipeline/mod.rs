// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

pub struct RawPipeline {
    pipeline: wgpu::RenderPipeline,
}

impl RawPipeline {
    pub fn new(pipeline: wgpu::RenderPipeline) -> Self {
        Self {
            pipeline,
        }
    }
}

pub struct BackendPipeline<'a> {
    shader_src: Option<&'a str>,
    vertex_entry: Option<&'a str>,
    fragment_entry: Option<&'a str>,
    raw: Option<RawPipeline>,
}

impl BackendPipeline<'_> {
    pub fn new() -> Self {
        Self {
            shader_src: None,
            vertex_entry: None,
            fragment_entry: None,
            raw: None,
        }
    }

    // Конфигурация пайплайна
}