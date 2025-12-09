// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use crate::easy_gpu::context::Context;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub width: u32,
    pub height: u32,
    pub format: wgpu::TextureFormat,
}

impl Texture {
    pub fn from_bytes(
        ctx: &Context,
        data: &[u8],
        width: u32,
        height: u32,
        format: wgpu::TextureFormat
    ) -> Result<Self, String> {
        let block_size = get_block_size(format).ok_or_else(|| 
            format!("Format {:?} is not supported for auto-upload", format)
        )?;

        let bytes_per_row = width * block_size;
        let expected_size = (bytes_per_row * height) as usize;

        if data.len() != expected_size {
            return Err(format!(
                "Size mismatch. Expected {}, got {}.",
                expected_size, data.len()
            ));
        }

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        ctx.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Ok(Self { texture, view, sampler, width, height, format })
    }

    pub fn create_render_target(
        ctx: &Context,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat
    ) -> Self {
        let size = wgpu::Extent3d { width, height, depth_or_array_layers: 1 };
        let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Render Target"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Self { texture, view, sampler, width, height, format }
    }
}

const fn get_block_size(format: wgpu::TextureFormat) -> Option<u32> {
    use wgpu::TextureFormat::*;
    match format {
        R8Unorm | R8Snorm | R8Uint | R8Sint => Some(1),
        R16Uint | R16Sint | R16Float | Rg8Unorm | Rg8Snorm | Rg8Uint | Rg8Sint => Some(2),
        R32Uint | R32Sint | R32Float | Rg16Uint | Rg16Sint | Rg16Float | 
        Rgba8Unorm | Rgba8UnormSrgb | Bgra8Unorm | Bgra8UnormSrgb | 
        Rgba8Snorm | Rgba8Uint | Rgba8Sint | Rgb10a2Unorm => Some(4),
        Rg32Uint | Rg32Sint | Rg32Float | Rgba16Uint | Rgba16Sint | Rgba16Float => Some(8),
        Rgba32Uint | Rgba32Sint | Rgba32Float => Some(16),
        _ => None,
    }
}