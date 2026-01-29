// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use crate::core::context::{BackendContext, RawContext};
use crate::error::MoonBackendError;

// Абстрация над wgpu, добавить другие типы по необходимости, но этих двух должно
// хватить для кейсов использования MoonWalk
#[derive(Clone, Copy)]
pub enum BackendTextureFormat {
    // Стандарт
    Rgba8UnormSrgb = 1,
    
    Bgra8UnormSrgb = 2,
}

// Приватное перечисление для определения типа текстуры
enum TextureType {
    Default = 1,
    RenderTarget = 2,
    Depth = 3,
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

    /// Этот метод устанавливает формат. Если не указать то будет
    /// BackendTextureFormat::Rgba8UnormSrgb
    pub fn set_format(&mut self, format: BackendTextureFormat) {
        self.format = format;
    }

    /// Установить название, оно используется только для отладки. Если не указать
    /// то будет использоваться стандартное "Default texture" которое устанавливается
    /// в методе new
    pub fn set_label(&mut self, label: String) {
        self.label = label;
    }

    /// Получить формат текстуры
    pub fn get_format(&mut self) -> BackendTextureFormat {
        self.format.clone()
    }

    /// Получить название текстуры
    pub fn get_label(&mut self) -> String {
        self.label.clone()
    }
}

/// Сырая текстура. Нужна чтобы передать без подключения wgpu
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
    // Ширина
    pub width: u32,
    
    // Высота
    pub height: u32,
    
    // Конфигурация
    pub config: BackendTextureConfig,
    
    // Сырая wgpu текстура
    raw: Option<RawTexture>,
}

impl BackendTexture {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,

            // По умолчанию
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
        self.create_texture(context, Some(bytes), width, height, TextureType::Default)
    }

    pub fn create_render_target(
        &mut self,
        context: &mut BackendContext,
        width: u32,
        height: u32,
    ) -> Result<(), MoonBackendError> {
        self.create_texture(context, None, width, height, TextureType::RenderTarget)
    }

    pub fn create_depth_texture(
        &mut self,
        context: &mut BackendContext,
        width: u32,
        height: u32,
    ) -> Result<(), MoonBackendError> {
        self.create_texture(context, None, width, height, TextureType::Depth)
    }

    fn create_texture(
        &mut self,
        context: &mut BackendContext,
        bytes: Option<&[u8]>,
        width: u32,
        height: u32,
        texture_type: TextureType,
    ) -> Result<(), MoonBackendError> {
        match &mut context.get_raw() {
            Some(raw_context) => {
                // Общие блоки кода для всех

                // Размер текстуры
                let size = self.pack_size(width, height);

                let format = self.config.get_format();
                let texture_format = self.map_format_to_wgpu(format);

                let usage = match texture_type {
                    TextureType::Default => self.get_usage(),
                    TextureType::RenderTarget => self.get_usage(),
                    TextureType::Depth => self.get_usage_depth(),
                };

                let texture = raw_context.device.create_texture(&wgpu::TextureDescriptor {
                    // То самое название из конфига
                    label: Some(&self.config.get_label()),
                    
                    // Тот самый размер
                    size,

                    // Дефолт, потом можно добавить настройки для них в конфигурации,
                    // но кейсы MoonWalk не требуют
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,

                    // Формат
                    format: texture_format,
                    
                    usage,
                    view_formats: &[],
                });

                // Запись в текстуру через очередь в сыром контексте только если
                // texture_type это дефолтная текстура, так как иначе запись
                // не нужна

                if let TextureType::Default = texture_type {
                    raw_context.queue.write_texture(
                        wgpu::TexelCopyTextureInfo {
                            texture: &texture,
                            mip_level: 0,
                            origin: wgpu::Origin3d::ZERO,
                            aspect: wgpu::TextureAspect::All,
                        },

                        // Я уверен в том что паники здесь не будет, так как
                        // этот блок выполняется ТОЛЬКО если texture_type это
                        // TextureType::Default (обычная текстура), а такое
                        // возможно только если вызов был из функции from_raw,
                        // а там массив байтов обязательный, без Option
                        // Паника тут может быть только в случае затупа, поэтому
                        // нужно проверять это всё. Без .unwrap() код был бы
                        // длиннее, но прикол в том что один этот комментарий
                        // длиннее чем код match который нужен для проверки на
                        // None, лол ;)
                        bytes.unwrap(),

                        wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(4 * width),
                            rows_per_image: Some(height),
                        },

                        size,
                    );
                }

                // Снова общие для всех части
                let view = texture.create_view(
                    &wgpu::TextureViewDescriptor::default()
                );
                
                let sampler_descriptor = match texture_type {
                    TextureType::Default | TextureType::RenderTarget => self.get_sampler_descriptor(),
                    TextureType::Depth => self.get_sampler_descriptor_depth(),
                };

                let sampler = raw_context.device.create_sampler(&sampler_descriptor);
                
                let bind_group_layout = self.create_bind_group_layout(&raw_context);
                let bind_group = self.create_bind_group(
                    &raw_context, &view, &bind_group_layout, &sampler,
                );

                // Заполнение параметров, Result здесь возвращается просто чтобы
                // вернуть ContextNotFoundError если что
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

    /// Хард код usage в метод, так как в константу нельзя :(
    fn get_usage(&self) -> wgpu::TextureUsages {
        wgpu::TextureUsages::TEXTURE_BINDING 
            | wgpu::TextureUsages::RENDER_ATTACHMENT 
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST
    }

    /// Специальный хардкод для depth. Отсуствуют COPY_SRC и COPY_DST
    /// так как они не нужны depth текстуре. Я не знаю влияет ли положительно
    /// отсуствие двух usage, но на всякий случай было бы неплохо убрать
    fn get_usage_depth(&self) -> wgpu::TextureUsages {
        wgpu::TextureUsages::RENDER_ATTACHMENT 
            | wgpu::TextureUsages::TEXTURE_BINDING
    }

    /// Хардкод сэмлер дескриптора через метод
    fn get_sampler_descriptor(&self) -> wgpu::SamplerDescriptor<'_> {
        // [MAYBE]
        // [HARDCODE]
        // Настройки сэмплера тоже можно добавить в конфиг, потом
        // этим займусь, пока сделаю пометку
        wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        }
    }

    /// Хардкод сэмлер дескриптора через метод специально для текстуры
    /// глубины (depth)
    fn get_sampler_descriptor_depth(&self) -> wgpu::SamplerDescriptor<'_> {
        // [MAYBE]
        // [HARDCODE]
        wgpu::SamplerDescriptor {
            compare: Some(wgpu::CompareFunction::LessEqual),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        }
    }

    /// Этот метод нужен для создания лайаута бинд группы для сырой текстуры wgpu
    /// Он создан для решения проблемы дубляжа кода между частями модуля
    fn create_bind_group_layout(&self, raw_context: &RawContext) -> wgpu::BindGroupLayout {
        raw_context.device.create_bind_group_layout(
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

                // [HARDCODE]
                // не очень круто для отладки
                label: Some("texture_bind_group_layout"),
            }
        )
    }

    /// Этот метод нужен для создания бинд группы для сырой текстуры wgpu
    fn create_bind_group(
        &self,
        raw_context: &RawContext,
        view: &wgpu::TextureView,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        raw_context.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(view),
                    },

                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                ],

                // [HARDCODE]
                // не очень круто для отладки
                label: Some("texture_bind_group"),
            }
        )
    }

    /// Этот метод нужен чтобы конвертировать абстрактное перечисление BackendTextureFormat
    /// в формат wgpu
    fn map_format_to_wgpu(&self, format: BackendTextureFormat) -> wgpu::TextureFormat {
        match format {
            BackendTextureFormat::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8UnormSrgb,
            BackendTextureFormat::Bgra8UnormSrgb => wgpu::TextureFormat::Bgra8UnormSrgb,
        }
    }

    // Упаковка разрешения текстуры в wgpu::Extent3d
    fn pack_size(&self, width: u32, height: u32) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        }
    }
}