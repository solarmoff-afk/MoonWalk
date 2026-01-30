// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

pub mod types;
pub mod vertex;
pub mod bind;

use types::*;
use vertex::VertexLayout;
use bind::BindGroup;

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use parking_lot::Mutex;

use crate::error::MoonBackendError;

/// Структура для кэширования пайплайна
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct PipelineCacheKey {
    shader_hash: u64,
    vertex_layouts_hash: u64,
    bind_groups_hash: u64,
    format_hash: u64,
}

// Глобальный кэш пайплайнов который необходим чтобы предотвратить перекомпиляцию
// уже существующего пайплайна
lazy_static::lazy_static! {
    static ref PIPELINE_CACHE: Mutex<HashMap<PipelineCacheKey, Arc<wgpu::RenderPipeline>>> = 
        Mutex::new(HashMap::new());
}

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

    /// Список (вектор) вершинных лайаутов
    vertex_layouts: Vec<VertexLayout>,
    bind_groups: Vec<BindGroup>,

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

            vertex_layouts: Vec::new(),
            bind_groups: Vec::new(),

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

    /// Этот метод устанавливает добавляет вертексный лайаут
    pub fn add_vertex_layout(mut self, layout: VertexLayout) -> Self {
        self.vertex_layouts.push(layout);
        self
    }

    /// Добавить bind group
    pub fn add_bind_group(mut self, bind_group: BindGroup) -> Self {
        self.bind_groups.push(bind_group);
        self
    }
    
    // [MAYBE]
    // Собрать все параметры и RenderConfig в raw gpu пайплайн
    // Это легаси с утечкой абстрации, использовать только метод collect
    // pub fn build(
    //     &self,
    //     ctx: &Context,
    //     wgpu_format: wgpu::TextureFormat,
    //     wgpu_bind_groups: &[&wgpu::BindGroupLayout],
    // ) -> Result<PipelineResult, MoonWalkError> {
    //     // Валидация конфигурации
    //     self.validate()?;

    //     let cache_key = self.create_cache_key(ctx);

    //     if let Some(cached) = PIPELINE_CACHE.lock().get(&cache_key).cloned() {
    //         return Ok(PipelineResult {
    //             pipeline: Pipeline { raw: (*cached).clone() },
    //             split_count: 1,
    //             used_stride: self.get_max_stride(),
    //             cache_hit: true,
    //         });
    //     }

    //     let result = match self.fallback_strategy {
    //         FallbackStrategy::None => self.build_direct(ctx, wgpu_format, wgpu_bind_groups, 1)?,
    //         FallbackStrategy::Adaptive => self.clone().build_with_fallback(ctx, wgpu_format, wgpu_bind_groups)?,
    //         FallbackStrategy::Split => self.clone().build_with_split(ctx, wgpu_format, wgpu_bind_groups)?,
    //         FallbackStrategy::Reduce => self.clone().build_with_reduce(ctx, wgpu_format, wgpu_bind_groups)?,
    //     };

    //     // Кэширование результата
    //     if let Some(label) = &self.label {
    //         log::debug!("Caching pipeline: {}", label);
    //     }

    //     PIPELINE_CACHE.lock().insert(cache_key, Arc::new(result.pipeline.raw.clone()));

    //     Ok(result)
    // }

    fn validate(&self) -> Result<(), MoonBackendError> {
        if self.vertex_entry.is_empty() {
            return Err(MoonBackendError::PipelineError("Vertex shader entry point not set".into()));
        }

        if self.fragment_entry.is_empty() {
            return Err(MoonBackendError::PipelineError("Fragment shader entry point not set".into()));
        }

        if self.vertex_layouts.is_empty() {
            return Err(MoonBackendError::PipelineError("No vertex layouts specified".into()));
        }

        // Проверка уникальности shader locations
        let mut locations = HashSet::new();

        for layout in &self.vertex_layouts {
            if let Err(e) = layout.validate() {
                return Err(MoonBackendError::PipelineError(format!("Invalid vertex layout: {}", e)));
            }

            for attr in &layout.attributes {
                if !locations.insert(attr.location) {
                    return Err(MoonBackendError::PipelineError(
                        format!("Duplicate shader location: {}", attr.location)
                    ));
                }
            }
        }

        // Проверка уникальность bindings
        let mut bindings = HashSet::new();

        for bind_group in &self.bind_groups {
            for entry in &bind_group.entries {
                let key = (entry.binding, entry.entry_type);
                
                if !bindings.insert(key) {
                    return Err(MoonBackendError::PipelineError(
                        format!("Duplicate binding: {}", entry.binding)
                    ));
                }
            }
        }

        Ok(())
    }
}