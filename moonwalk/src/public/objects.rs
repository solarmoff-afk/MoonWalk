// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use glam::{Vec2, Vec4, Mat4};

use crate::core::objects::ObjectId;
use crate::{MoonWalk, FontAsset, PathBuilder, TextAlign};
use crate::draw::text::FontId;

impl MoonWalk {
    /// [WAIT DOC]
    pub fn new_path_builder(&self) -> PathBuilder {
        PathBuilder::new()
    }

    /// [WAIT DOC]
    pub fn parse_svg_path(&self, pb: &mut crate::draw::path::PathBuilder, data: &str) -> Result<(), String> {
        crate::draw::path::svg::parse_svg_path(pb, data)
    }
    
    /// Эта функция нужна для получения размеров текста. Принимает контент строкой, шрифт
    /// (FontAsset, его можно получить через функцию load_font и load_font_from_bytes)
    /// размер шрифта и максимальную ширину. На выходе идёт Vec2 из крейта glam который
    /// содержит ширину и высоту текста по указанным параметрам
    pub fn measure_text(&mut self, text: &str, font: FontAsset, size: f32, max_width: f32) -> Vec2 {
        let (w, h) = self.renderer.text_engine.measure_text(
            text, 
            FontId(font.0 as usize), 
            size, 
            max_width
        );
        
        Vec2::new(w, h)
    } 
}
