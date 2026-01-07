// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use glam::{Vec3, Vec4, Mat4};

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

    /// Фильтр который применяет к текстуре хромокей, то есть позволяет заменить
    /// указанный цвет на прозрачный. Принимает айди текстуры у которой будет
    /// заменён цвет, принимает сам цвет (Vec3 из glam, не Vec4 так как прозрачность
    /// в этом контексте не учитывается. Если ваш цвет Vec4 то просто передайте
    /// xyz, а w не передавайте) и силу применения фильтра (f32)
    pub fn chromakey(&mut self, texture_id: u32, key_color: Vec3, tolerance: f32) {
        self.renderer.apply_chromakey(texture_id, key_color.to_array(), tolerance);
    }

    /// Фильтр маски, он позволяет взять две текстуры (одна исходная, вторая маска)
    /// у которой есть прозрачные пиксели и использовать как маску для основной текстуры.
    /// Принимает айди исхожной текстуры, айди маски (у которой будет использовать альфа
    /// канал) и булевый параметр будет ли инвертирована маска. Если не инвертирована
    /// (false), то прозрачные пиксели маски делают пиксели исходной текстуры видимыми,
    /// а непрозрачные скрывает. Если true то прозрачные маски скрывают пиксель, а
    /// непрозрачные делают видимой (классическая маска)
    pub fn apply_mask(&mut self, target_id: u32, mask_id: u32, invert: bool) {
        self.renderer.apply_stencil(target_id, mask_id, invert);
    }

    /// Применяет произвольную цветовую матрицу к текстуре. Это позволяет делать сложные
    /// эффекты типа сепии, инверсии, замены каналов (rgb на bgr) и так далее
    /// Формула: пиксель = матрицы * пиксель + offset. Принимает texture_id что явлется
    /// id текстуры для трансформации, matrix (матрица 4 на 4 из glam) где:
    ///  - Столбцы матрицы отвечают за входные каналы (R, G, B, A)
    ///  - Строки отвечают за выходные каналы.
    /// и offset (4д вектор Vec4 из glam, который прибавляется к результату (смещение цвета)
    pub fn color_matrix(&mut self, texture_id: u32, matrix: Mat4, offset: Vec4) {
        let mat_arr = matrix.to_cols_array_2d();
        let off_arr = offset.to_array();

        self.renderer.apply_color_matrix(texture_id, mat_arr, off_arr);
    }
}
