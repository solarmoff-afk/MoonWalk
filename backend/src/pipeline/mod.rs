// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

pub mod types;

use types::*;

/// Стратегия обработки ограничений видеокарты по данным на вершину
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallbackStrategy {
    /// Разделить большие буферы на несколько маленьких
    Split,

    /// Уменьшить страйд до допустимого значения
    Reduce,

    /// Адаптивная стратегия где сначала сплит, потом reduce
    Adaptive,

    /// Без фаллбека то есть вернуть ошибку если не укладывается в лимиты
    None,
}

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

/// Конфигурация рендера
#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// Формат цели рендера
    pub target_format: Format,
    
    /// Режим смешивания
    pub blend_mode: BlendMode,
    
    /// Режим отсечения
    pub cull_mode: CullMode,
    
    /// Топология примитивов
    pub topology: Topology,
    
    /// Включён ли тест глубины
    pub depth_test: bool,
    pub depth_write: bool,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            target_format: Format::Float32x4,
            blend_mode: BlendMode::None,
            cull_mode: CullMode::None,
            topology: Topology::TriangleList,
            depth_test: false,
            depth_write: false,
        }
    }
}

pub struct BackendPipeline {
    shader_source: String,
    vertex_entry: String,
    fragment_entry: String,
    render_config: RenderConfig,
    label: Option<String>,

    /// Стратегия разрешения паники validation error от wgpu когда данные не
    /// помещаются в видеокарту
    fallback_strategy: FallbackStrategy,

    raw: Option<RawPipeline>,
}

impl BackendPipeline {
    pub fn new(shader_source: &str) -> Self {
        Self {
            shader_source: shader_source.to_string(),
            vertex_entry: "vs_main".to_string(),
            fragment_entry: "fs_main".to_string(),
            render_config: RenderConfig::default(),
            label: None,

            // Стандартно adaptive
            fallback_strategy: FallbackStrategy::Adaptive,

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

    /// Этот метод нужен чтобы установить название пайплайна для отладки
    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    /// Этот метод устанавливает формат цели рендера
    pub fn target_format(mut self, format: Format) -> Self {
        self.render_config.target_format = format;
        self
    }

    /// Этот метод устанавливает режим блендинга
    pub fn blend(mut self, mode: BlendMode) -> Self {
        self.render_config.blend_mode = mode;
        self
    }

    /// Этот метод устанавливает режим отсечения граней
    pub fn cull(mut self, mode: CullMode) -> Self {
        self.render_config.cull_mode = mode;
        self
    }

    /// Этот метод устанавливает топологию примитивов
    pub fn topology(mut self, topology: Topology) -> Self {
        self.render_config.topology = topology;
        self
    }

    /// Этот метод устанавливает тест глубины
    pub fn depth_test(mut self, enabled: bool) -> Self {
        self.render_config.depth_test = enabled;
        self
    }

    /// Этот метод устанавливает запись глубины (true или false)
    pub fn depth_write(mut self, enabled: bool) -> Self {
        self.render_config.depth_write = enabled;
        self
    }

    /// Этот метод устанавливает стратегию фалбек
    pub fn fallback_strategy(mut self, strategy: FallbackStrategy) -> Self {
        self.fallback_strategy = strategy;
        self
    }
}