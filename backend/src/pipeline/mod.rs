// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

pub mod types;
pub mod vertex;
pub mod bind;

use types::*;
use vertex::VertexLayout;
use bind::{BindGroup, RawBindGroupLayout};

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::borrow::Cow;
use parking_lot::Mutex;

use crate::error::MoonBackendError;
use crate::core::context::BackendContext;
use crate::render::texture::{BackendTextureFormat, map_format_to_wgpu};

/// Структура для кэширования пайплайна
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct PipelineCacheKey {
    shader_hash: u64,
    vertex_layouts_hash: u64,
    bind_groups_hash: u64,
    format_hash: u64,
}

/// Результат создания пайплайна
#[derive(Debug, Clone)]
pub struct PipelineResult {
    /// Созданный пайплайн
    // pub pipeline: crate::gpu::Pipeline,

    /// Количество разделений буферов (для фалбэка)
    pub split_count: u32,

    /// Используемый размер stride в байтах
    pub used_stride: u32,
    
    /// Попал ли пайплайн в кэш
    pub cache_hit: bool,
}

impl PipelineResult {
    // [MAYBE]
    // Временный метод для разработки, потом удалить 
    pub fn indev() -> Self {
        Self {
            split_count: 0,
            used_stride: 0,
            cache_hit: false,
        }
    }
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

    pub polygon_mode: PolygonMode,
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
            polygon_mode: PolygonMode::Fill,
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

    pub fn polygon_mode(mut self, polygon_mode: PolygonMode) -> Self {
        self.render_config.polygon_mode = polygon_mode;
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

    pub fn build(
        &self,
        context: &mut BackendContext,
        texture_format: BackendTextureFormat,
        bind_group_layouts: &[&RawBindGroupLayout],
    ) -> Result<PipelineResult, MoonBackendError> {
        // Валидация конфигурации
        self.validate()?;

        let cache_key = self.create_cache_key();

        // Если такой пайплайн уже есть в кэше то просто нужно вернуть его,
        // нет смысла создавать новый, только память будет расходывать
        // из-за хака с box::leak
        if let Some(cached) = PIPELINE_CACHE.lock().get(&cache_key).cloned() {
            return Ok(PipelineResult {
                // pipeline: Pipeline {
                //     raw: (*cached).clone()
                // },

                split_count: 1,
                used_stride: self.get_max_stride(),
                cache_hit: true,
            })
        };

        // let result = match self.fallback_strategy {
        //     FallbackStrategy::None => self.build_direct(ctx, wgpu_format, wgpu_bind_groups, 1)?,
        //     FallbackStrategy::Adaptive => self.clone().build_with_fallback(ctx, wgpu_format, wgpu_bind_groups)?,
        //     FallbackStrategy::Split => self.clone().build_with_split(ctx, wgpu_format, wgpu_bind_groups)?,
        //     FallbackStrategy::Reduce => self.clone().build_with_reduce(ctx, wgpu_format, wgpu_bind_groups)?,
        // };

        Ok(PipelineResult::indev())
    }
    
    // [MAYBE]
    // Собрать все параметры и RenderConfig в raw gpu пайплайн
    // Это легаси с утечкой абстрации, использовать только метод compile
    // upd: Всё таки я сделаю breken change, build не буде принимать
    // wgpu типы
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

    /// Валидация параметров пайплайна перед сборкой
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

    /// Метод для получения ключа кэширования в системе кэша, оптимизация.
    /// Зачем пересоздавать пайплайн если он уже есть в кэше?
    fn create_cache_key(&self) -> PipelineCacheKey {
        let mut hasher = DefaultHasher::new();
        
        // Хэширование шейдер
        self.shader_source.hash(&mut hasher);
        self.vertex_entry.hash(&mut hasher);
        self.fragment_entry.hash(&mut hasher);

        let shader_hash = hasher.finish();

        // Хэширование vertex layouts
        let mut hasher = DefaultHasher::new();
        for layout in &self.vertex_layouts {
            layout.stride.hash(&mut hasher);
            (layout.step_mode as u8).hash(&mut hasher);
            
            for attr in &layout.attributes {
                (attr.format as u8).hash(&mut hasher);
                
                attr.location.hash(&mut hasher);
                attr.offset.hash(&mut hasher);
            }
        }

        let vertex_layouts_hash = hasher.finish();

        // Хэширование bind groups
        let mut hasher = DefaultHasher::new();
        
        for bind_group in &self.bind_groups {
            for entry in &bind_group.entries {
                entry.binding.hash(&mut hasher);
                (entry.entry_type as u8).hash(&mut hasher);
                (entry.visibility as u8).hash(&mut hasher);
                
                if let Some(st) = &entry.sample_type {
                    (*st as u8).hash(&mut hasher);
                }
                
                if let Some(st) = &entry.sampler_type {
                    (*st as u8).hash(&mut hasher);
                }
            }
        }

        let bind_groups_hash = hasher.finish();

        // Хэширование render config
        let mut hasher = DefaultHasher::new();
        (self.render_config.blend_mode as u8).hash(&mut hasher);
        (self.render_config.cull_mode as u8).hash(&mut hasher);
        (self.render_config.topology as u8).hash(&mut hasher);
        self.render_config.depth_test.hash(&mut hasher);
        self.render_config.depth_write.hash(&mut hasher);
        
        let format_hash = hasher.finish();

        PipelineCacheKey {
            shader_hash,
            vertex_layouts_hash,
            bind_groups_hash,
            format_hash,
        }
    }

    /// Прямое создание без фалбэкаa
    fn build_direct(
        &self,
        context: &mut BackendContext,
        format: BackendTextureFormat,
        bind_group_layouts: &[&RawBindGroupLayout],
        split_count: u32,
    ) -> Result<(), MoonBackendError> {
        match &mut context.get_raw() {
            Some(raw_context) => {
                let shader = raw_context.device.create_shader_module(
                    wgpu::ShaderModuleDescriptor {
                        label: Some(&format!("{} shader module", &self.get_label())),
                        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&self.shader_source)),
                    }
                );

                // Этот код приводит к снижению производительности на маппинг,
                // но это необходимо чтобы избежать утечки абстрации
                let temp: Vec<&wgpu::BindGroupLayout> = bind_group_layouts.iter()
                    .map(|rg| &rg.raw).collect();
                
                let wgpu_bind_groups_layouts: &[&wgpu::BindGroupLayout] = &temp;

                let layout = raw_context.device.create_pipeline_layout(
                    &wgpu::PipelineLayoutDescriptor {
                        label: Some(&format!("{} pipeline layout", &self.get_label())),
                        bind_group_layouts: wgpu_bind_groups_layouts,
                        push_constant_ranges: &[],
                    }
                );

                // Конвертация вершинных лайаутов в wgpu и сборка в вектор
                let mut vertex_layouts: Vec<wgpu::VertexBufferLayout> = Vec::new();
                
                for layout in &self.vertex_layouts {
                    let wgpu_layout = self.convert_vertex_layout(layout);
                    vertex_layouts.push(wgpu_layout);
                }

                let raw_pipeline = raw_context.device.create_render_pipeline(
                    &wgpu::RenderPipelineDescriptor {
                        // Форматирование с label для отладки
                        label: Some(&format!("{} render pipeline", &self.get_label())),
                        layout: Some(&layout),
                        
                        vertex: wgpu::VertexState {
                            module: &shader,
                            entry_point: Some(&self.vertex_entry),
                            buffers: &vertex_layouts,
                            compilation_options: Default::default(),
                        },

                        fragment: Some(wgpu::FragmentState {
                            module: &shader,
                            entry_point: Some(&self.fragment_entry),
                            targets: &[Some(wgpu::ColorTargetState {
                                format: map_format_to_wgpu(format),
                                blend: Some(map_blend_state(self.render_config.blend_mode)),
                                
                                write_mask: wgpu::ColorWrites::ALL,
                            })],
                            compilation_options: Default::default(),
                        }),

                        primitive: wgpu::PrimitiveState {
                            topology: map_topology(self.render_config.topology),
                            strip_index_format: None,
                            front_face: wgpu::FrontFace::Ccw,
                            cull_mode: map_cull_mode(self.render_config.cull_mode),
                            polygon_mode: map_polygon_mode(self.render_config.polygon_mode),
                            unclipped_depth: false,
                            conservative: false,
                        },

                        depth_stencil: get_depth_stencil_state(
                            self.render_config.depth_test,
                            self.render_config.depth_write
                        ),
                        
                        multisample: wgpu::MultisampleState::default(),
                        multiview: None,
                        cache: None,
                    }
                );

                Ok(())
            },

            None => Err(MoonBackendError::ContextNotFoundError),
        }
    }

    /// Получить максимальный размер для передачи в шейдер по шине
    fn get_max_stride(&self) -> u32 {
        self.vertex_layouts.iter()
            .map(|l| l.stride)
            .max()
            .unwrap_or(0)
    }

    /// Для сокращения кода чтобы не писать везде match на self.label
    fn get_label(&self) -> String {
        match &self.label {
            Some(label) => label.to_string(),
            None => "Label not found".to_string(),
        }
    }

    // Конвертация в wgpu типы
    fn convert_vertex_layout(&self, layout: &VertexLayout) -> wgpu::VertexBufferLayout<'static> {
        let attributes: Vec<wgpu::VertexAttribute> = layout.attributes
            .iter()
            .map(|attr| wgpu::VertexAttribute {
                format: self.convert_format(attr.format),
                offset: attr.offset as u64,
                shader_location: attr.location,
            })
            .collect();
    
        // [HACK]
        // Используется Box::leak для получения 'static времени жизни
        // Пайплайнов обычно мало (от 5 до 10) и живут всё время приложения
        // В будущем можно использовать arena allocator если понадобится
        // Уточненеи: вершинный лайаут это лайаут структуры входа внутри
        // шейдера, а один пайплайн = один шейдер = один вершинный лайаут 
        let attributes = Box::leak(attributes.into_boxed_slice());
    
        wgpu::VertexBufferLayout {
            array_stride: layout.stride as u64,
            
            step_mode: match layout.step_mode {
                StepMode::Vertex => wgpu::VertexStepMode::Vertex,
                StepMode::Instance => wgpu::VertexStepMode::Instance,
            },

            attributes,
        }
    }

    fn convert_format(&self, format: Format) -> wgpu::VertexFormat {
        match format {
            Format::Float32 => wgpu::VertexFormat::Float32,
            Format::Float32x2 => wgpu::VertexFormat::Float32x2,
            Format::Float32x3 => wgpu::VertexFormat::Float32x3,
            Format::Float32x4 => wgpu::VertexFormat::Float32x4,
            Format::Uint32 => wgpu::VertexFormat::Uint32,
            Format::Uint16x2 => wgpu::VertexFormat::Uint16x2,
            Format::Uint16x4 => wgpu::VertexFormat::Uint16x4,
            Format::Unorm16x4 => wgpu::VertexFormat::Unorm16x4,
            Format::Snorm16x4 => wgpu::VertexFormat::Snorm16x4,
        }
    }
}