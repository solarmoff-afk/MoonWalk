// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use bytemuck::{Pod, Zeroable};

use crate::rendering::batching::common::SortableInstance; 

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

    fn get_type_id(&self) -> u32 {
        self.type_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    /// Тест упаковки цвета
    #[test]
    fn test_pack_color() {
        let white = [1.0, 1.0, 1.0, 1.0];
        let packed = ObjectInstance::pack_color(white);
        
        assert_ne!(packed & 0xFF, 0); // R
        assert_ne!((packed >> 8) & 0xFF, 0); // G
        assert_ne!((packed >> 16) & 0xFF, 0); // B
        assert_ne!((packed >> 24) & 0xFF, 0); // A
        
        // Черный
        let black = [0.0, 0.0, 0.0, 1.0];
        let packed = ObjectInstance::pack_color(black);

        assert_eq!(packed & 0xFF, 0); // R
        assert_eq!((packed >> 8) & 0xFF, 0); // G
        assert_eq!((packed >> 16) & 0xFF, 0); // B
        assert_ne!((packed >> 24) & 0xFF, 0); // A
        
        // Красный
        let red = [1.0, 0.0, 0.0, 1.0];
        let packed = ObjectInstance::pack_color(red);
        assert_ne!(packed & 0xFF, 0); // R должен быть > 0
        assert_eq!((packed >> 8) & 0xFF, 0); // G = 0
        assert_eq!((packed >> 16) & 0xFF, 0); // B = 0
    }

    /// Тест упаковки градиента
    #[test]
    fn test_pack_gradient() {
        // Нулевой градиент
        let zero = [0.0, 0.0, 0.0, 0.0];
        let packed = ObjectInstance::pack_gradient(zero);
        assert_eq!(packed, [0, 0, 0, 0]);
        
        // Максимальные значения (с допуском на округление)
        let max = [1.0, 1.0, 1.0, 1.0];
        let packed = ObjectInstance::pack_gradient(max);
        for &val in packed.iter() {
            assert!(val == i16::MAX || val == i16::MAX); // Допускаем оба варианта
        }
        
        // Минимальные значения
        let min = [-1.0, -1.0, -1.0, -1.0];
        let packed = ObjectInstance::pack_gradient(min);
        for &val in packed.iter() {
            assert!(val == -i16::MAX || val == -i16::MAX); // Допускаем оба варианта
        }
    }

    /// Тест что градиент обрезается до [-1, 1]
    #[test]
    fn test_pack_gradient_clamping() {
        let over = [2.0, 2.0, 2.0, 2.0];
        let packed = ObjectInstance::pack_gradient(over);
        for &val in packed.iter() {
            assert!(val == i16::MAX || val == i16::MAX); // Обрезалось до максимума
        }
        
        let under = [-2.0, -2.0, -2.0, -2.0];
        let packed = ObjectInstance::pack_gradient(under);
        for &val in packed.iter() {
            assert!(val == -i16::MAX || val == -i16::MAX); // Обрезалось до минимума
        }
    }

    /// Тест упаковки радиусов
    #[test]
    fn test_pack_radii() {
        let radii = [10.0, 20.0, 30.0, 40.0];
        let packed = ObjectInstance::pack_radii(radii);
        
        assert!(packed[0] < packed[1]);  // 10 < 20
        assert!(packed[1] < packed[2]);  // 20 < 30
        assert!(packed[2] < packed[3]);  // 30 < 40
        
        // Нулевые радиусы
        let zero = [0.0, 0.0, 0.0, 0.0];
        assert_eq!(ObjectInstance::pack_radii(zero), [0, 0, 0, 0]);
    }

    /// Тест упаковки UV координат
    #[test]
    fn test_pack_uv() {
        // Полная текступа
        let full = [0.0, 0.0, 1.0, 1.0];
        let packed = ObjectInstance::pack_uv(full);

        assert_eq!(packed[0], 0);
        assert_eq!(packed[1], 0);
        assert!(packed[2] >= 65534); // близко к 65535
        assert!(packed[3] >= 65534);
        
        // Половина текстуры
        let half = [0.25, 0.25, 0.5, 0.5];
        let packed = ObjectInstance::pack_uv(half);

        assert!(packed[0] >= 16383 && packed[0] <= 16385);
        assert!(packed[2] >= 32767 && packed[2] <= 32769);
    }

    /// Тест упаковки эффектов
    #[test]
    fn test_pack_effects() {
        let border = 10.0;
        let shadow = 20.0;
        let packed = ObjectInstance::pack_effects(border, shadow);
        
        assert!((packed[0] as f32 / 16.0 - border).abs() < 0.1);
        assert!((packed[1] as f32 / 16.0 - shadow).abs() < 0.1);
        
        // Нулевые эффекты
        assert_eq!(ObjectInstance::pack_effects(0.0, 0.0), [0, 0]);
    }

    /// Тест точности преобразований (round-trip)
    #[test]
    fn test_round_trip_color() {
        let original = [0.1, 0.2, 0.3, 0.4];
        let packed = ObjectInstance::pack_color(original);
        
        // Имитация распаковки в шейдере
        let r = ((packed >> 0) & 0xFF) as f32 / 255.0;
        let g = ((packed >> 8) & 0xFF) as f32 / 255.0;
        let b = ((packed >> 16) & 0xFF) as f32 / 255.0;
        let a = ((packed >> 24) & 0xFF) as f32 / 255.0;
        
        assert_relative_eq!(r, original[0], epsilon = 1.0/255.0);
        assert_relative_eq!(g, original[1], epsilon = 1.0/255.0);
        assert_relative_eq!(b, original[2], epsilon = 1.0/255.0);
        assert_relative_eq!(a, original[3], epsilon = 1.0/255.0);
    }

    /// Тест что структура имеет разумный размер
    #[test]
    fn test_struct_size() {
        let size = std::mem::size_of::<ObjectInstance>();
        
        assert!(size <= 128, "Структура слишком большая: {} байт", size);
        assert!(size >= 32, "Структура слишком маленькая: {} байт", size);
        
        // Проверяем выравнивание
        assert_eq!(size % 4, 0, "Размер {} должен быть кратен 4", size);
    }

    /// Тест получения z-index
    #[test]
    fn test_get_z_index() {
        let mut instance = ObjectInstance {
            pos_size: [0.0; 4],
            uv: [0; 4],
            radii: [0; 4],
            gradient_data: [0; 4],
            extra: [42.0, 0.0],
            color: 0,
            color2: 0,
            type_id: 0,
            effect_data: [0; 2],
        };
        
        assert_eq!(instance.get_z_index(), 42.0);
        
        instance.extra[0] = 24.0;
        assert_eq!(instance.get_z_index(), 24.0);
    }

    /// Тест констант QuadVertex
    #[test]
    fn test_quad_vertex_constants() {
        assert_eq!(QuadVertex::QUAD.len(), 4);
        
        assert_eq!(QuadVertex::QUAD[0].position, [0.0, 0.0]);
        assert_eq!(QuadVertex::QUAD[1].position, [0.0, 1.0]);
        assert_eq!(QuadVertex::QUAD[2].position, [1.0, 1.0]);
        assert_eq!(QuadVertex::QUAD[3].position, [1.0, 0.0]);
        
        assert_eq!(QuadVertex::INDICES, [0, 1, 2, 0, 2, 3]);
    }

    /// Тест граничных значений для всех упаковщиков
    #[test]
    fn test_all_packers_boundaries() {
        // Uv границы
        let packed = ObjectInstance::pack_uv([-1.0, -1.0, 2.0, 2.0]);
        assert_eq!(packed[0], 0);
        assert_eq!(packed[1], 0);
        assert!(packed[2] >= 65534);
        assert!(packed[3] >= 65534);
        
        // Радиусы не должны переполняться
        let huge_radii = [10000.0, 10000.0, 10000.0, 10000.0];
        let packed = ObjectInstance::pack_radii(huge_radii);
        for &val in packed.iter() {
            assert!(val <= u16::MAX);
        }
    }
}
