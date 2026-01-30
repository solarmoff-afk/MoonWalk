// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use wgpu::SurfaceTargetUnsafe;

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
            context: None
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
}