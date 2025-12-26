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
/// 3: uv (x, y, w, h)
/// 4: extra (z, rotation)
/// 5: color запакованный в u32 в (r, g, b, a) (красный, зелёный, синий и альфв канал)
/// 6: type_id, тут либо 0 либо айди текстуры
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ObjectInstance {
    pub pos_size:       [f32; 4],
    pub uv:             [u16; 4],
    pub radii:          [u16; 4],
    pub gradient_data:  [i16; 4],
    pub extra:          [f32; 2],
    pub color2:         u32,
    pub color:          u32,
    pub type_id:        u32,
    pub effect_data:    [u16; 2],
}

impl ObjectInstance {
    // let dummy = [ObjectInstance { 
    //     pos_size: [0.0; 4],
    //     uv: [0; 4],
    //     radii: [0; 4],
    //     gradient_data: [0; 4], 
    //     extra: [0.0; 2],
    //     color: 0,
    //     color2: 0,
    //     type_id: 0, 
    // }];

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

    /// Эта функция запаковывает градиент [x, y, радиус, радиус] в массив из 4 i16,
    /// это критически необходимо так как лимит 86 байт на передачу данных в шейдер
    pub fn pack_gradient(data: [f32; 4]) -> [i16; 4] {
        [
            (data[0].clamp(-1.0, 1.0) * 32767.0) as i16,
            (data[1].clamp(-1.0, 1.0) * 32767.0) as i16,
            (data[2].clamp(-1.0, 1.0) * 32767.0) as i16,
            (data[3].clamp(-1.0, 1.0) * 32767.0) as i16,
        ]
    }

    /// Функция для упаковки скругления углов из f32;4 в u16;4, что
    /// позволяет экономить 8 байт что достаточно много, учитывая,
    /// что лимит (Для железа на котором ведётся тестирование как минимуи)
    /// 86 байт.
    pub fn pack_radii(r: [f32; 4]) -> [u16; 4] {
        [
            (r[0] * 16.0) as u16,
            (r[1] * 16.0) as u16,
            (r[2] * 16.0) as u16,
            (r[3] * 16.0) as u16,
        ]
    }

    /// [WAIT DOC]
    pub fn pack_uv(uv: [f32; 4]) -> [u16; 4] {
        [
            (uv[0] * 65535.0) as u16,
            (uv[1] * 65535.0) as u16,
            (uv[2] * 65535.0) as u16,
            (uv[3] * 65535.0) as u16,
        ]
    }

    /// [WAIT DOC]
    pub fn pack_effects(border: f32, shadow: f32) -> [u16; 2] {
        [
            (border * 16.0) as u16,
            (shadow * 16.0) as u16,
        ]
    }
}

impl SortableInstance for ObjectInstance {
    fn get_z_index(&self) -> f32 {
        self.extra[0]
    }
}

// На устройствах со слабым gpu лимит байт на вершину может быть ещё меньше,
// 32 байта. Оптимизировать данные под 32 байта невероятно сложно и долго
// (С точки зрения времени упаковки и распаковки), поэтому нужно разделение
// на 2 инстанса

// Первая часть, 32 байта
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct InstancePartA {
    pub pos_size: [f32; 4], // 16
    pub uv:       [u16; 4], // 8
    pub extra:    [f32; 2], // 8
}

// Вторая часть, 32 байта
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct InstancePartB {
    pub radii:         [u16; 4], // 8
    pub gradient_data: [i16; 4], // 8
    pub color2:        u32,      // 4
    pub color:         u32,      // 4
    pub type_id:       u32,      // 4
    pub effect_data:   [u16; 2], // 4
}