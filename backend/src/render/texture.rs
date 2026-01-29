// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use crate::core::context::BackendContext;
use crate::error::MoonBackendError;

#[derive(Clone, Copy)]
pub enum BackendTextureFormat {
    Rgba8UnormSrgb = 1,
    Bgra8UnormSrgb = 2,
}

pub struct BackendTextureConfig {
    format: BackendTextureFormat,
    label: String,
    
    // TODO
}

impl BackendTextureConfig {
    pub fn new() -> Self {
        Self {
            format: BackendTextureFormat::Rgba8UnormSrgb,
            label: "Default texture".to_string(),
        }
    }

    pub fn set_format(&mut self, format: BackendTextureFormat) {
        self.format = format;
    }

    pub fn set_label(&mut self, label: String) {
        self.label = label;
    }

    pub fn get_format(&mut self) -> BackendTextureFormat {
        self.format.clone()
    }

    pub fn get_label(&mut self) -> String {
        self.label.clone()
    }
}

pub struct RawTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub bind_group: wgpu::BindGroup,
}

impl RawTexture {
    pub fn new(
        texture: wgpu::Texture,
        view: wgpu::TextureView,
        sampler: wgpu::Sampler,
        bind_group: wgpu::BindGroup
    ) -> Self {
        Self {
            texture,
            view,
            sampler,
            bind_group,
        }
    }
}

pub struct BackendTexture {
    pub width: u32,
    pub height: u32,
    pub config: BackendTextureConfig,
    raw: Option<RawTexture>,
}

impl BackendTexture {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            config: BackendTextureConfig::new(),
            raw: None,
        }
    }

    pub fn from_raw(
        &mut self,
        context: &mut BackendContext,
        bytes: &[u8],
        width: u32,
        height: u32,
    ) -> Result<(), MoonBackendError> {
        match &mut context.get_raw() {
            Some(raw_context) => {
                let usages: wgpu::TextureUsages = wgpu::TextureUsages::TEXTURE_BINDING 
                    | wgpu::TextureUsages::RENDER_ATTACHMENT 
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::COPY_DST;

                // Размер текстуры
                let size = wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                };

                let texture_format = match self.config.get_format() {
                    BackendTextureFormat::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8UnormSrgb,
                    BackendTextureFormat::Bgra8UnormSrgb => wgpu::TextureFormat::Bgra8UnormSrgb,
                };

                let texture = raw_context.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some(&self.config.get_label()),
                    size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: texture_format,
                    usage: usages,
                    view_formats: &[],
                });

                raw_context.queue.write_texture(
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
                
                let sampler = raw_context.device.create_sampler(&wgpu::SamplerDescriptor {
                    address_mode_u: wgpu::AddressMode::ClampToEdge,
                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                    address_mode_w: wgpu::AddressMode::ClampToEdge,
                    mag_filter: wgpu::FilterMode::Linear,
                    min_filter: wgpu::FilterMode::Linear,
                    mipmap_filter: wgpu::FilterMode::Nearest,
                    ..Default::default()
                });

                let bind_group_layout = raw_context.device.create_bind_group_layout(
                    &wgpu::BindGroupLayoutDescriptor {
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
                    }
                );

                let bind_group = raw_context.device.create_bind_group(
                    &wgpu::BindGroupDescriptor {
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
                    }
                );

                self.width = width;
                self.height = height;
                self.raw = Some(RawTexture::new(texture, view, sampler, bind_group));

                Ok(())
            }

            None => {
                Err(MoonBackendError::ContextNotFoundError)
            }
        }
    }
}