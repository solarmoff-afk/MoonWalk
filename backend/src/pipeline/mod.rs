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

pub struct BackendPipeline {
    shader_source: Option<String>,
    vertex_entry: Option<String>,
    fragment_entry: Option<String>,
    raw: Option<RawPipeline>,
}

impl BackendPipeline {
    pub fn new() -> Self {
        Self {
            shader_source: None,
            vertex_entry: None,
            fragment_entry: None,
            raw: None,
        }
    }

    // Конфигурация пайплайна
    // Установить исходный код шейдера
    pub fn set_shader_source(&mut self, source: String) {
        self.shader_source = Some(source);
    }

    // Установить точку входа для вершинного шейдера
    pub fn set_vertex_entry(&mut self, vertex_entry: String) {
        self.vertex_entry = Some(vertex_entry);
    }

    // Установить точку входа для фрагментного шейдера
    pub fn set_fragment_entry(&mut self, fragment_entry: String) {
        self.fragment_entry = Some(fragment_entry);
    }
}