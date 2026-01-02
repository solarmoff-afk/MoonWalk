// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use crate::MoonWalk;

impl MoonWalk {
    /// Эта функция перезаписывает текстуру применив к ней блюр по гаусу. Для
    /// правильного блюра обязательно применить блюр дважды, первый раз
    /// с horizontal true (горизонтальный проход), второй раз с horizontal false
    /// (вертикальный проход). Принимает айди текстуры, радиус блюра и направление
    pub fn blur_texture(&mut self, texture_id: u32, radius: f32, horizontal: bool) {
        self.renderer.apply_blur(texture_id, radius, horizontal);
    }

    /// Эта функция перезаписывает текстуру обновив её яркость. Принимает айди текстуры
    /// и значение яркости. 0.0 - нулевая яркость, 1.0 - стандартная (как в оригинале),
    /// 2.0 - в два раза ярче
    pub fn brightness(&mut self, texture_id: u32, factor: f32) {
        let (matrix, offset) = crate::filters::color_matrix::matrix_brightness(factor);
        self.renderer.apply_color_matrix(texture_id, matrix, offset);
    }
    
    /// Эта функция перезаписывает текстуру обновив её контраст. Принимает айди текстуры
    /// и новое значение контраста
    pub fn contrast(&mut self, texture_id: u32, contrast: f32) {
        let (matrix, offset) = crate::filters::color_matrix::matrix_contrast(contrast);
        self.renderer.apply_color_matrix(texture_id, matrix, offset);
    }
    
    /// Эта функция перезаписывает текстуру обновив её насыщеность. Принимает айди текстуры
    /// и новое значение насыщености
    pub fn saturation(&mut self, texture_id: u32, sat: f32) {
        let (matrix, offset) = crate::filters::color_matrix::matrix_saturation(sat);
        self.renderer.apply_color_matrix(texture_id, matrix, offset);
    }
    
    /// Эта функция перезаписывает текстуру обновив её тон. Принимает айди текстуры
    /// и градусы для угла поворота цветового колеса (от 0 до 360)
    pub fn hue_shift(&mut self, texture_id: u32, degrees: f32) {
        let (matrix, offset) = crate::filters::color_matrix::matrix_hue(degrees);
        self.renderer.apply_color_matrix(texture_id, matrix, offset);
    }
}