// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use wgpu::SurfaceTargetUnsafe;

pub struct Context {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: Option<wgpu::Surface<'static>>,
    pub config: wgpu::SurfaceConfiguration,
    pub adapter: wgpu::Adapter,
    pub adapter_info: wgpu::AdapterInfo,
    pub instance: wgpu::Instance,
}

impl Context {
    pub async fn new(
        window: &(impl HasWindowHandle + HasDisplayHandle),
        width: u32,
        height: u32,
    ) -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

        let target = unsafe {
            SurfaceTargetUnsafe::from_window(window).unwrap()
        };

        let surface = unsafe {
            instance.create_surface_unsafe(target)
        }.expect("Failed to create surface");

        let surface = unsafe {
            std::mem::transmute::<wgpu::Surface<'_>, wgpu::Surface<'static>>(surface)
        };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("No suitable GPU adapter found");

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
            .find(|f| *f == wgpu::TextureFormat::Bgra8UnormSrgb) // Ищем BGRA явно
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

        Self {
            device,
            queue,
            surface: Some(surface),
            config,
            adapter,
            adapter_info,
            instance,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.configure_surface();
        }
    }

    pub fn recreate_surface(
        &mut self,
        window: &(impl HasWindowHandle + HasDisplayHandle),
        width: u32,
        height: u32,
    ) {
        let target = unsafe {
            SurfaceTargetUnsafe::from_window(window).unwrap()
        };

        let new_surface = unsafe {
            self.instance.create_surface_unsafe(target)
        }.expect("Failed to recreate surface");

        let new_surface = unsafe {
            std::mem::transmute::<wgpu::Surface<'_>, wgpu::Surface<'static>>(new_surface)
        };

        let caps = new_surface.get_capabilities(&self.adapter);
        
        // [FIX]
        // Bag report #1: Fix context for windows
        let format = caps
            .formats.iter()
            .copied()
            .find(|f| *f == wgpu::TextureFormat::Bgra8UnormSrgb)
            .or_else(|| caps.formats.iter().copied().find(|f| f.is_srgb()))
            .unwrap_or(caps.formats[0]);

        self.config.width = width;
        self.config.height = height;
        self.config.format = format;
        self.config.alpha_mode = caps.alpha_modes[0];

        self.surface = Some(new_surface);
        self.configure_surface();
    }

    fn configure_surface(&self) {
        if let Some(surface) = &self.surface {
            surface.configure(&self.device, &self.config);
        }
    }

    pub fn create_encoder(&self) -> wgpu::CommandEncoder {
        self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        })
    }

    pub fn submit(&self, encoder: wgpu::CommandEncoder) {
        self.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn set_present_mode(&mut self, mode: wgpu::PresentMode) {
        self.config.present_mode = mode;
        self.configure_surface();
    }
}