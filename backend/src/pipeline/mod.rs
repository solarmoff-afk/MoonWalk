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
    shader_source: String,
    vertex_entry: String,
    fragment_entry: String,
    raw: Option<RawPipeline>,
}

impl BackendPipeline {
    pub fn new(shader_source: &str) -> Self {
        Self {
            shader_source: shader_source.to_string(),
            vertex_entry: "vs_main".to_string(),
            fragment_entry: "fs_main".to_string(),
            raw: None,
        }
    }

    /// Метод чтобы установить точку входа вершинного шейдера
    pub fn vertex_shader(mut self, entry: &str) -> Self {
        self.vertex_entry = entry.to_string();
        self
    }

    /// Метол чтобы установить точку входа фрагментного шейдера
    pub fn fragment_shader(mut self, entry: &str) -> Self {
        self.fragment_entry = entry.to_string();
        self
    }
}