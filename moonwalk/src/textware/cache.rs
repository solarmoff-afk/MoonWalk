// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use cosmic_text::{CacheKey, SwashCache};

use moonwalk_backend::{
    core::context::BackendContext,
    pipeline::bind::RawBindGroup,
    render::texture::BackendTexture
};
use moonwalk_backend::render::texture::BackendTextureFormat;
use moonwalk_backend::pipeline::types::{TextureType, SamplerType};
use moonwalk_backend::pipeline::bind::BindGroup;

use swash::scale::image::{Content, Image as SwashImage};
use std::collections::HashMap;

use crate::textware::font::FontSystem;
use crate::MoonWalkError;

const ATLAS_SIZE: u32 = 2048;
const PADDING: u32 = 1;

pub struct GlyphCache {
    swash_cache: SwashCache,
    texture: BackendTexture,
    next_x: u32,
    next_y: u32,
    row_height: u32,
    glyphs: HashMap<CacheKey, (SwashImage, (f32, f32, f32, f32))>,
    pending_uploads: Vec<(CacheKey, u32, u32, SwashImage)>,
}

impl GlyphCache {
    pub fn new(context: &mut BackendContext) -> Result<Self, MoonWalkError> {
        let mut texture = BackendTexture::new(ATLAS_SIZE, ATLAS_SIZE);
        texture.config.set_format(BackendTextureFormat::R8Unorm);
        
        texture.create_render_target(context, ATLAS_SIZE, ATLAS_SIZE)?;
    
        // Создаём структуру с texture и None
        Ok(Self {
            swash_cache: SwashCache::new(),
            texture,
            next_x: PADDING,
            next_y: PADDING,
            row_height: 0,
            glyphs: HashMap::new(),
            pending_uploads: Vec::new(),
        })
    }
    
    pub fn get_bind_group(&self) -> Result<&RawBindGroup, MoonWalkError> {
        self.texture.get_raw_bind_group().ok_or(MoonWalkError::BindGroupNotFoundError)
    }
    
    pub fn upload_pending(&mut self, context: &mut BackendContext) {
        if self.pending_uploads.is_empty() {
            return;
        }

        for (key, x, y, image) in self.pending_uploads.drain(..) {
            let w = image.placement.width;
            let h = image.placement.height;
            
            if w == 0 || h == 0 {
                continue;
            }

            context.write_texture(&self.texture, x, y, w, h, &image.data);

            let uv_rect = (
                x as f32 / ATLAS_SIZE as f32,
                y as f32 / ATLAS_SIZE as f32,
                w as f32 / ATLAS_SIZE as f32,
                h as f32 / ATLAS_SIZE as f32,
            );
            self.glyphs.insert(key, (image, uv_rect));
        }
    }

    pub fn get_glyph(&mut self, key: CacheKey, font_system: &mut FontSystem) -> Option<(SwashImage, (f32, f32, f32, f32))> {
        if let Some((image, rect)) = self.glyphs.get(&key) {
            return Some((image.clone(), *rect));
        }

        let image = self.swash_cache.get_image(&mut font_system.sys, key).clone()?;
        
        if image.content != Content::Mask {
            return None;
        }

        let rect = self.place_glyph(key, image.clone())?;
        Some((image, rect))
    }

    fn place_glyph(&mut self, key: CacheKey, image: SwashImage) -> Option<(f32, f32, f32, f32)> {
        let w = image.placement.width;
        let h = image.placement.height;

        if self.next_x + w + PADDING > ATLAS_SIZE {
            self.next_x = PADDING;
            self.next_y += self.row_height + PADDING;
            self.row_height = 0;
        }

        if self.next_y + h + PADDING > ATLAS_SIZE {
            return None;
        }

        let x = self.next_x;
        let y = self.next_y;

        self.pending_uploads.push((key, x, y, image));
        self.next_x += w + PADDING;
        self.row_height = self.row_height.max(h);

        Some((
            x as f32 / ATLAS_SIZE as f32,
            y as f32 / ATLAS_SIZE as f32,
            w as f32 / ATLAS_SIZE as f32,
            h as f32 / ATLAS_SIZE as f32,
        ))
    }
}

pub fn get_cache_key(glyph: &cosmic_text::PhysicalGlyph) -> CacheKey {
    glyph.cache_key
}
