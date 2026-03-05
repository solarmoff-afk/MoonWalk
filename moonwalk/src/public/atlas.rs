// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use rect_packer::Packer;

use crate::{TextureId, Vec2, Vec4};
use crate::error::MoonWalkError;
use crate::MoonWalk;

/// Атлас это текстура которая создаётся из нескольких текстур. Используется для
/// оптимизиации, так как позволяет не делать отдельные команды отрисовки в
/// батчинге на каждый текстурный объект, а использовать только часть атласа
/// из множества текстур
pub struct MoonAtlas {
    pub raw: TextureId,
    uv: Vec<(f32, f32, f32, f32)>,
}

impl MoonAtlas {
    pub fn new() -> Self {
        Self {
            // Быстрый плейсхолдер
            raw: TextureId::new(0),
            uv: Vec::new(),
        }
    }

    pub fn push_uv(&mut self, uv: (f32, f32, f32, f32)) {
        self.uv.push(uv);
    }

    /// Получить uv координаты по индексу текстуру, тут тот же порядок в котором
    /// передаются текстуры при строительстве атласа
    pub fn get_frame(&self, frame: usize) -> [f32; 4] {
        let frame_uv = self.uv.get(frame).unwrap_or(&(0.0, 0.0, 0.0, 0.0));

        [frame_uv.0, frame_uv.1, frame_uv.2, frame_uv.3]
    }
}

impl MoonWalk {
    pub fn build_texture_atlas(&mut self, textures: Vec<TextureId>, width: u32, height: u32) -> Result<MoonAtlas, MoonWalkError> {
        let config = rect_packer::Config {
            width: width as i32,
            height: height as i32,
            border_padding: 1,
            rectangle_padding: 1,
        };

        let mut packer = Packer::new(config);

        let mut atlas_surface = self.new_surface(width, height)?;
        let mut atlas = MoonAtlas::new();

        for texture in &textures {
            let size = self.get_texture_size(*texture);

            if let Some(rect) = packer.pack(size.x as i32, size.y as i32, false) {
                let x = rect.x;
                let y = rect.y;
                let w = rect.width;
                let h = rect.height;

                let u1 = x as f32 / width as f32;
                let v1 = y as f32 / height as f32;
                let u2 = (x + w) as f32 / width as f32;
                let v2 = (y + h) as f32 / height as f32;

                let rect = atlas_surface.new_rect();
                atlas_surface.set_position(rect, Vec2::new(x as f32, y as f32));
                atlas_surface.set_size(rect, Vec2::new(w as f32, h as f32));
                atlas_surface.set_texture(rect, *texture);

                atlas.push_uv((u1, v1, u2, v2));
            }
        }

        atlas_surface.render(self, Some(Vec4::ZERO));
        atlas.raw = atlas_surface.snapshot(self, Vec2::new(0.0, 0.0), Vec2::new(width as f32, height as f32))?;

        Ok(atlas)
    }
} 
