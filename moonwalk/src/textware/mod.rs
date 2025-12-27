// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

// Этот файл реализует публичный API библиотеки обёртки для cosmic-text и swash.
// нужна для упрощения использлвания и батчинга текста в основном движке MoonWalk.
// Изначально библиотека планировалась как отдельная зависимость, но было принято
// решение слить в основной движок как модуль.

#![allow(dead_code)]
#![allow(unused_imports)]

pub mod cache;
mod error;
mod font;

pub use error::TextError;
pub use font::{FontSystem, FontId};
pub use cache::GlyphCache;
pub use cosmic_text::{Attrs, Metrics, Family, Wrap};

use bytemuck::{Pod, Zeroable};
use cosmic_text::Shaping;
use std::collections::HashMap;

/// DONT TOUCH / НЕ ТРОГАТЬ
/// В ШЕЙДЕРЕ ИДЁТ ХАРДКОД НА u32::MAX, изменение приведёт к поломке текстовой
/// системы. Это критически важно
pub const ATLAS_ID: u32 = u32::MAX;

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

/// [WAIT DOC]
struct BufferState {
    text_hash: u64,
    font_id: u64,
    font_size_bits: u32,
    bounds_bits: (u32, u32),
    align_byte: u8,
}

/// Основная структура для этого модуля, хранит кэщ и шрифтовую систему 
pub struct TextWare {
    pub atlas_id: Option<u32>,
    pub font_system: FontSystem,
    pub glyph_cache: GlyphCache,
    buffers: HashMap<u64, (cosmic_text::Buffer, BufferState)>,
    scratch_buffer: cosmic_text::Buffer,
}

/// Сам текст, его цвет, айди шрифта, цвет и cosmic-text буфер
pub struct Text {
    pub buffer: cosmic_text::Buffer,
    pub color: [f32; 4],
    font_id: Option<FontId>,
}

/// Необходимо кэшировать строку для оптимизации 
fn hash_str(s: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

impl TextWare {
    #[cfg(not(target_os = "android"))]
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let mut font_system = FontSystem::new();

        let scratch_buffer = cosmic_text::Buffer::new(
            &mut font_system.sys, 
            Metrics::new(24.0, 24.0)
        ); 

        Self {
            atlas_id: Some(ATLAS_ID),
            font_system,
            glyph_cache: GlyphCache::new(device, queue),
            buffers: HashMap::new(),
            scratch_buffer,
        }
    }

    /// Специфичная функция new для андроид, разделение нужно
    /// из-за необходимости использовать AssetManager на ОС андроид
    #[cfg(target_os = "android")]
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, asset_manager: ndk::asset::AssetManager) -> Self {
        let mut font_system = FontSystem::new(asset_manager);

        let scratch_buffer = cosmic_text::Buffer::new(
            &mut font_system.sys, 
            Metrics::new(24.0, 24.0)
        );

        Self {
            atlas_id: Some(ATLAS_ID),
            font_system,
            glyph_cache: GlyphCache::new(device, queue),
            buffers: HashMap::new(),
            scratch_buffer,
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

    pub fn get_bind_group(&self) -> wgpu::BindGroup {
        self.glyph_cache.get_bind_group().clone()
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

    /// Подготавливает текст (layout) и возвращает буфер с глифами.
    /// Используется в UberBatch.
    pub fn process_text(
        &mut self,
        id: u64,
        text: &str,
        font_id: FontId,
        font_size: f32,
        max_width: f32,
        max_height: f32,
        align: u8,
    ) -> &cosmic_text::Buffer {
        let current_state = BufferState {
            text_hash: hash_str(text),
            font_id: font_id.0,
            font_size_bits: font_size.to_bits(),
            bounds_bits: (max_width.to_bits(), max_height.to_bits()),
            align_byte: align,
        };

        let entry = self.buffers.entry(id).or_insert_with(|| {
            let font_system = &mut self.font_system.sys;
            let buffer = cosmic_text::Buffer::new(font_system, Metrics::new(font_size, font_size));

            (buffer, BufferState { 
                text_hash: 0, font_id: u64::MAX, font_size_bits: 0, bounds_bits: (0,0), align_byte: 255 
            })
        });
        
        let (buffer, cached_state) = entry;

        let changed = 
            cached_state.text_hash != current_state.text_hash ||
            cached_state.font_id != current_state.font_id ||
            cached_state.font_size_bits != current_state.font_size_bits ||
            cached_state.bounds_bits != current_state.bounds_bits;

        if changed {
            let family_name_str = if let Some(name) = self.font_system.get_family_name(font_id) {
                 Some(name.clone())
            } else {
                 None
            };

            let font_system = &mut self.font_system.sys;
            
            let metrics = Metrics::new(font_size, font_size * 1.2);
            buffer.set_metrics(font_system, metrics);
            buffer.set_size(font_system, max_width, max_height); 
            
            let mut attrs = Attrs::new();
            if let Some(name) = family_name_str.as_deref() {
                 attrs = attrs.family(Family::Name(name));
            }

            buffer.set_text(font_system, text, attrs, Shaping::Advanced);

            let cosmic_align = match align {
                1 => Some(cosmic_text::Align::Center),
                2 => Some(cosmic_text::Align::End),
                3 => Some(cosmic_text::Align::Justified),
                _ => Some(cosmic_text::Align::Left), // 0
            };

            for line in buffer.lines.iter_mut() {
                line.set_align(cosmic_align);
            }

            buffer.shape_until_scroll(font_system, false);
             
            *cached_state = current_state;
        }

        buffer
    }

    pub fn collect_glyphs(
        &mut self, 
        id: u64, 
        text: &str,
        font_id: FontId,
        font_size: f32,
        max_width: f32,
        max_height: f32,
        align: u8,
    ) -> Vec<(f32, f32, cosmic_text::CacheKey)> {
        self.process_text(id, text, font_id, font_size, max_width, max_height, align);

        let buffer = self.buffers.get(&id).expect("Buffer not found after process");

        let mut glyphs = Vec::new();
        
        for run in buffer.0.layout_runs() {
            for glyph in run.glyphs {
                let physical = glyph.physical((0., 0.), 1.0);
                let x = glyph.x; 
                let y = run.line_y + physical.y as f32;
                glyphs.push((x, y, physical.cache_key));
            }
        }
        
        glyphs
    }

    pub fn measure_text(
        &mut self,
        text: &str,
        font_id: FontId,
        font_size: f32,
        max_width: f32,
    ) -> (f32, f32) {
        let family_name_str = if let Some(name) = self.font_system.get_family_name(font_id) {
             Some(name.clone())
        } else {
             None
        };

        let font_system = &mut self.font_system.sys;
        let buffer = &mut self.scratch_buffer;

        let metrics = Metrics::new(font_size, font_size * 1.2);
        let line_height = metrics.line_height;

        buffer.set_metrics(font_system, metrics);

        // Высота f32::MAX чтобы измерить полный текст без обрезки
        buffer.set_size(font_system, max_width, f32::MAX); 
        
        let mut attrs = Attrs::new();
        if let Some(name) = family_name_str.as_deref() {
             attrs = attrs.family(Family::Name(name));
        }

        // Шейпинг (Это очень дорогая часть, но она критически необходима)
        buffer.set_text(font_system, text, attrs, Shaping::Advanced);
        buffer.shape_until_scroll(font_system, false);

        let mut width = 0.0f32;
        let mut height = 0.0f32;

        for run in buffer.layout_runs() {
            width = width.max(run.line_w);
            height = height.max(run.line_y + line_height);
        }
        
        // Округляем вверх ceil т. к. рендеринг может требовать целых пикселей,
        // но тут возвращается флоат для точности
        (width, height)
    }
}
