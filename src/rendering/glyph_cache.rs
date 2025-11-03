use cosmic_text::{CacheKey, FontSystem, SwashCache, SubpixelBin};
use swash::scale::image::{Content, Image as SwashImage};
use std::collections::HashMap;

const ATLAS_SIZE: u32 = 2048;

#[allow(dead_code)]
pub struct GlyphCache {
    swash_cache: SwashCache,
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    next_x: u32,
    next_y: u32,
    row_height: u32,
    glyphs: HashMap<CacheKey, (SwashImage, (f32, f32, f32, f32))>,
}

impl GlyphCache {
    pub fn new(device: &wgpu::Device) -> Self {
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
            label: Some("Glyph Atlas Texture"),
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
            label: Some("Glyph Bind Group Layout"),
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
            label: Some("Glyph Bind Group"),
        });

        Self {
            swash_cache: SwashCache::new(),
            texture,
            bind_group,
            next_x: 1,
            next_y: 1,
            row_height: 0,
            glyphs: HashMap::new(),
        }
    }

    pub fn get_bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
    
    pub fn get_glyph(&mut self, key: CacheKey, font_system: &mut FontSystem) -> Option<(SwashImage, (f32, f32, f32, f32))> {
        if let Some((image, rect)) = self.glyphs.get(&key) {
            return Some((image.clone(), *rect));
        }

        let image = self.swash_cache.get_image(font_system, key).as_ref().map(|i| (*i).clone())?;
        if image.content != Content::Mask {
            return None;
        }

        self.place_glyph(key, image.clone()).map(|rect| (image, rect))
    }
    
    fn place_glyph(&mut self, key: CacheKey, image: SwashImage) -> Option<(f32, f32, f32, f32)> {
        let w = image.placement.width;
        let h = image.placement.height;
        
        if self.next_x + w + 1 > ATLAS_SIZE {
            self.next_x = 1;
            self.next_y += self.row_height + 1;
            self.row_height = 0;
        }

        if self.next_y + h + 1 > ATLAS_SIZE {
            return None;
        }
        
        let uv_rect = (
            self.next_x as f32 / ATLAS_SIZE as f32,
            self.next_y as f32 / ATLAS_SIZE as f32,
            w as f32 / ATLAS_SIZE as f32,
            h as f32 / ATLAS_SIZE as f32,
        );
        
        self.glyphs.insert(key, (image, uv_rect));
        self.next_x += w + 1;
        self.row_height = self.row_height.max(h);
        
        Some(uv_rect)
    }
}

pub fn get_cache_key(glyph: &cosmic_text::LayoutGlyph) -> CacheKey {
    CacheKey {
        font_id: glyph.cache_key.font_id,
        glyph_id: glyph.cache_key.glyph_id,
        font_size_bits: glyph.cache_key.font_size_bits,
        x_bin: SubpixelBin::new(glyph.x_offset).1,
        y_bin: SubpixelBin::new(glyph.y_offset).1,
    }
}