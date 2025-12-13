// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use wgpu;
use std::sync::Arc;
use moonwalk::rendering::texture::Texture;
use moonwalk::easy_gpu::Context;

#[test]
fn test_texture_from_memory() {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        force_fallback_adapter: false, // Или true, если нет GPU
        ..Default::default()
    })).unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default(), None)).unwrap();
let mut img_buf = Vec::new();
    {
        use image::{ImageEncoder, ColorType};
        let pixel = [255u8, 0, 0, 255]; // Красный
        let encoder = image::codecs::png::PngEncoder::new(&mut img_buf);
        encoder.write_image(&pixel, 1, 1, ColorType::Rgba8.into()).unwrap();
    }

    let loaded = image::load_from_memory(&img_buf).unwrap();
    assert_eq!(loaded.width(), 1);
    assert_eq!(loaded.height(), 1);
}