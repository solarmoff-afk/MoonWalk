// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

// Этот файл реализует публичный API библиотеки обёртки для cosmic-text и swash.
// нужна для упрощения использлвания и батчинга текста в основном движке MoonWalk.
// Изначально библиотека планировалась как отдельная зависимость, но было принято
// решение слить в основной движок как модуль.

mod error;
mod font;
mod cache;

pub use error::TextError;
pub use font::{FontSystem, FontId};
pub use cache::GlyphCache;
pub use cosmic_text::{Attrs, Color as CosmicColor, Metrics, Weight, Family, Wrap, Align};

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct TextVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

/// Структура для хранения текста. Хранит вершины текста и индексные буферы
pub struct TextMesh {
    pub vertices: Vec<TextVertex>,
    pub indices: Vec<u16>,
}

/// Основная структура для этого модуля, хранит кэщ и шрифтовую систему 
pub struct TextWare {
    font_system: FontSystem,
    glyph_cache: GlyphCache,
}

/// Сам текст, его цвет, айди шрифта, цвет и cosmic-text буфер
pub struct Text {
    pub buffer: cosmic_text::Buffer,
    pub color: [f32; 4],
    font_id: Option<FontId>, 
}

impl TextWare {
    #[cfg(not(target_os = "android"))]
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        Self {
            font_system: FontSystem::new(),
            glyph_cache: GlyphCache::new(device, queue),
        }
    }

    /// Специфичная функция new для андроид, разделение нужно
    /// из-за необходимости использовать AssetManager на ОС андроид
    #[cfg(target_os = "android")]
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, asset_manager: ndk::asset::AssetManager) -> Self {
        Self {
            font_system: FontSystem::new(asset_manager),
            glyph_cache: GlyphCache::new(device, queue),
        }
    }

    /// Данная функция нужна для статического добавления шрифта в проект,
    /// не предназначена для FFI. 
    pub fn load_font_bytes(&mut self, data: &[u8], name: &str) -> Result<FontId, TextError> {
        self.font_system.load_font_from_bytes(data, name)
    }

    pub fn load_font_file(&mut self, path: &str) -> Result<FontId, TextError> {
        self.font_system.load_font(path)
    }

    pub fn create_text(&mut self, content: &str, font_id: Option<FontId>, font_size: f32, line_height: Option<f32>) -> Text {
        let metrics = Metrics::new(font_size, line_height.unwrap_or(font_size * 1.2));
        let mut buffer = cosmic_text::Buffer::new(&mut self.font_system.sys, metrics);
        
        let mut attrs = Attrs::new();
        
        let family_name = if let Some(id) = font_id {
            self.font_system.get_family_name(id).cloned()
        } else {
            None
        };

        if let Some(name) = family_name.as_ref() {
            attrs = attrs.family(Family::Name(name.as_str()));
        }

        buffer.set_text(&mut self.font_system.sys, content, attrs, cosmic_text::Shaping::Advanced);
        
        Text {
            buffer,
            color: [1.0, 1.0, 1.0, 1.0], // Белый цвет как дефолт
            font_id,
        }
    }

    pub fn update_text(&mut self, text: &mut Text, content: &str) {
        let mut attrs = Attrs::new();
        
        let family_name = if let Some(id) = text.font_id {
            self.font_system.get_family_name(id).cloned()
        } else {
            None
        };

        if let Some(name) = family_name.as_ref() {
            attrs = attrs.family(Family::Name(name.as_str()));
        }

        text.buffer.set_text(&mut self.font_system.sys, content, attrs, cosmic_text::Shaping::Advanced);
    }

    pub fn resize_text(&mut self, text: &mut Text, font_size: f32, line_height: Option<f32>) {
        let metrics = Metrics::new(font_size, line_height.unwrap_or(font_size * 1.2));
        text.buffer.set_metrics(&mut self.font_system.sys, metrics);
    }

    pub fn set_size(&mut self, text: &mut Text, width: Option<f32>, height: Option<f32>) {
        let w = width.unwrap_or(f32::MAX);
        let h = height.unwrap_or(f32::MAX);
        text.buffer.set_size(&mut self.font_system.sys, w, h);
    }

    pub fn set_wrap(&mut self, text: &mut Text, wrap: Wrap) {
        text.buffer.set_wrap(&mut self.font_system.sys, wrap);
    }

    pub fn prepare(&mut self, queue: &wgpu::Queue) {
        self.glyph_cache.upload_pending(queue);
    }

    pub fn get_bind_group(&self) -> &wgpu::BindGroup {
        self.glyph_cache.get_bind_group()
    }

    pub fn generate_mesh(&mut self, text: &mut Text) -> TextMesh {
        text.buffer.shape_until_scroll(&mut self.font_system.sys, false);

        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut index_count = 0;

        for run in text.buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                let physical = glyph.physical((0., 0.), 1.0);
                
                let key = cache::get_cache_key(&physical);

                if let Some((image, uv_rect)) = self.glyph_cache.get_glyph(key, &mut self.font_system) {
                    let left = image.placement.left as f32;
                    let top = image.placement.top as f32;
                    let w = image.placement.width as f32;
                    let h = image.placement.height as f32;

                    let x = physical.x as f32 + left;
                    let y = run.line_y + physical.y as f32 - top;

                    let (u, v, uw, vh) = uv_rect;
                    let c = text.color;
                    let z = 0.0;

                    vertices.push(TextVertex { position: [x, y, z], uv: [u, v], color: c });
                    vertices.push(TextVertex { position: [x, y + h, z], uv: [u, v + vh], color: c });
                    vertices.push(TextVertex { position: [x + w, y + h, z], uv: [u + uw, v + vh], color: c });
                    vertices.push(TextVertex { position: [x + w, y, z], uv: [u + uw, v], color: c });

                    indices.extend_from_slice(&[
                        index_count, index_count + 1, index_count + 2,
                        index_count, index_count + 2, index_count + 3,
                    ]);
                    index_count += 4;
                }
            }
        }

        TextMesh { vertices, indices }
    }
}