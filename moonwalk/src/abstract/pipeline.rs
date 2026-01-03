// abstract/pipeline.rs
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;

use crate::gpu::Context;
use crate::error::MoonWalkError;
use crate::gpu::Pipeline; 

// Глобальный кэш пайплайнов который необходим чтобы предотвратить перекомпиляцию
// уже существующего пайплайна
lazy_static::lazy_static! {
    static ref PIPELINE_CACHE: Mutex<HashMap<PipelineCacheKey, Arc<wgpu::RenderPipeline>>> = 
        Mutex::new(HashMap::new());
}

/// Структура для кэширования пайплайна, включает device_id на всякий случай (пайплайны 
/// действительны только на одной видеокарте поэтому кэшируем и её айди)
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct PipelineCacheKey {
    shader_hash: u64,
    vertex_layouts_hash: u64,
    bind_groups_hash: u64,
    format_hash: u64,
    device_id: u64,
}

/// Формат данных для вершинных атрибутов
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
    /// 32-битное число с плавающей точкой (4 байта)
    Float32,
    /// 2 компонента по 32 бита с плавающей точкой (8 байт)
    Float32x2,
    /// 3 компонента по 32 бита с плавающей точкой (12 байт)
    Float32x3,
    /// 4 компонента по 32 бита с плавающей точкой (16 байт)
    Float32x4,
    /// 32-битное беззнаковое целое (4 байта)
    Uint32,
    /// 2 компонента по 16 бит беззнаковых целых (4 байта)
    Uint16x2,
    /// 4 компонента по 16-бит беззнаковых целых (8 байт)
    Uint16x4,
    /// 4 компонента по 16 бит нормализованных беззнаковых целых (8 байт)
    Unorm16x4,
    /// 4 компонента по 16 бит нормализованных знаковых целых (8 байт)
    Snorm16x4, 
}

/// Режим шага для вершинных буферов
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepMode {
    /// Данные читаются для каждой вершины
    Vertex,
    /// Данные читаются для каждого инстанса
    Instance,
}

/// Этапы шейдера в которых доступен ресурс
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderStage {
    /// Только вершинный шейдер
    Vertex,
    /// Только фрагментный шейдер
    Fragment,
    /// Оба шейдера
    Both,
}

/// Тип ресурса в bind group
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BindEntryType {
    /// Uniform буфер
    Uniform,
    /// Текстура
    Texture,
    /// Сэмплер
    Sampler,
    /// Storage буфер только для чтения
    StorageRead,
    /// Storage буфер для записи
    StorageWrite,
}

/// Тип семплинга текстуры
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureType {
    /// Текстура с числами с плавающей точкой
    Float,
    /// Текстура глубины
    Depth,
    /// Текстура с беззнаковыми целыми
    Uint,
}

/// Тип фильтрации сэмплера
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SamplerType {
    /// Линейная фильтрация
    Linear,
    /// Ближайший сосед
    Nearest,
    /// Сравнительный сэмплер для теней
    Comparison,
}

/// Режим смешивания цветов
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    /// Без смешивания, заменяет содержимое
    None,
    /// Альфа-смешивание с учетом прозрачности
    Alpha,
    /// Аддитивное смешивание
    Additive,
    /// Мультипликативное смешивание
    Multiply,
}

/// Режим отсечения граней для 3d графики. Это позволяет оптимизировать 3d сцены,
/// так как грани внутри объектов можно не рисовать (если выбран Back)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CullMode {
    /// Без отсечения
    None,
    /// Отсекать лицевые грани
    Front,
    /// Отсекать задние грани
    Back,
}

/// Топология примитивов
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Topology {
    /// Список точек
    PointList,
    /// Список линий
    LineList,
    /// Полоса линий
    LineStrip,
    /// Список треугольников
    TriangleList,
    /// Полоса треугольников
    TriangleStrip,
}

/// Результат создания пайплайна
#[derive(Debug, Clone)]
pub struct PipelineResult {
    /// Созданный пайплайн
    pub pipeline: crate::gpu::Pipeline,
    /// Количество разделений буферов (для фалбэка)
    pub split_count: u32,
    /// Используемый размер stride в байтах
    pub used_stride: u32,
    /// Попал ли пайплайн в кэш
    pub cache_hit: bool,
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

/// Атрибут вершины
#[derive(Debug, Clone)]
pub struct VertexAttr {
    /// Формат данных атрибута
    pub format: Format,
    /// Индекс location в шейдере
    pub location: u32,
    /// Смещение в байтах от начала вершины
    pub offset: u32,
}

impl VertexAttr {
    pub fn new() -> Self {
        Self {
            format: Format::Float32,
            location: 0,
            offset: 0,
        }
    }

    /// Этот метод нужен чтобы установить формат данных
    pub fn format(mut self, format: Format) -> Self {
        self.format = format;
        self
    }

    /// Метод чтобы установить идекс location в шейдере
    pub fn location(mut self, location: u32) -> Self {
        self.location = location;
        self
    }

    /// Мктод чтобы установить смещение от начала вершины
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = offset;
        self
    }
}

impl Default for VertexAttr {
    fn default() -> Self {
        Self::new()
    }
}

/// Лайаут вершинных атрибутов
#[derive(Debug, Clone)]
pub struct VertexLayout {
    /// Размер одной вершины в байтах
    pub stride: u32,
    /// Режим шага для этого буфера
    pub step_mode: StepMode,
    /// Список атрибутов в этом буфере
    pub attributes: Vec<VertexAttr>,
}

impl VertexLayout {
    pub fn new() -> Self {
        Self {
            stride: 0,
            step_mode: StepMode::Vertex,
            attributes: Vec::new(),
        }
    }

    pub fn stride(mut self, stride: u32) -> Self {
        self.stride = stride;
        self
    }

    pub fn step_mode(mut self, mode: StepMode) -> Self {
        self.step_mode = mode;
        self
    }

    pub fn add_attr(mut self, attr: VertexAttr) -> Self {
        self.attributes.push(attr);
        self
    }

    // Метод для валидации лайаута
    pub fn validate(&self) -> Result<(), String> {
        if self.stride == 0 && !self.attributes.is_empty() {
            return Err("Layout has attributes but stride is 0".to_string());
        }

        let mut max_offset = 0;
        for attr in &self.attributes {
            if attr.offset > max_offset {
                max_offset = attr.offset + VertexLayout::format_size(attr.format);
            }
        }

        if max_offset > self.stride {
            return Err(format!(
                "Attributes exceed stride: max_offset={}, stride={}",
                max_offset, self.stride
            ));
        }

        Ok(())
    }

    fn format_size(format: Format) -> u32 {
        match format {
            Format::Float32 => 4,
            Format::Float32x2 => 8,
            Format::Float32x3 => 12,
            Format::Float32x4 => 16,
            Format::Uint32 => 4,
            Format::Uint16x2 => 4,
            Format::Uint16x4 => 8,
            Format::Unorm16x4 => 8,
            Format::Snorm16x4 => 8,
        }
    }
}

impl Default for VertexLayout {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BindGroupEntry {
    /// Индекс binding
    pub binding: u32,
    /// Тип ресурса
    pub entry_type: BindEntryType,
    /// В каких шейдерах доступен
    pub visibility: ShaderStage,
    /// Тип текстуры (только для Texture)
    pub sample_type: Option<TextureType>,
    /// Тип сэмплера (только для Sampler)
    pub sampler_type: Option<SamplerType>,
}

/// Описание bind group
#[derive(Debug, Clone)]
pub struct BindGroup {
    /// Список элементов в bind group
    pub entries: Vec<BindGroupEntry>,
}

impl BindGroup {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add_uniform(mut self, binding: u32, visibility: ShaderStage) -> Self {
        self.entries.push(BindGroupEntry {
            binding,
            entry_type: BindEntryType::Uniform,
            visibility,
            sample_type: None,
            sampler_type: None,
        });

        self
    }

    pub fn add_texture(mut self, binding: u32, sample_type: TextureType) -> Self {
        self.entries.push(BindGroupEntry {
            binding,
            entry_type: BindEntryType::Texture,
            visibility: ShaderStage::Fragment,
            sample_type: Some(sample_type),
            sampler_type: None,
        });

        self
    }

    pub fn add_sampler(mut self, binding: u32, sampler_type: SamplerType) -> Self {
        self.entries.push(BindGroupEntry {
            binding,
            entry_type: BindEntryType::Sampler,
            visibility: ShaderStage::Fragment,
            sample_type: None,
            sampler_type: Some(sampler_type),
        });

        self
    }

    pub fn add_storage(mut self, binding: u32, read_only: bool, visibility: ShaderStage) -> Self {
        self.entries.push(BindGroupEntry {
            binding,
            
            entry_type: if read_only {
                BindEntryType::StorageRead
            } else {
                BindEntryType::StorageWrite
            },

            visibility,
            sample_type: None,
            sampler_type: None,
        });

        self
    }

    pub(crate) fn build(&self, ctx: &Context) -> Result<wgpu::BindGroupLayout, MoonWalkError> {
        let entries: Vec<wgpu::BindGroupLayoutEntry> = self.entries
            .iter()
            .map(|entry| {
                let visibility = match entry.visibility {
                    ShaderStage::Vertex => wgpu::ShaderStages::VERTEX,
                    ShaderStage::Fragment => wgpu::ShaderStages::FRAGMENT,
                    ShaderStage::Both => wgpu::ShaderStages::VERTEX_FRAGMENT,
                };

                let ty = match entry.entry_type {
                    BindEntryType::Uniform => wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },

                    BindEntryType::Texture => wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: match entry.sample_type.unwrap_or(TextureType::Float) {
                            TextureType::Float => wgpu::TextureSampleType::Float { filterable: true },
                            TextureType::Depth => wgpu::TextureSampleType::Depth,
                            TextureType::Uint => wgpu::TextureSampleType::Uint,
                        },
                    },

                    BindEntryType::Sampler => wgpu::BindingType::Sampler(
                        match entry.sampler_type.unwrap_or(SamplerType::Linear) {
                            SamplerType::Linear => wgpu::SamplerBindingType::Filtering,
                            SamplerType::Nearest => wgpu::SamplerBindingType::NonFiltering,
                            SamplerType::Comparison => wgpu::SamplerBindingType::Comparison,
                        }
                    ),

                    BindEntryType::StorageRead => wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },

                    BindEntryType::StorageWrite => wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },

                };

                wgpu::BindGroupLayoutEntry {
                    binding: entry.binding,
                    visibility,
                    ty,
                    count: None,
                }
            })
            .collect();

        let layout = ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &entries,
        });

        Ok(layout)
    }
}

impl Default for BindGroup {
    fn default() -> Self {
        Self::new()
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
    /// Включить тест глубины
    pub depth_test: bool,
    /// Включить запись глубины
    pub depth_write: bool,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            target_format: Format::Float32x4, // Placeholder
            blend_mode: BlendMode::None,
            cull_mode: CullMode::None,
            topology: Topology::TriangleList,
            depth_test: false,
            depth_write: false,
        }
    }
}

pub struct MoonPipeline {
    /// Исходный код шейдера
    shader_source: String,
    /// Точка входа вершинного шейдера
    vertex_shader: String,
    /// Точка входа фрагментного шейдера
    fragment_shader: String,
    /// Список вершинных лайаутов
    vertex_layouts: Vec<VertexLayout>,
    /// Список bind groups
    bind_groups: Vec<BindGroup>,
    /// Конфигурация рендера
    render_config: RenderConfig,
    /// Стратегия фалбэка
    fallback_strategy: FallbackStrategy,
    /// Метка для отладки
    label: Option<String>,
}

impl MoonPipeline {
    pub fn new(shader_source: &str) -> Self {
        Self {
            shader_source: shader_source.to_string(),
            vertex_shader: "vs_main".to_string(),
            fragment_shader: "fs_main".to_string(),
            vertex_layouts: Vec::new(),
            bind_groups: Vec::new(),
            render_config: RenderConfig::default(),
            fallback_strategy: FallbackStrategy::Adaptive,
            label: None,
        }
    }

    /// Метод чтобы установить точку входа вершинного шейдера
    pub fn vertex_shader(mut self, entry: &str) -> Self {
        self.vertex_shader = entry.to_string();
        self
    }

    /// Метол чтобы установить точку входа фрагментного шейдера
    pub fn fragment_shader(mut self, entry: &str) -> Self {
        self.fragment_shader = entry.to_string();
        self
    }

    /// Метод чтобы добавить vertex layout
    pub fn add_vertex_layout(mut self, layout: VertexLayout) -> Self {
        self.vertex_layouts.push(layout);
        self
    }

    /// Добавить bind group
    pub fn add_bind_group(mut self, bind_group: BindGroup) -> Self {
        self.bind_groups.push(bind_group);
        self
    }

    /// Установить формат цели рендера
    pub fn target_format(mut self, format: Format) -> Self {
        self.render_config.target_format = format;
        self
    }

    /// Установить режим блендинга
    pub fn blend(mut self, mode: BlendMode) -> Self {
        self.render_config.blend_mode = mode;
        self
    }

    /// Установить режим отсечения граней
    pub fn cull(mut self, mode: CullMode) -> Self {
        self.render_config.cull_mode = mode;
        self
    }

    /// Установить топологию примитивов
    pub fn topology(mut self, topology: Topology) -> Self {
        self.render_config.topology = topology;
        self
    }

    /// Включить тест глубины
    pub fn depth_test(mut self, enabled: bool) -> Self {
        self.render_config.depth_test = enabled;
        self
    }

    /// Включить запись глубины
    pub fn depth_write(mut self, enabled: bool) -> Self {
        self.render_config.depth_write = enabled;
        self
    }

    /// Установить стратегию фалбек
    pub fn fallback_strategy(mut self, strategy: FallbackStrategy) -> Self {
        self.fallback_strategy = strategy;
        self
    }

    /// Установить метку для отладки
    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    /// Создать стандартный лайаут для прямоугольников (64 байта) он является
    /// специфичным для стандартного батчинга (UberBatch) мунволка и
    /// может устареть. При изменении сигнатуры shape.wgsl нужно изменить
    /// и этот метод
    pub fn create_rect_instance_layout() -> VertexLayout {
        VertexLayout::new()
            .stride(64)
            .step_mode(StepMode::Instance)
            .add_attr(VertexAttr::new()
                .format(Format::Float32x4)
                .location(1)
                .offset(0))
            .add_attr(VertexAttr::new()
                .format(Format::Unorm16x4)
                .location(2)
                .offset(16))
            .add_attr(VertexAttr::new()
                .format(Format::Uint16x4)
                .location(3)
                .offset(24))
            .add_attr(VertexAttr::new()
                .format(Format::Snorm16x4)
                .location(4)
                .offset(32))
            .add_attr(VertexAttr::new()
                .format(Format::Float32x2)
                .location(5)
                .offset(40))
            .add_attr(VertexAttr::new()
                .format(Format::Uint32)
                .location(6)
                .offset(48))
            .add_attr(VertexAttr::new()
                .format(Format::Uint32)
                .location(7)
                .offset(52))
            .add_attr(VertexAttr::new()
                .format(Format::Uint32)
                .location(8)
                .offset(56))
            .add_attr(VertexAttr::new()
                .format(Format::Uint16x2)
                .location(9)
                .offset(60))
    }

    /// Метод чтобы собрать пайплайн
    pub fn build(
        &self,
        ctx: &Context,
        wgpu_format: wgpu::TextureFormat,
        wgpu_bind_groups: &[&wgpu::BindGroupLayout],
    ) -> Result<PipelineResult, MoonWalkError> {
        // Валидация конфигурации
        self.validate()?;

        let cache_key = self.create_cache_key(ctx);

        if let Some(cached) = PIPELINE_CACHE.lock().get(&cache_key).cloned() {
            return Ok(PipelineResult {
                pipeline: Pipeline { raw: (*cached).clone() },
                split_count: 1,
                used_stride: self.get_max_stride(),
                cache_hit: true,
            });
        }

        let result = match self.fallback_strategy {
            FallbackStrategy::None => self.build_direct(ctx, wgpu_format, wgpu_bind_groups, 1)?,
            FallbackStrategy::Adaptive => self.clone().build_with_fallback(ctx, wgpu_format, wgpu_bind_groups)?,
            FallbackStrategy::Split => self.clone().build_with_split(ctx, wgpu_format, wgpu_bind_groups)?,
            FallbackStrategy::Reduce => self.clone().build_with_reduce(ctx, wgpu_format, wgpu_bind_groups)?,
        };

        // Кэширование результата
        if let Some(label) = &self.label {
            log::debug!("Caching pipeline: {}", label);
        }

        PIPELINE_CACHE.lock().insert(cache_key, Arc::new(result.pipeline.raw.clone()));

        Ok(result)
    }

    /// Метод для валидации конфигурации
    fn validate(&self) -> Result<(), MoonWalkError> {
        if self.vertex_shader.is_empty() {
            return Err(MoonWalkError::ShaderError("Vertex shader entry point not set".into()));
        }

        if self.fragment_shader.is_empty() {
            return Err(MoonWalkError::ShaderError("Fragment shader entry point not set".into()));
        }

        if self.vertex_layouts.is_empty() {
            return Err(MoonWalkError::ShaderError("No vertex layouts specified".into()));
        }

        // Проверка уникальности shader locations
        let mut locations = std::collections::HashSet::new();
        for layout in &self.vertex_layouts {
            if let Err(e) = layout.validate() {
                return Err(MoonWalkError::ShaderError(format!("Invalid vertex layout: {}", e)));
            }

            for attr in &layout.attributes {
                if !locations.insert(attr.location) {
                    return Err(MoonWalkError::ShaderError(
                        format!("Duplicate shader location: {}", attr.location)
                    ));
                }
            }
        }

        // Проверка уникальность bindings
        let mut bindings = std::collections::HashSet::new();
        for bind_group in &self.bind_groups {
            for entry in &bind_group.entries {
                let key = (entry.binding, entry.entry_type);
                if !bindings.insert(key) {
                    return Err(MoonWalkError::ShaderError(
                        format!("Duplicate binding: {}", entry.binding)
                    ));
                }
            }
        }

        Ok(())
    }

    fn create_cache_key(&self, ctx: &Context) -> PipelineCacheKey {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();
        
        // Хэширование шейдер
        self.shader_source.hash(&mut hasher);
        self.vertex_shader.hash(&mut hasher);
        self.fragment_shader.hash(&mut hasher);
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

        // Получение айди устройства (видеокарты)
        // [HACK]
        // Получить айди нельщя, но мы можем за него адрес в памяти
        let device_ptr = &ctx.device as *const wgpu::Device;
        let device_id = device_ptr as usize as u64;

        PipelineCacheKey {
            shader_hash,
            vertex_layouts_hash,
            bind_groups_hash,
            format_hash,
            device_id: device_id.try_into().unwrap(),
        }
    }

    fn get_max_stride(&self) -> u32 {
        self.vertex_layouts.iter()
            .map(|l| l.stride)
            .max()
            .unwrap_or(0)
    }

    // Прямое создание без фалбэка
    fn build_direct(
        &self,
        ctx: &Context,
        wgpu_format: wgpu::TextureFormat,
        wgpu_bind_groups: &[&wgpu::BindGroupLayout],
        split_count: u32,
    ) -> Result<PipelineResult, MoonWalkError> {
        use crate::gpu::PipelineBuilder;

        let mut builder = PipelineBuilder::new(ctx, &self.shader_source); 

        // Конвертация vertex layouts
        for layout in &self.vertex_layouts {
            let wgpu_layout = self.convert_vertex_layout(layout);
            builder = builder.add_layout(wgpu_layout);
        }

        // Конвертация bind groups
        let converted_bind_groups = self.convert_bind_groups(ctx);
        let mut all_bind_groups: Vec<&wgpu::BindGroupLayout> = 
            converted_bind_groups.iter().collect();
        
        all_bind_groups.extend_from_slice(wgpu_bind_groups);

        // Настройка рендера
        builder = self.apply_render_config(builder);

        // Создание пайплайна
        let pipeline = builder.build(wgpu_format, &all_bind_groups);

        Ok(PipelineResult {
            pipeline,
            split_count,
            used_stride: self.get_max_stride(),
            cache_hit: false,
        })
    }

    /// Создание с адаптивным фалбэком
    fn build_with_fallback(
        &self,
        ctx: &Context,
        wgpu_format: wgpu::TextureFormat,
        wgpu_bind_groups: &[&wgpu::BindGroupLayout],
    ) -> Result<PipelineResult, MoonWalkError> {
        let max_stride = ctx.device.limits().max_vertex_buffer_array_stride;
        
        // Если stride в пределах лимитов то создание напрямую
        if self.get_max_stride() <= max_stride {
            return self.build_direct(ctx, wgpu_format, wgpu_bind_groups, 1);
        }

        // разные стратегии fallback
        let strategies = [
            self.try_split_strategy(ctx, wgpu_format, wgpu_bind_groups, max_stride),
            self.try_reduce_strategy(ctx, wgpu_format, wgpu_bind_groups, max_stride),
        ];

        for strategy_result in strategies.iter().flatten() {
            return Ok(strategy_result.clone());
        }

        Err(MoonWalkError::ShaderError(
            "Failed to create pipeline with any fallback strategy".into()
        ))
    }

    // Стратегия разделить layout на несколько
    fn try_split_strategy(
        &self,
        ctx: &Context,
        wgpu_format: wgpu::TextureFormat,
        wgpu_bind_groups: &[&wgpu::BindGroupLayout],
        max_stride: u32,
    ) -> Option<PipelineResult> {
        // Попытка разделить каждый слишком большой layout
        let mut split_layouts = Vec::new();
        let mut split_count = 0;

        for layout in &self.vertex_layouts {
            if layout.stride > max_stride {
                // Деление пополам
                if let Some((a, b)) = self.split_layout_half(layout, max_stride) {
                    split_layouts.push(a);
                    split_layouts.push(b);
                    split_count += 1;
                } else {
                    return None;
                }
            } else {
                split_layouts.push(layout.clone());
            }
        }

        // Создание нового пайплайна с разделенными лайаута
        let mut new_pipeline = self.clone();
        new_pipeline.vertex_layouts = split_layouts;

        new_pipeline.build_direct(ctx, wgpu_format, wgpu_bind_groups, split_count)
            .map(|mut result| {
                result.split_count = split_count;
                result
            })
            .ok()
    }

    // Стратегия уменьшить stride
    fn try_reduce_strategy(
        &self,
        ctx: &Context,
        wgpu_format: wgpu::TextureFormat,
        wgpu_bind_groups: &[&wgpu::BindGroupLayout],
        max_stride: u32,
    ) -> Option<PipelineResult> {
        // Нахождение минимального возможного stride для атрибутов
        let mut reduced_layouts = Vec::new();

        for layout in &self.vertex_layouts {
            let mut new_layout = layout.clone();
            new_layout.stride = new_layout.stride.min(max_stride);
            reduced_layouts.push(new_layout);
        }

        // Создание новых пайплайн с уменьшенным stride
        let mut new_pipeline = self.clone();
        new_pipeline.vertex_layouts = reduced_layouts;

        new_pipeline.build_direct(ctx, wgpu_format, wgpu_bind_groups, 1)
            .ok()
    }

    // Разделить layout пополам
    fn split_layout_half(
        &self,
        layout: &VertexLayout,
        max_stride: u32,
    ) -> Option<(VertexLayout, VertexLayout)> {
        if layout.attributes.len() < 2 {
            return None;
        }

        let mid = layout.attributes.len() / 2;
        let (first, second) = layout.attributes.split_at(mid);

        // Вычисление новые strides
        let first_stride = self.calculate_stride_for_attrs(first);
        let second_stride = self.calculate_stride_for_attrs(second);

        if first_stride > max_stride || second_stride > max_stride {
            // Одна из половин все еще слишком большая
            return None;
        }

        Some((
            VertexLayout {
                stride: first_stride,
                step_mode: layout.step_mode,
                attributes: first.to_vec(),
            },

            VertexLayout {
                stride: second_stride,
                step_mode: layout.step_mode,
                attributes: second.to_vec(),
            },
        ))
    }

    fn calculate_stride_for_attrs(&self, attrs: &[VertexAttr]) -> u32 {
        if attrs.is_empty() {
            return 0;
        }

        let last = attrs.last().unwrap();
        last.offset + self.format_size_bytes(last.format)
    }

    fn format_size_bytes(&self, format: Format) -> u32 {
        match format {
            Format::Float32 => 4,
            Format::Float32x2 => 8,
            Format::Float32x3 => 12,
            Format::Float32x4 => 16,
            Format::Uint32 => 4,
            Format::Uint16x2 => 4,
            Format::Uint16x4 => 8,
            Format::Unorm16x4 => 8,
            Format::Snorm16x4 => 8,
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

    fn convert_bind_groups(&self, ctx: &Context) -> Vec<wgpu::BindGroupLayout> {
        self.bind_groups.iter()
            .map(|bg| self.convert_bind_group(ctx, bg))
            .collect()
    }

    fn convert_bind_group(&self, ctx: &Context, bind_group: &BindGroup) -> wgpu::BindGroupLayout {
        let entries: Vec<wgpu::BindGroupLayoutEntry> = bind_group.entries
            .iter()
            .map(|entry| {
                let visibility = match entry.visibility {
                    ShaderStage::Vertex => wgpu::ShaderStages::VERTEX,
                    ShaderStage::Fragment => wgpu::ShaderStages::FRAGMENT,
                    ShaderStage::Both => wgpu::ShaderStages::VERTEX_FRAGMENT,
                };

                let ty = match entry.entry_type {
                    BindEntryType::Uniform => wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },

                    BindEntryType::Texture => wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: match entry.sample_type.unwrap_or(TextureType::Float) {
                            TextureType::Float => wgpu::TextureSampleType::Float { filterable: true },
                            TextureType::Depth => wgpu::TextureSampleType::Depth,
                            TextureType::Uint => wgpu::TextureSampleType::Uint,
                        },
                    },

                    BindEntryType::Sampler => wgpu::BindingType::Sampler(
                        match entry.sampler_type.unwrap_or(SamplerType::Linear) {
                            SamplerType::Linear => wgpu::SamplerBindingType::Filtering,
                            SamplerType::Nearest => wgpu::SamplerBindingType::NonFiltering,
                            SamplerType::Comparison => wgpu::SamplerBindingType::Comparison,
                        }
                    ),

                    BindEntryType::StorageRead => wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },

                    BindEntryType::StorageWrite => wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                };

                wgpu::BindGroupLayoutEntry {
                    binding: entry.binding,
                    visibility,
                    ty,
                    count: None,
                }
            })
            .collect();

        ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: self.label.as_deref(),
            entries: &entries,
        })
    }

    fn apply_render_config<'a>(&self, builder: crate::gpu::PipelineBuilder<'a>) -> crate::gpu::PipelineBuilder<'a> {
        // Блендинг
        let blend_state = match self.render_config.blend_mode {
            BlendMode::None => wgpu::BlendState::REPLACE,
            BlendMode::Alpha => wgpu::BlendState::ALPHA_BLENDING,
            BlendMode::Additive => wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent::OVER,
            },
            BlendMode::Multiply => {
                wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::Dst,
                        dst_factor: wgpu::BlendFactor::Zero,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent::OVER,
                }
            }
        };

        let mut builder = builder.blend_state(blend_state);

        // Отсечение граней
        let cull_mode = match self.render_config.cull_mode {
            CullMode::None => None,
            CullMode::Front => Some(wgpu::Face::Front),
            CullMode::Back => Some(wgpu::Face::Back),
        };
        builder = builder.with_cull_mode(cull_mode);

        // Топология
        let topology = match self.render_config.topology {
            Topology::PointList => wgpu::PrimitiveTopology::PointList,
            Topology::LineList => wgpu::PrimitiveTopology::LineList,
            Topology::LineStrip => wgpu::PrimitiveTopology::LineStrip,
            Topology::TriangleList => wgpu::PrimitiveTopology::TriangleList,
            Topology::TriangleStrip => wgpu::PrimitiveTopology::TriangleStrip,
        };
        builder = builder.with_topology(topology);

        // Тест глубины
        if self.render_config.depth_test {
            let depth_stencil = wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: self.render_config.depth_write,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            };
            builder = builder.depth_stencil_state(depth_stencil);
        }

        builder
    }

    fn build_with_split(
        self,
        ctx: &Context,
        wgpu_format: wgpu::TextureFormat,
        wgpu_bind_groups: &[&wgpu::BindGroupLayout],
    ) -> Result<PipelineResult, MoonWalkError> {
        // Реализация сплит стратегии
        self.try_split_strategy(ctx, wgpu_format, wgpu_bind_groups, ctx.device.limits().max_vertex_buffer_array_stride)
            .ok_or_else(|| MoonWalkError::ShaderError("Split strategy failed".into()))
    }

    fn build_with_reduce(
        self,
        ctx: &Context,
        wgpu_format: wgpu::TextureFormat,
        wgpu_bind_groups: &[&wgpu::BindGroupLayout],
    ) -> Result<PipelineResult, MoonWalkError> {
        // Реализация reduce стратегии
        self.try_reduce_strategy(ctx, wgpu_format, wgpu_bind_groups, ctx.device.limits().max_vertex_buffer_array_stride)
            .ok_or_else(|| MoonWalkError::ShaderError("Reduce strategy failed".into()))
    }

    /// Очистить кэш пайплайнов для всех устройств
    pub fn clear_pipeline_cache() {
        PIPELINE_CACHE.lock().clear();
    }

    /// Очистить кэш для конкретного устройства
    pub fn clear_pipeline_cache_for_device(device_id: u64) {
        PIPELINE_CACHE.lock().retain(|k, _| k.device_id != device_id);
    }
}

impl Clone for MoonPipeline {
    fn clone(&self) -> Self {
        Self {
            shader_source: self.shader_source.clone(),
            vertex_shader: self.vertex_shader.clone(),
            fragment_shader: self.fragment_shader.clone(),
            vertex_layouts: self.vertex_layouts.clone(),
            bind_groups: self.bind_groups.clone(),
            render_config: self.render_config.clone(),
            fallback_strategy: self.fallback_strategy,
            label: self.label.clone(),
        }
    }
}
