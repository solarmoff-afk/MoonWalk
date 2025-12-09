// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use bytemuck::{Pod, Zeroable};

use crate::batching::common::SortableInstance; 

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct QuadVertex {
    // Позиция вершины в 2D пространстве (Мировая система координат)
    pub position: [f32; 2],
}

impl QuadVertex {
    // Описание констант для прямоугольника. Всегда 4 вершины
    pub const QUAD: [Self; 4] = [
        Self { position: [0.0, 0.0] },
        Self { position: [0.0, 1.0] },
        Self { position: [1.0, 1.0] },
        Self { position: [1.0, 0.0] },
    ];

    // и 6 индексов.
    pub const INDICES: [u32; 6] = [0, 1, 2, 0, 2, 3];
}

/// Структура для экземпляра прямоугольника. Лайаут:
/// 1: pos_size (x, y, w, h) (координаты x/y и ширина/высота w/h)
/// 2: radii (tl, tr, br, bl) (Верх-лево, верх-право, низ-право, низ-лево)
/// 3: extra (z, rotation)
/// 4: color (r, g, b, a) (красный, зелёный, синий и альфв канал)
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct RectInstance {
    pub pos_size: [f32; 4],
    pub radii:    [f32; 4],
    pub extra:    [f32; 2],
    pub color:    u32,
}

impl RectInstance {
    /// Оптимизация низкого уровня для экономии
    /// данных который проходят через шину CPU-GPU
    /// Хелпер для упаковки [r, g, b, a] (0.0 - 1.0) в u32 (0xAABBGGRR)
    pub fn pack_color(c: [f32; 4]) -> u32 {
        let r = (c[0] * 255.0) as u32;
        let g = (c[1] * 255.0) as u32;
        let b = (c[2] * 255.0) as u32;
        let a = (c[3] * 255.0) as u32;
        
        // r это младший байт, нужно для WGPU
        (a << 24) | (b << 16) | (g << 8) | r
    }
}

impl SortableInstance for RectInstance {
    fn get_z_index(&self) -> f32 {
        self.extra[0]
    }
}