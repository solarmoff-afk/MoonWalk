// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

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