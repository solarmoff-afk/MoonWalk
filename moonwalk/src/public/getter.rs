// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use glam::{Vec2, Vec4};

use crate::MoonWalk;

impl MoonWalk {
    /// Возвращает логические размеры окна (ширина и высота)
    pub fn get_window_size(&mut self) -> Vec2 {
        // [HACK] [UNWRAP]
        let size = self.renderer.context.get_size()
            .expect("Context not found");
        let width = size.x as f32;
        let height = size.y as f32;
        let scale = self.renderer.scale_factor;

        if scale <= 0.0 {
            return Vec2::new(width, height);
        }

        Vec2::new(width / scale, height / scale)
    }

    /// Получение скейл фактора
    pub fn get_scale_factor(&self) -> f32 {
        self.renderer.scale_factor
    }
}
