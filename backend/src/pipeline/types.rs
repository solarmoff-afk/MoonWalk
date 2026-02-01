// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

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

/// Режим смешивания цветов
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlendMode {
    /// Без смешивания, заменяет содержимое
    None,
    /// Альфа-смешивание с учетом прозрачности
    Alpha,
    /// Аддитивное смешивание
    Additive,
    /// Мультипликативное смешивание
    Multiply,

    Screen,
    Subtract,
    Eraser,
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

#[derive(Debug, Clone, Copy)]
pub enum PolygonMode {
    Fill = 1,
}

pub fn map_blend_state(blend_mode: BlendMode) -> wgpu::BlendState {
    match blend_mode {
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
        },

        BlendMode::Screen => wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrc,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent::OVER,
        },

        BlendMode::Subtract => wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::ReverseSubtract,
            },

            alpha: wgpu::BlendComponent::OVER,
        },

        BlendMode::Eraser => wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Zero,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },

            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Zero,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
        },
    }
}

pub fn map_topology(topology: Topology) -> wgpu::PrimitiveTopology {
    match topology {
        Topology::PointList => wgpu::PrimitiveTopology::PointList,
        Topology::LineList => wgpu::PrimitiveTopology::LineList,
        Topology::LineStrip => wgpu::PrimitiveTopology::LineStrip,
        Topology::TriangleList => wgpu::PrimitiveTopology::TriangleList,
        Topology::TriangleStrip => wgpu::PrimitiveTopology::TriangleStrip,
    }
}

pub fn map_cull_mode(cull_mode: CullMode) -> Option<wgpu::Face> {
    match cull_mode {
        CullMode::None => None,
        CullMode::Front => Some(wgpu::Face::Front),
        CullMode::Back => Some(wgpu::Face::Back),
    }
}

pub fn map_polygon_mode(polygon_mode: PolygonMode) -> wgpu::PolygonMode {
    match polygon_mode {
        PolygonMode::Fill => wgpu::PolygonMode::Fill,
    }
}

pub fn get_depth_stencil_state(
    depth_test: bool,
    depth_write: bool
) -> Option<wgpu::DepthStencilState> {
    if depth_test {
        return Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: depth_write,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        });
    }

    None
}

pub fn format_size_bytes(format: Format) -> u32 {
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