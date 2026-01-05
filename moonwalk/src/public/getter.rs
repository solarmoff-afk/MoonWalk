// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use glam::{Vec2, Vec4};

use crate::objects::ObjectId;
use crate::{MoonWalk, FontAsset, PathBuilder, TextAlign};

impl MoonWalk {
    pub fn get_position(&self, id: ObjectId) -> Vec2 {
        self.renderer.state.store.get_position(id)
    }

    /// Получить текущий размер объекта (Vec2 из glam)
    pub fn get_size(&self, id: ObjectId) -> Vec2 {
        self.renderer.state.store.get_size(id)
    }

    /// Получить текущий угол поворота (в радианах)
    pub fn get_rotation(&self, id: ObjectId) -> f32 {
        self.renderer.state.store.get_rotation(id)
    }

    /// Получить основной цвет объекта (Vec4 из glam)
    pub fn get_color(&self, id: ObjectId) -> Vec4 {
        self.renderer.state.store.get_color(id)
    }

    /// Получить вторичный цвет объекта (Vec4 из glam)
    pub fn get_color2(&self, id: ObjectId) -> Vec4 {
        self.renderer.state.store.get_color2(id)
    }

    /// Получить z индекс объекта
    pub fn get_z_index(&self, id: ObjectId) -> f32 {
        self.renderer.state.store.get_z_index(id)
    }

    /// Получить группу коллизий объекта (hit group)
    pub fn get_hit_group(&self, id: ObjectId) -> u16 {
        self.renderer.state.store.get_hit_group(id)
    }

    /// Получить радиусы скругления (только для прямоугольника)
    pub fn get_rounded(&self, id: ObjectId) -> Vec4 {
        self.renderer.state.store.get_rounded(id)
    }

    /// Получить текстовое содержимое (только для текста)
    pub fn get_text(&self, id: ObjectId) -> &str {
        self.renderer.state.store.get_text(id)
    }

    /// Получить размер шрифта
    pub fn get_font_size(&self, id: ObjectId) -> f32 {
        self.renderer.state.store.get_font_size(id)
    }

    /// Получить границы текста (bounds)
    pub fn get_text_size(&self, id: ObjectId) -> Vec2 {
        self.renderer.state.store.get_text_bounds(id)
    }

    /// Получить выравнивание текста
    pub fn get_text_align(&self, id: ObjectId) -> TextAlign {
        let val = self.renderer.state.store.get_text_align(id);
        match val {
            0 => TextAlign::Left,
            1 => TextAlign::Center,
            2 => TextAlign::Right,
            3 => TextAlign::Justified,
            _ => TextAlign::Left,
        }
    }
    
    /// Проверить жив ли объект
    pub fn is_alive(&self, id: ObjectId) -> bool {
        self.renderer.state.store.is_alive(id)
    }

    /// Возвращает логические размеры окна (ширина и высота)
    pub fn get_window_size(&self) -> Vec2 {
        let width = self.renderer.context.config.width as f32;
        let height = self.renderer.context.config.height as f32;
        let scale = self.renderer.scale_factor;

        if scale <= 0.0 {
            return Vec2::new(width, height);
        }

        Vec2::new(width / scale, height / scale)
    }
}
