// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use wgpu::SurfaceTargetUnsafe;

use crate::render::texture::{BackendTexture, BackendTextureFormat, map_wgpu_to_format};
use crate::error::MoonBackendError;

const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

// Это перечисление нужно для абстрации над голым wgpu
pub enum BackendPresentMode {
    // First-In, First-Out, вертикальная синхронизация (vsync). Gpu ждёт
    // следующего обновления дисплея прежде чем показать новый кадр
    // обычное ограничение это 60 fps (60 гц монитора), если на мониторе
    // 120 гц то будет ограничение в 120 fps и так далее, зависит от герцовки
    // монитора
    Fifo = 1,

    // Автоматический без вертикальной синхронизации. Gpu показывает
    // кадры как можно быстрее без ожидания обновления
    AutoNoVsync = 2,
}

// Структура публичная на будущее, например, если потребуется get_raw метод
pub struct RawContext {
    // Видеокарта
    pub device: wgpu::Device,
    
    // Очередь
    pub queue: wgpu::Queue,
    
    // Поверхность рисования (экран)
    pub surface: Option<wgpu::Surface<'static>>,
    
    // Конфигурация поверхности
    pub config: wgpu::SurfaceConfiguration,
    
    // Адаптер и его информация
    pub adapter: wgpu::Adapter,
    pub adapter_info: wgpu::AdapterInfo,
    
    // Экземпляр wgpu
    pub instance: wgpu::Instance,

    pub current_frame: Option<wgpu::SurfaceTexture>,
}

impl RawContext {
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        surface: Option<wgpu::Surface<'static>>,
        config: wgpu::SurfaceConfiguration,
        adapter: wgpu::Adapter,
        adapter_info: wgpu::AdapterInfo,
        instance: wgpu::Instance,
    ) -> Self {
        Self {
            device,
            queue,
            surface,
            config,
            adapter,
            adapter_info,
            instance,
            current_frame: None,
        }
    }
}

pub struct Vec2u32 {
    pub x: u32,
    pub y: u32,
}

impl Vec2u32 {
    pub fn new(x: u32, y: u32) -> Self {
        Self {
            x,
            y,
        }
    }
}

pub struct BackendContext {
    context: Option<RawContext>,
}

impl BackendContext {
    pub fn new() -> Self {
        // Не создаём RawContext чтобы создать его потом синхронно через pollster
        Self {
            context: None,
        }
    }

    pub fn create_context_sync(
        &mut self,
        window: &(impl HasWindowHandle + HasDisplayHandle),
        width: u32,
        height: u32
    ) {
        self.context = Some(pollster::block_on(
            self.create_context_async(window, width, height)
        ));
    }

    pub async fn create_context_async(
        &mut self,
        window: &(impl HasWindowHandle + HasDisplayHandle),
        width: u32,
        height: u32,
    ) -> RawContext {
        // Создание экземпляра
        let instance = wgpu::Instance::new(
            &wgpu::InstanceDescriptor::default()
        );

        let target = unsafe {
            SurfaceTargetUnsafe::from_window(window).unwrap()
        };

        // TODO: Заменить панику на Result
        let surface = unsafe {
            instance.create_surface_unsafe(target)
        }.expect("Failed to create surface");

        let surface = unsafe {
            std::mem::transmute::<wgpu::Surface<'_>, wgpu::Surface<'static>>(surface)
        };

        // TODO: Заменить панику на Result
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("No suitable GPU adapter found");

        // TODO: Заменить панику на Result
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("MoonWalk Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let caps = surface.get_capabilities(&adapter);
        
        // [FIX]
        // Bag report #1: Fix context for windows
        let format = caps
            .formats.iter()
            .copied()
            .find(|f| *f == TEXTURE_FORMAT) // Ищем BGRA явно
            .or_else(|| caps.formats.iter().copied().find(|f| f.is_srgb())) // Если нет, берем любой sRGB
            .unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let adapter_info = adapter.get_info();

        RawContext::new(
            device,
            queue,
            Some(surface),
            config,
            adapter,
            adapter_info,
            instance,
        )
    }

    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), MoonBackendError> {
        match &mut self.context.as_mut() {
            Some(context) => {
                if width > 0 && height > 0 {
                    context.config.width = width;
                    context.config.height = height;
                    self.configure_surface();
                }

                Ok(())
            },

            None => Err(MoonBackendError::ContextNotFoundError),
        }
    }

    pub fn recreate_surface(
        &mut self,
        window: &(impl HasWindowHandle + HasDisplayHandle),
        width: u32,
        height: u32,
    ) -> Result<(), MoonBackendError> {
        match &mut self.context.as_mut() {
            Some(context) => {
                let target = unsafe {
                    SurfaceTargetUnsafe::from_window(window).unwrap()
                };

                let new_surface = unsafe {
                    context.instance.create_surface_unsafe(target)
                }.expect("Failed to recreate surface");

                // HACK: Это может вызвать UB, нужен рефакторинг
                let new_surface = unsafe {
                    std::mem::transmute::<wgpu::Surface<'_>, wgpu::Surface<'static>>(new_surface)
                };

                let caps = new_surface.get_capabilities(&context.adapter);
                
                // Bag report #1: Fix context for windows
                let format = caps
                    .formats.iter()
                    .copied()
                    .find(|f| *f == wgpu::TextureFormat::Bgra8UnormSrgb)
                    .or_else(|| caps.formats.iter().copied().find(|f| f.is_srgb()))
                    .unwrap_or(caps.formats[0]);

                context.config.width = width;
                context.config.height = height;
                context.config.format = format;
                context.config.alpha_mode = caps.alpha_modes[0];

                context.surface = Some(new_surface);
                self.configure_surface();
            
                Ok(())
            },
            
            None => Err(MoonBackendError::ContextNotFoundError),
        }
    }

    pub fn get_raw(&mut self) -> Option<&RawContext> {
        self.context.as_ref()
    }

    fn configure_surface(&mut self) {
        // Так как configure_surface приватный метод который вызывают
        // другие публичнве методы с проверкой на то, что context не None
        // поэтому match тут нужен просто чтобы компилятор не ругался
        
        match &mut self.context.as_mut() {
            Some(context) => {
                if let Some(surface) = &context.surface {
                    surface.configure(&context.device, &context.config);
                }
            },
            None => println!("WTF?"),
        }
    }

    pub fn set_present_mode(&mut self, mode: BackendPresentMode) -> Result<(), MoonBackendError> {
        match &mut self.context.as_mut() {
            Some(context) => {
                let mode = match mode {
                    BackendPresentMode::Fifo => wgpu::PresentMode::Fifo,
                    BackendPresentMode::AutoNoVsync => wgpu::PresentMode::AutoNoVsync,
                };

                context.config.present_mode = mode;
                self.configure_surface();

                Ok(())
            },
            None => Err(MoonBackendError::ContextNotFoundError),
        }
    }

    pub fn get_format(&mut self) -> BackendTextureFormat {
        match &mut self.context.as_mut() {
            Some(raw_context) => {
                map_wgpu_to_format(raw_context.config.format)
            },

            None => BackendTextureFormat::Rgba8UnormSrgb,
        }
    }

    pub fn write_texture(
        &mut self,
        texture: &BackendTexture,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        data: &Vec<u8>,
    ) -> Result<(), MoonBackendError> {
        match (&mut self.get_raw(), texture.get_raw()) {
            (Some(raw_context), Some(raw_texture)) => {
                raw_context.queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &raw_texture.texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x,
                            y,
                            z: 0
                        },
                        aspect: wgpu::TextureAspect::All,
                    },

                    &data,
                    
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(w),
                        rows_per_image: None,
                    },
                    
                    wgpu::Extent3d {
                        width: w,
                        height: h,
                        depth_or_array_layers: 1
                    },
                );

                Ok(())
            },

            _ => {
                Err(MoonBackendError::ContextNotFoundError)
            }
        }
    }

    /// Записывает текстуру с явным указанием bytes_per_row (для выравнивания)
    /// Используется для случаев, когда данные уже padded или требуется точный контроль
    pub fn write_texture_aligned(
        &mut self,
        texture: &BackendTexture,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        data: &[u8],
        bytes_per_row: u32,
    ) -> Result<(), MoonBackendError> {
        match (&mut self.get_raw(), texture.get_raw()) {
            (Some(raw_context), Some(raw_texture)) => {
                raw_context.queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &raw_texture.texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x,
                            y,
                            z: 0,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                        
                    data,
                    
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(bytes_per_row),
                        rows_per_image: None,
                    },
                    
                    wgpu::Extent3d {
                        width: w,
                        height: h,
                        depth_or_array_layers: 1,
                    },
                );
                    
                Ok(())
            },
            
            _ => Err(MoonBackendError::ContextNotFoundError),
        }
    }

    pub fn get_size(&mut self) -> Result<Vec2u32, MoonBackendError> {
        match &mut self.context.as_mut() {
            Some(raw_context) => {
                Ok(Vec2u32::new(raw_context.config.width, raw_context.config.height))
            },

            None => Err(MoonBackendError::ContextNotFoundError),
        }
    }
}

pub struct SurfaceRenderer {
    cached_texture: Option<BackendTexture>,
    cached_width: u32,
    cached_height: u32,
    pending_frame: Option<wgpu::SurfaceTexture>,
    pending_view: Option<wgpu::TextureView>,
    pending_sampler: Option<wgpu::Sampler>,
}

impl SurfaceRenderer {
    pub fn new() -> Self {
        Self {
            cached_texture: None,
            cached_width: 0,
            cached_height: 0,
            pending_frame: None,
            pending_view: None,
            pending_sampler: None,
        }
    }

    // [AI]
    pub fn begin(&mut self, context: &mut BackendContext) -> Result<&mut BackendTexture, MoonBackendError> {
        let raw_context = match context.context.as_mut() {
            Some(raw_context) => raw_context,
            None => return Err(MoonBackendError::ContextNotFoundError),
        };
        
        let surface = match raw_context.surface.as_ref() {
            Some(surface) => surface,
            None => return Err(MoonBackendError::SurfaceNotInitializedError),
        };
        
        let frame = match surface.get_current_texture() {
            Ok(frame) => frame,
            Err(error) => return Err(MoonBackendError::SurfaceError(error.to_string())),
        };
        
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = raw_context.device.create_sampler(&wgpu::SamplerDescriptor::default());
        
        self.pending_frame = Some(frame);
        self.pending_view = Some(view);
        self.pending_sampler = Some(sampler);
        
        let width = raw_context.config.width;
        let height = raw_context.config.height;
        
        let needs_new_texture = match self.cached_texture.as_ref() {
            Some(texture) => texture.width != width || texture.height != height,
            None => true,
        };
        
        match needs_new_texture {
            true => {
                let mut new_texture = BackendTexture::new(width, height);
                
                match new_texture.create_render_target(context, width, height) {
                    Ok(_) => {
                        match (self.pending_view.take(), self.pending_sampler.take()) {
                            (Some(view), Some(sampler)) => {
                                new_texture.update_view(view);
                                new_texture.update_sampler(sampler);
                                
                                self.cached_texture = Some(new_texture);
                                self.cached_width = width;
                                self.cached_height = height;
                            },
                            
                            _ => return Err(MoonBackendError::SurfaceError("Failed to get pending resources".to_string())),
                        }
                    },

                    Err(error) => return Err(error),
                }
            },

            false => {
                match (self.pending_view.take(), self.pending_sampler.take()) {
                    (Some(view), Some(sampler)) => {
                        match self.cached_texture.as_mut() {
                            Some(texture) => {
                                texture.update_view(view);
                                texture.update_sampler(sampler);
                            },

                            None => return Err(MoonBackendError::SurfaceError("No cached texture".to_string())),
                        }
                    },

                    _ => return Err(MoonBackendError::SurfaceError("Failed to get pending resources".to_string())),
                }
            },
        }
        
        match self.cached_texture.as_mut() {
            Some(texture) => Ok(texture),
            None => Err(MoonBackendError::SurfaceError("Failed to get texture".to_string())),
        }
    }
    
    // [AI]
    pub fn end(&mut self) -> Result<(), MoonBackendError> {
        match self.pending_frame.take() {
            Some(frame) => {
                frame.present();

                Ok(())
            },

            None => Err(MoonBackendError::SurfaceError("No frame to present".to_string())),
        }
    }
}
