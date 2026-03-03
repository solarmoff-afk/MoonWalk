// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use std::collections::HashMap;
use std::sync::Arc;

use etagere::{AtlasAllocator, size2};
use image::RgbaImage;

use moonwalk_backend::core::context::BackendContext;
use moonwalk_backend::pipeline::bind::RawBindGroup;
use crate::rendering::state::RenderState;
use crate::text::error::TextError;

use moonpaint::{PositionedGlyph, RasterBackend};
use moonpaint::layout::GlyphFont;

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum FontKey {
    Id(usize),
    Path(std::path::PathBuf),
    Memory(usize),
}

impl From<&GlyphFont> for FontKey {
    fn from(font: &GlyphFont) -> Self {
        match font {
            GlyphFont::User { identifier, .. } => FontKey::Id(*identifier),
            GlyphFont::System { path, .. } => FontKey::Path(path.clone()),
            GlyphFont::Fontique { data, .. } => {
                let ptr = Arc::as_ptr(data) as usize;
                FontKey::Memory(ptr)
            }
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct GlyphKey {
    pub font_key: FontKey, 
    pub glyph_id: u32,
    pub size_bits: u32,
    pub style_index: u16,
}

// Структура для отложенной загрузки глифов на видеокарту
struct PendingUpload {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    data: Vec<u8>,
    bytes_per_row: u32,
}

pub struct GlyphCache {
    pub bind_group: Option<RawBindGroup>,
    packer: AtlasAllocator,
    
    // По ключу глифа можем получить его uv координаты в атласе, оффсет и ширину
    // с высотой
    //
    //  [?] (UV [u, v, uw, vh], OffsetX, OffsetY, Width, Height)
    cache: HashMap<GlyphKey, ([f32; 4], f32, f32, f32, f32)>,
    
    atlas_size: u32,
    real_atlas_id: u32,
    
    scratch_image: RgbaImage,
    pending_uploads: Vec<PendingUpload>,
}

impl GlyphCache {
    pub fn new(atlas_size: u32, real_atlas_id: u32) -> Self {
        Self {
            bind_group: None,
            packer: AtlasAllocator::new(size2(atlas_size as i32, atlas_size as i32)),
            cache: HashMap::with_capacity(2048),
            atlas_size,
            real_atlas_id,
            scratch_image: RgbaImage::new(128, 128),
            pending_uploads: Vec::with_capacity(128),
        }
    }

    pub fn set_bind_group(&mut self, bind_group: RawBindGroup) {
        self.bind_group = Some(bind_group);
    }

    /// Гарантирует наличие глифа в атласе, работает полностью на процессоре,
    /// принимает бэкенд растеризатора
    pub fn ensure_glyph<B: RasterBackend>(
        &mut self,
        backend: &mut B,
        glyph: &PositionedGlyph,
    ) -> Result<([f32; 4], f32, f32, f32, f32), TextError> {
        let key = GlyphKey {
            font_key: FontKey::from(&glyph.font),
            glyph_id: glyph.glyph_identifier,
            size_bits: glyph.size.to_bits(),
            style_index: glyph.style_index,
        };

        if let Some(cached) = self.cache.get(&key) {
            return Ok(*cached);
        }

        // [DONT TOUCH] [HACK] [HARDCODE]
        // Такие крутые модное молодёжные шрифты как Hundo.ttf могут брать куда
        // больше чем нужно в размерах, из-за чего без умножения часть глифов
        // обрезается не там где нужно. 4.0 это хардкод умножения на который
        // хватает чтобы такие шрифты не обрезались слишком рано
        let req_size = (glyph.size * 4.0).ceil() as u32;
        if self.scratch_image.width() < req_size || self.scratch_image.height() < req_size {
            self.scratch_image = RgbaImage::new(req_size, req_size);
        }
        
        // Очищаем временный холст
        self.scratch_image.fill(0);

        // Хардкод 1.5 также нужен для нормального отображения
        let padding_x = (glyph.size * 1.5).ceil();
        let padding_y = (glyph.size * 1.5).ceil();
        
        let mut render_glyph = glyph.clone();
        render_glyph.x = padding_x;
        render_glyph.y = padding_y;
        
        // Белый чтобы шейдер мог красить глиф через умножение
        render_glyph.color = [255, 255, 255, 255]; 

        // Вызов растеризации через FreeType
        backend.render_glyph(&mut self.scratch_image, &render_glyph)
            .map_err(TextError::Paint)?;
 
        let (min_x, min_y, max_x, max_y) = find_bounds(
            &self.scratch_image,
            req_size,
            req_size
        );
        
        let width = if max_x >= min_x {
            max_x - min_x + 1
        } else {
            0
        };

        let height = if max_y >= min_y {
            max_y - min_y + 1
        } else {
            0
        };

        // Если глиф пустой (например пробел)
        if width == 0 || height == 0 {
            let res = ([0.0; 4], 0.0, 0.0, 0.0, 0.0);

            self.cache.insert(key, res);
            return Ok(res);
        }

        // Используется etagere для аллокации в атласе чтобы не изобретать
        // велосипед
        let alloc_pad = 1;
        let alloc_w = (width + alloc_pad * 2) as i32;
        let alloc_h = (height + alloc_pad * 2) as i32;

        let mut alloc = self.packer.allocate(size2(alloc_w, alloc_h));
        if alloc.is_none() {
            // Атлас переполнен, нужно попробовать почистить всё и сбросить очередь.
            // Это вызовет джанк (фриз короче)
            self.packer.clear();
            self.cache.clear();
            self.pending_uploads.clear();

            alloc = self.packer.allocate(size2(alloc_w, alloc_h));
        }

        let alloc = alloc.ok_or(TextError::AtlasFull)?;
        let p = alloc.rectangle.min;
        let atlas_x = (p.x as u32) + alloc_pad;
        let atlas_y = (p.y as u32) + alloc_pad;

        // Подготовка буфера для видеокарты, выравнивание требует wgpu валидатор
        // для rgba текстур
        let bytes_per_pixel = 4;
        let align_mask = 255; // 256 байт выравнивание
        
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let padded_bytes_per_row = (unpadded_bytes_per_row + align_mask) & !align_mask;
        
        let mut upload_buffer = vec![0u8; (padded_bytes_per_row * height) as usize];

        for row in 0..height {
            let src_y = min_y + row;
            let dst_offset = (row * padded_bytes_per_row) as usize;
            
            for col in 0..width {
                let src_x = min_x + col;
                let pixel = self.scratch_image.get_pixel(src_x, src_y);
                let pixel_offset = dst_offset + (col * bytes_per_pixel) as usize;

                upload_buffer[pixel_offset] = pixel[0];
                upload_buffer[pixel_offset + 1] = pixel[1];
                upload_buffer[pixel_offset + 2] = pixel[2];
                upload_buffer[pixel_offset + 3] = pixel[3];
            }
        }

        // Кладем в очередь на загрузку (выполнится в prepare)
        self.pending_uploads.push(PendingUpload {
            x: atlas_x,
            y: atlas_y,
            width,
            height,
            data: upload_buffer,
            bytes_per_row: padded_bytes_per_row,
        });

        // Расчёт uv
        let inv_dim = 1.0 / self.atlas_size as f32;
        let uv = [
            atlas_x as f32 * inv_dim,
            atlas_y as f32 * inv_dim,
            width as f32 * inv_dim,
            height as f32 * inv_dim,
        ];

        let offset_x = min_x as f32 - padding_x;
        let offset_y = min_y as f32 - padding_y;

        let res = (uv, offset_x, offset_y, width as f32, height as f32);
        self.cache.insert(key, res);
        Ok(res)
    }

    /// Вызывается перед рендером, когда есть доступ к контексту и к состоянию
    /// рендеринга (чтобы взять текстуры)
    pub fn upload_pending(
        &mut self,
        context: &mut BackendContext,
        state: &RenderState,
    ) -> Result<(), TextError> {
        if self.pending_uploads.is_empty() {
            return Ok(());
        }

        let texture = state.textures.get(&self.real_atlas_id)
            .ok_or(TextError::TextureNotFound)?;

        for upload in self.pending_uploads.drain(..) {
            // Запись в текстуру через отдельный метод контекста который поддерживает
            // выравнивание, если выравнивание не будет или оно будет неправильным
            // то wgpu валидатор (wgpu используется в moonwalk_backend)вызовет панику
            context.write_texture_aligned(
                texture,
                upload.x,
                upload.y,
                upload.width,
                upload.height,
                &upload.data,
                upload.bytes_per_row,
            ).map_err(TextError::Backend)?;
        }

        Ok(())
    }
}

fn find_bounds(img: &RgbaImage, w: u32, h: u32) -> (u32, u32, u32, u32) {
    let mut min_x = w;
    let mut min_y = h;
    let mut max_x = 0;
    let mut max_y = 0;

    for y in 0..h {
        for x in 0..w {
            if img.get_pixel(x, y)[3] > 0 {
                if x < min_x { min_x = x; }
                if y < min_y { min_y = y; }
                if x > max_x { max_x = x; }
                if y > max_y { max_y = y; }
            }
        }
    }

    (min_x, min_y, max_x, max_y)
}
