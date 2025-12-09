// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use cosmic_text::{CacheKey, SwashCache};
use swash::scale::image::{Content, Image as SwashImage};
use std::collections::HashMap;

use crate::textware::font::FontSystem;

const ATLAS_SIZE: u32 = 2048;
const PADDING: u32 = 1;

pub struct GlyphCache {
    swash_cache: SwashCache,
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    next_x: u32,
    next_y: u32,
    row_height: u32,
    glyphs: HashMap<CacheKey, (SwashImage, (f32, f32, f32, f32))>,
    pending_uploads: Vec<(CacheKey, u32, u32, SwashImage)>,
}

impl GlyphCache {
    pub fn new(device: &wgpu::Device, _queue: &wgpu::Queue) -> Self {
        let texture_size = wgpu::Extent3d {
            width: ATLAS_SIZE,
            height: ATLAS_SIZE,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: None,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: None,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: None,
        });

        Self {
            swash_cache: SwashCache::new(),
            texture,
            bind_group,
            next_x: PADDING,
            next_y: PADDING,
            row_height: 0,
            glyphs: HashMap::new(),
            pending_uploads: Vec::new(),
        }
    }

    pub fn get_bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn upload_pending(&mut self, queue: &wgpu::Queue) {
        if self.pending_uploads.is_empty() {
            return;
        }

        for (key, x, y, image) in self.pending_uploads.drain(..) {
            let w = image.placement.width;
            let h = image.placement.height;
            if w == 0 || h == 0 { continue; }

            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x, y, z: 0 },
                    aspect: wgpu::TextureAspect::All,
                },
                &image.data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(w),
                    rows_per_image: None,
                },
                wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            );

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
        
        if image.content != Content::Mask { return None; }

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