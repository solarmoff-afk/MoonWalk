// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use wgpu;

#[test]
fn validate_rect_shader() {
    // Создаем контекст WGPU без окна
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    // Пытаемся найти любой адаптер (даже программный)
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        force_fallback_adapter: false,
        compatible_surface: None,
    })).expect("Failed to find wgpu adapter");

    let (device, _) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor::default(),
        None,
    )).expect("Failed to create wgpu device");

    // Читаем файл шейдера
    let shader_source = include_str!("../src/shaders/shape.wgsl");

    // Пытаемся скомпилировать шейдер. Если в коде есть ошибки синтаксиса то
    // эта функция запаникует и тест упадет
    let _ = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Test Rect Shader"),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });
}