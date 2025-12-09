// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use crate::easy_gpu::context::Context;
use crate::easy_gpu::texture::Texture;

pub struct Framebuffer {
    pub view: wgpu::TextureView,
    pub depth_view: Option<wgpu::TextureView>,
    pub width: u32,
    pub height: u32,
    pub format: wgpu::TextureFormat,
}

impl Framebuffer {
    pub fn for_surface(texture: &wgpu::SurfaceTexture, config: &wgpu::SurfaceConfiguration) -> Self {
        let view = texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            view,
            depth_view: None,
            width: config.width,
            height: config.height,
            format: config.format,
        }
    }

    pub fn offscreen(ctx: &Context, width: u32, height: u32, format: wgpu::TextureFormat) -> (Self, Texture) {
        let texture = Texture::create_render_target(ctx, width, height, format);
        
        let fb = Self {
            view: texture.texture.create_view(&wgpu::TextureViewDescriptor::default()),
            depth_view: None,
            width,
            height,
            format,
        };
        (fb, texture)
    }
}