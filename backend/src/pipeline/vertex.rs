// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use super::types::*;

/// Атрибут вершины
#[derive(Debug, Clone)]
pub struct VertexAttr {
    /// Формат данных атрибута
    pub format: Format,

    /// Индекс локации в шейдере
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

    /// Метод чтобы установить идекс локации в шейдере
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
        // [MAYBE]
        // Формат ошибок String только из-за того, что пайплайн и VertexLayout
        // экспортируются публично и это уже легаси :(

        if self.stride == 0 && !self.attributes.is_empty() {
            return Err("Layout has attributes but stride is 0".to_string());
        }

        let mut max_offset = 0;
        for attr in &self.attributes {
            if attr.offset > max_offset {
                max_offset = attr.offset + VertexLayout::map_format_size(attr.format);
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

    /// Эта функция переводит Format в сырые значения u32
    fn map_format_size(format: Format) -> u32 {
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