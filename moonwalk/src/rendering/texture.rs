// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use crate::gpu::Context;
use crate::error::MoonWalkError;
use image::GenericImageView;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub bind_group: wgpu::BindGroup,
}

impl Texture {
    pub fn from_bytes(ctx: &Context, bytes: &[u8], label: &str) -> Result<Self, MoonWalkError> {
        let img = image::load_from_memory(bytes)
            .map_err(|e| MoonWalkError::TextureLoading(e.to_string()))?;
            
        Self::from_image(ctx, &img, label)
    }

    pub fn from_image(ctx: &Context, img: &image::DynamicImage, label: &str) -> Result<Self, MoonWalkError> {
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        Self::from_raw(ctx, &rgba, dimensions.0, dimensions.1, label)
    }

    pub fn from_raw(ctx: &Context, bytes: &[u8], width: u32, height: u32, label: &str) -> Result<Self, MoonWalkError> {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING 
                 | wgpu::TextureUsages::RENDER_ATTACHMENT 
                 | wgpu::TextureUsages::COPY_SRC
                 | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        ctx.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            label: Some("texture_bind_group_layout"),
        });

        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("texture_bind_group"),
        });

        Ok(Self {
            texture,
            view,
            sampler,
            bind_group,
        })
    }

    /// Создаёт пустую текстуру заданного размера в которую можно копировать данные.
    /// [*] Цель создания данной функции это реализация снапшотов
    pub fn create_empty(ctx: &Context, width: u32, height: u32, format: wgpu::TextureFormat, label: &str) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            
            format: format,

            usage: wgpu::TextureUsages::TEXTURE_BINDING
                 | wgpu::TextureUsages::RENDER_ATTACHMENT
                 | wgpu::TextureUsages::COPY_SRC
                 | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

            label: Some("texture_bind_group_layout"),
        });

        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,

            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],

            label: Some("texture_bind_group"),
        });

        Self {
            texture,
            view,
            sampler,
            bind_group,
        }
    }

    pub fn create_render_target(ctx: &Context, width: u32, height: u32, format: wgpu::TextureFormat) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Offscreen Target"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING 
                 | wgpu::TextureUsages::RENDER_ATTACHMENT 
                 | wgpu::TextureUsages::COPY_SRC
                 | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float {
                            filterable: true
                        },
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
            
            label: Some("texture_bind_group_layout"),
        });

        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
            
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            
            label: Some("texture_bind_group"),
        });

        Self {
            texture,
            view,
            sampler,
            bind_group,
        }
    }

    pub fn create_depth_texture(ctx: &crate::gpu::Context, width: u32, height: u32, label: &str) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        
        let texture = ctx.device.create_texture(&desc);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            compare: Some(wgpu::CompareFunction::LessEqual),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let layout = ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Depth,
                    },
                    count: None,
                },

                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
            ],
            label: Some("Depth Texture Layout"),
        });

        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },

                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Depth Bind Group"),
        });

        Self {
            texture,
            view,
            sampler,
            bind_group,
        }
    }

    /// Скачивает данные текстуры из видеокарты в озу и возвращает буфер изображения
    /// Эта операция блокирующая и относительно медленная
    pub fn download(&self, ctx: &crate::gpu::Context) -> Result<image::RgbaImage, crate::MoonWalkError> {
        let device = &ctx.device;
        let queue = &ctx.queue;

        let width = self.texture.width();
        let height = self.texture.height();
        
        // wgpu требует выравнивания байтов в строке по 256
        let bytes_per_pixel = 4;
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = (unpadded_bytes_per_row + align - 1) / align * align;

        let buffer_size = (padded_bytes_per_row * height) as wgpu::BufferAddress;

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Download buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Download encoder"),
        });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },

            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: None,
                },
            },

            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        queue.submit(Some(encoder.finish()));

        let slice = buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        
        slice.map_async(wgpu::MapMode::Read, move |res| {
            tx.send(res).unwrap();
        });

        // Блокировка потока пока wgpu не кончит (:3)
        device.poll(wgpu::Maintain::Wait);
        
        rx.recv()
            .map_err(|_| crate::MoonWalkError::IOError("Failed to map buffer".to_string()))?
            .map_err(|e| crate::MoonWalkError::IOError(e.to_string()))?;

        let data = slice.get_mapped_range();
        let mut pixels: Vec<u8> = Vec::with_capacity((width * height * 4) as usize);

        for chunk in data.chunks(padded_bytes_per_row as usize) {
            pixels.extend_from_slice(&chunk[..unpadded_bytes_per_row as usize]);
        }

        drop(data);
        buffer.unmap();

        image::RgbaImage::from_raw(width, height, pixels)
            .ok_or_else(|| crate::MoonWalkError::IOError("Failed to create image buffer".to_string()))
    }

    pub fn read_pixel(&self, ctx: &crate::gpu::Context, x: u32, y: u32) -> Result<[u8; 4], crate::MoonWalkError> {
        let device = &ctx.device;
        let queue = &ctx.queue;

        // Проверка границ для безопасности
        if x >= self.texture.width() || y >= self.texture.height() {
            return Err(crate::MoonWalkError::IOError("Pixel coordinates out of bounds".to_string()));
        }

        // Wgpu требует чтобы bytes_per_row был кратен 256 даже если нужно всего 4 байта
        // нужно выделить и настроить копирование под 256
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u64;
        let buffer_size = align; 

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Single Pixel Read Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Pixel Read Encoder"),
        });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },

            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(256),
                    rows_per_image: None,
                },
            },

            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        queue.submit(Some(encoder.finish()));

        // Как обычно ожидание пока wgpu кончит
        let slice = buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        
        slice.map_async(wgpu::MapMode::Read, move |res| {
            tx.send(res).unwrap();
        });

        device.poll(wgpu::Maintain::Wait);
        
        rx.recv()
            .map_err(|_| crate::MoonWalkError::IOError("Failed to map pixel buffer".to_string()))?
            .map_err(|e| crate::MoonWalkError::IOError(e.to_string()))?;

        let data = slice.get_mapped_range();
        let pixel = [data[0], data[1], data[2], data[3]];
        
        drop(data);
        buffer.unmap();

        Ok(pixel)
    }
}
