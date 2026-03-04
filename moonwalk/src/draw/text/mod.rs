// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

// Этот файл реализует публичный API библиотеки обёртки для cosmic-text и swash.
// нужна для упрощения использлвания и батчинга текста в основном движке MoonWalk.
// Изначально библиотека планировалась как отдельная зависимость, но было принято
// решение слить в основной движок как модуль

pub mod error;
pub mod cache;

use std::collections::HashMap;
use std::hash::{Hash, Hasher, DefaultHasher};

use moonwalk_backend::core::context::BackendContext;
use moonwalk_backend::render::texture::BackendTexture;
use moonwalk_backend::pipeline::bind::RawBindGroup;
use moonwalk_backend::render::texture::BackendTextureFormat; 

use crate::rendering::state::RenderState;
use crate::MoonWalkError;

pub use self::error::TextError;
use self::cache::GlyphCache;

use moonpaint::{
    Context as PaintContext,
    FreeTypeBackend,
    FreeTypeConfig,
    DebugBackend,
    Span,
    TextAlign,
};

/// DONT TOUCH / НЕ ТРОГАТЬ
/// В ШЕЙДЕРЕ ИДЁТ ХАРДКОД НА u32::MAX, изменение приведёт к поломке текстовой
/// системы. Это критически важно. u32::MAX в расте это всегда 4294967295,
/// зависимости от разрядности процессора тут нет
pub const ATLAS_ID: u32 = u32::MAX;

/// Айди шрифта
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct FontId(pub usize);

fn hash_input(text: &str, font_id: usize, size: u32, width: u32) -> u64 {
    let mut hasher = DefaultHasher::new();
    
    text.hash(&mut hasher);
    font_id.hash(&mut hasher);
    size.hash(&mut hasher);
    width.hash(&mut hasher);
    
    hasher.finish()
}

#[derive(Clone, Debug)]
pub struct TextBatchItem {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub uv: [f32; 4], 
    pub is_emoji: bool,
}

pub fn map_align(align: u8) -> TextAlign {
    match align {
        0 => TextAlign::Left,
        1 => TextAlign::Center,
        2 => TextAlign::Right,
        3 => TextAlign::Justify,

        // Использую left как выравнивание поумолчанию, так как это стандарт
        // в текстовых редакторах
        _ => TextAlign::Left,
    }
}

pub struct TextWare {
    pub paint_context: PaintContext,
    pub raster_backend: FreeTypeBackend,
    pub glyph_cache: GlyphCache,
    pub atlas_id: Option<u32>,
    
    // Так как из moonpaint::FontId невозможно достать сырой айдишник из структуры
    // (а менять архитектуру я не хочу) тут идёт переводчик местного типа в
    // импортный
    font_map: HashMap<usize, moonpaint::FontId>,

    // Следующий айдишник
    next_font_id: usize,

    // Кэш лайаута на всякий случай
    layout_cache: HashMap<u64, (Vec<TextBatchItem>, f32, f32)>, 
}

impl TextWare {
    pub fn new(
        state: &mut RenderState,
        context: &mut BackendContext,
    ) -> Result<Self, MoonWalkError> {
        let paint_context = PaintContext::new();
        
        // FreeType стандарт индустрии, он используется для растеризации глифов,
        // но требует cmake и cc для сборки, так как написан на C 
        //  [!] Нельзя использовать texware в многопоточке, так как работта с FreeType 
        // однопоточная
        let raster_backend = FreeTypeBackend::new(FreeTypeConfig::default())
            .map_err(|e| MoonWalkError::TextError(TextError::Paint(e)))?;

        // Достаточный размер атласа для повседневного использования. Атлас это
        // текстура в которую запекаются все глифы после растреризации и используется
        // в батчинге как текстура с нужным uv. Если атлас переполнится то
        // он просто будет очищен
        let atlas_size = 4096;

        // Сам атлас, текстура у которой формат Rgba8UnormSrgb для поддержки цветных глифов
        // (эмодзи короче)
        let mut texture = BackendTexture::new(atlas_size, atlas_size);
        texture.config.set_format(BackendTextureFormat::Rgba8UnormSrgb);
        texture.create_render_target(context, atlas_size, atlas_size)?;

        // Текстура добавляется в состояние рендеринга для получения к ней доступа
        let atlas_id = state.add_texture(texture);
        let atlas_texture = state.textures.get(&atlas_id)
            .ok_or(MoonWalkError::TextError(TextError::AtlasCreationFailed))?;
        
        let bind_group = atlas_texture.get_raw_bind_group()
            .ok_or(MoonWalkError::TextError(TextError::BindGroupMissing))?
            .clone();

        let mut glyph_cache = GlyphCache::new(atlas_size, atlas_id);
        glyph_cache.set_bind_group(bind_group);

        Ok(Self {
            paint_context,
            raster_backend,
            glyph_cache,
            atlas_id: Some(ATLAS_ID),
            font_map: HashMap::new(),
            next_font_id: 1,
            layout_cache: HashMap::new(),
        })
    }

    /// Загружает все накопленные данные в видеокарту. Нужно вызывать после подготовки
    /// батчинга и до рендеринга, это очень важно
    pub fn prepare(
        &mut self,
        context: &mut BackendContext,
        state: &RenderState,
    ) -> Result<(), MoonWalkError> {
        // Да, это просто обёртка над upload_pending из структуры кэша чтобы не
        // лезть в кишки на верхнем уровне
        self.glyph_cache.upload_pending(context, state)
            .map_err(MoonWalkError::TextError)
    }

    pub fn get_bind_group(&self) -> Result<&RawBindGroup, MoonWalkError> {
        self.glyph_cache.bind_group.as_ref()
            .ok_or(MoonWalkError::TextError(TextError::BindGroupMissing))
    }

    /// Загрузить шрифт из файла
    ///  [*] В публичном апи не используется, там просто получаются байты из
    ///  ResourceManager и загружаются в load_font_from_bytes, но возможно
    ///  этот метод может быть полезен когда-нибудь, пусть будет
    pub fn load_font(&mut self, path: &str) -> Result<FontId, MoonWalkError> {
        let mp_id = self.paint_context.load_font(path)
            .map_err(|e| MoonWalkError::TextError(TextError::Paint(e)))?;
            
        let my_id = self.next_font_id;
        self.next_font_id += 1;
        
        self.font_map.insert(my_id, mp_id);
        Ok(FontId(my_id))
    }
    
    /// Загрузить шрифт из байтов
    pub fn load_font_bytes(&mut self, data: Vec<u8>) -> Result<FontId, MoonWalkError> {
        let mp_id = self.paint_context.load_font_from_bytes(data)
            .map_err(|e| MoonWalkError::TextError(TextError::Paint(e)))?;
            
        let my_id = self.next_font_id;
        self.next_font_id += 1;
        
        self.font_map.insert(my_id, mp_id);
        Ok(FontId(my_id))
    }

    /// Измеряет текст и возвращает его ширину (первый элемент) и высоту (второй элемент)
    /// Возвращает нули если нет шрифта или если moonpaint не смог применить лайаут к
    /// тексту
    pub fn measure_text(
        &mut self,
        text: &str,
        font_id: FontId,
        font_size: f32,
        max_width: f32,
    ) -> (f32, f32) {
        let mp_font_id = match self.font_map.get(&font_id.0) {
            Some(id) => *id,

            // Если шрифта нет то и текста нет, можно вернуть нули
            None => return (0.0, 0.0),
        };

        // DebugBackend не содержит растеризатора, поэтому он идеален для такого рода
        // операций с текстом
        let mut dummy = self.paint_context.create_canvas(1, 1, DebugBackend::new());
        let span = Span::new(text).font(mp_font_id).font_size(font_size);

        let max_w_opt = if max_width > 10000.0 {
            None
        } else {
            Some(max_width)
        };

        // Если не получается нарисовать текст через DebugBackend то вряд-ли он
        // пройдёт через FreeTypeBackend, а значит текста нет, а значит можно вернуть
        // нули без проблем
        match dummy.draw_text(&[span], 0.0, 0.0, max_w_opt, TextAlign::Left) {
            Ok(layout) => (layout.metrics.width, layout.metrics.height),
            Err(_) => (0.0, 0.0),
        }
    }

    /// Вызывается в батчинге, подготовка данных на процессоре
    pub fn process_text_batch(
        &mut self,
        id: u64,
        text: &str,
        font_id: FontId,
        font_size: f32,
        max_width: f32,
        align: TextAlign,
    ) -> Result<&Vec<TextBatchItem>, MoonWalkError> {
        let _hash = hash_input(text, font_id.0, font_size.to_bits(), max_width.to_bits());

        if self.layout_cache.contains_key(&id) {
             return Ok(&self.layout_cache.get(&id).unwrap().0);
        }

        let mp_font_id = self.font_map.get(&font_id.0)
            .ok_or(MoonWalkError::TextError(TextError::FontNotFound(font_id.0)))?;

        let mut layout_canvas = self.paint_context.create_canvas(1, 1, DebugBackend::new());
        let span = Span::new(text).font(*mp_font_id).font_size(font_size);
        
        let max_w_opt = if max_width > 10000.0 {
            None
        } else {
            Some(max_width)
        };
        
        // Несмотря на название draw_text через DebugBackend никакой отрисовки не происходит,
        // это нужно чисто для компоновки (лайаута) текста чтобы использовать эти данные
        // потом
        let layout = layout_canvas.draw_text(
            &[span],
            0.0, 0.0, 
            max_w_opt,
            align
        ).map_err(|e| MoonWalkError::TextError(TextError::Paint(e)))?;

        let mut batch_items = Vec::with_capacity(layout.glyphs.len());

        // Строим элементы для батчинга чтобы ему было легче применить их
        // к прямоугольнику и получить текст
        for glyph in layout.glyphs {
            if glyph.codepoint.is_whitespace() {
                continue;
            }

            if let Ok((uv_rect, off_x, off_y, w, h)) = self.glyph_cache.ensure_glyph(
                &mut self.raster_backend,
                &glyph,
            ) {
                if w > 0.0 && h > 0.0 {
                    let screen_x = glyph.x + off_x;
                    let screen_y = glyph.y + off_y;

                    batch_items.push(TextBatchItem {
                        x: screen_x,
                        y: screen_y,
                        w,
                        h,
                        uv: uv_rect,
                        is_emoji: glyph.is_emoji,
                    });
                }
            }
        }

        // Добавление кэша
        self.layout_cache.insert(
            id,
            (batch_items, layout.metrics.width, layout.metrics.height)
        );
        
        Ok(&self.layout_cache.get(&id).unwrap().0)
    }
}
