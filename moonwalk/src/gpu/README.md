# УСТАРЕВШИЙ ДОКУМЕНТ
> easy-gpu стал внутренним модулем MoonWalk, а не отдельной зависимостью. README.md остаётся как справочник для других разработчиков.

# EasyGPU (Обертка над wgpu)

Это модуль-обертка над графическим API `wgpu`, написанный на языке Rust. Основная цель модуля — упростить рутинную инициализацию графического контекста, управление ресурсами (буферами, текстурами) и пайплайнами рендеринга. Она скрывает сложность настройки дескрипторов `wgpu`, предоставляя высокоуровневый типизированный интерфейс, напоминающий классические подходы к графике, но с производительностью современного API.

## Зависимости

Для работы с модулем в вашем проекте (`Cargo.toml`) должны быть указаны следующие зависимости, так как публичный API библиотеки использует типы из них:

```toml
[dependencies]
wgpu = "0.24"         # Основной графический бэкенд
glam = "0.28"         # Математическая библиотека (Vec3, Mat4)
bytemuck = { version = "1.16", features = ["derive"] } # Для работы с байтовыми буферами
pollster = "0.3"      # Для блокирующего выполнения async кода
```

Для создания окна и управления событиями рекомендуется использовать `winit`.

## Архитектура и Модули

Модуль разделен на логические модули, каждый из которых отвечает за свой аспект графического конвейера.

### 1. Context (Контекст)

Модуль `context` отвечает за инициализацию взаимодействия с GPU.

**Структура `Context`** хранит:
-   `device`: Логическое устройство для создания ресурсов.
-   `queue`: Очередь команд для отправки задач на GPU.
-   `surface`: Поверхность рисования (связана с окном).
-   `config`: Конфигурация поверхности (размеры, формат, режим презентации).

**Ключевые методы:**
-   `new(window, width, height)`: Асинхронный конструктор. Автоматически выбирает подходящий графический адаптер (HighPerformance), создает устройство и настраивает поверхность с поддержкой sRGB.
-   `resize(width, height)`: Пересоздает конфигурацию поверхности (swapchain) при изменении размеров окна. Обязательно вызывать в обработчике событий ресайза.
-   `create_encoder()`: Создает `CommandEncoder` для записи команд рисования.
-   `submit(encoder)`: Завершает запись команд и отправляет их на исполнение в очередь.

### 2. Buffer (Буферы)

Модуль `buffer` предоставляет типобезопасную обертку `Buffer<T>`, где `T` должен реализовывать трейт `bytemuck::Pod`. Это гарантирует, что данные могут быть безопасно скопированы в память видеокарты.

**Виды буферов:**
-   `Buffer::vertex`: Создает вершинный буфер (Vertex Buffer).
-   `Buffer::index`: Создает индексный буфер (Index Buffer, всегда `u32`).
-   `Buffer::uniform`: Создает буфер констант (Uniform Buffer).
-   `Buffer::storage`: Создает буфер хранения (Storage Buffer).

**Управление данными:**
-   `update(&ctx, data)`: Полностью перезаписывает содержимое буфера. Если размер новых данных превышает текущий размер буфера, старый буфер уничтожается и создается новый с необходимым размером.
-   `update_one(&ctx, data)`: Оптимизированный метод для обновления одиночной структуры (полезно для Uniform-буферов).

### 3. Pipeline (Графический конвейер)

Модуль упрощает создание шейдерных программ и настройку этапов рендеринга.

**`PipelineBuilder`**:
Позволяет декларативно настроить параметры рендеринга перед сборкой.
-   `new(&ctx, shader_src)`: Принимает исходный код шейдера на языке WGSL.
-   `add_layout(layout)`: Добавляет описание структуры вершин (Stride, Attributes).
-   `with_topology(topology)`: Задает тип примитивов (список треугольников, линии, точки).
-   `with_stencil(stencil_state)`: Подключает настройки трафарета (stencil) для масок.
-   `no_blend()`: Отключает смешивание цветов (по умолчанию включено Alpha Blending).
-   `build(format, bind_group_layouts)`: Компилирует шейдер и создает готовый объект `Pipeline`.

### 4. RenderPass (Проход рендеринга)

Обертка `RenderPass` инкапсулирует процесс записи команд рисования в `wgpu::RenderPass`.

**Ключевые возможности:**
-   `new(encoder, view, clear_color)`: Начинает проход. Если передан `clear_color`, экран будет очищен этим цветом. Если `None`, будет загружено предыдущее содержимое (LoadOp::Load).
-   `set_pipeline`: Устанавливает активный шейдер.
-   `set_bind_group`: Привязывает группы ресурсов (текстуры, униформы).
-   `set_vertex_buffer` / `set_index_buffer`: Устанавливает геометрию.
-   `draw` / `draw_indexed`: Выполняет отрисовку.
-   `set_scissor`: Устанавливает прямоугольник отсечения (Scissor Rect).

### 5. MatrixStack (Матрицы и Трансформации)

Модуль реализует стек матриц, аналогичный старому конвейеру OpenGL (`glPushMatrix`/`glPopMatrix`), для удобного управления иерархическими трансформациями объектов.

**Структура `MatrixStack`:**
-   Хранит матрицы `projection`, `view`, `model`.
-   Использует библиотеку `glam` для математики.

**Методы:**
-   `set_ortho(width, height)`: Устанавливает ортогональную проекцию (левосторонняя система координат).
-   `push()` / `pop()`: Сохраняет или восстанавливает текущую матрицу модели из стека.
-   `translate`, `rotate_z`, `scale`: Модифицируют текущую матрицу модели.
-   `to_uniform()`: Преобразует состояние стека в структуру `MatrixUniform` (массивы 4x4), готовую для отправки в шейдер.

### 6. Texture (Текстуры)

Упрощает загрузку и создание текстур.
-   `from_bytes`: Загружает текстуру из массива байтов. Выполняет проверку соответствия размера данных указанным ширине, высоте и формату. Автоматически создает `TextureView` и `Sampler` (Linear).
-   `create_render_target`: Создает пустую текстуру, в которую можно осуществлять рендеринг (например, для пост-эффектов).

### 7. Mask (Маскирование / Stencil)

Предоставляет готовые пресеты `wgpu::StencilState` для реализации масок отсечения.
-   `Mask::write()`: Настраивает пайплайн на запись в буфер трафарета (записывает значение `0xFF`).
-   `Mask::read_equal()`: Настраивает пайплайн на проверку трафарета. Пиксели будут нарисованы только там, где в буфере трафарета значение равно `0xFF`.

---

## Пример использования (Базовый)

Ниже приведен пример базовой настройки приложения, создания ресурсов и отрисовки треугольника.

### Шейдер (shader.wgsl)
```wgsl
struct Uniforms {
    model: mat4x4<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> ubo: Uniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = ubo.proj * ubo.view * ubo.model * vec4<f32>(model.position, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
```

### Код на Rust

```rust
use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};
use crate::easygpu::{Context, Buffer, PipelineBuilder, MatrixStack, RenderPass};

// Определение структуры вершины для передачи в GPU
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new().with_title("EasyGPU Example").build(&event_loop).unwrap());

    // Инициализация контекста
    let mut ctx = pollster::block_on(Context::new(
        window.clone(),
        window.inner_size().width,
        window.inner_size().height
    ));

    // Создание данных
    let vertices = vec![
        Vertex { position: [0.0, 0.5], color: [1.0, 0.0, 0.0] },
        Vertex { position: [-0.5, -0.5], color: [0.0, 1.0, 0.0] },
        Vertex { position: [0.5, -0.5], color: [0.0, 0.0, 1.0] },
    ];
    let vertex_buffer = Buffer::vertex(&ctx, &vertices);
    
    // Стек матриц и униформ-буфер
    let mut matrix_stack = MatrixStack::new();
    matrix_stack.set_ortho(800.0, 600.0);
    let uniform_buffer = Buffer::uniform(&ctx, &matrix_stack.to_uniform());

    // Создание Layout для BindGroup
    let bind_group_layout = ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Uniform Bind Group Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Uniform Bind Group"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.raw.as_entire_binding(),
        }],
    });

    // Сборка Пайплайна
    let shader_source = include_str!("shader.wgsl");
    let pipeline = PipelineBuilder::new(&ctx, shader_source)
        .add_layout(wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x2, offset: 0, shader_location: 0 },
                wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 8, shader_location: 1 },
            ],
        })
        .build(ctx.config.format, &[&bind_group_layout]);

    // Цикл
    event_loop.run(move |event, target| {
        match event {
            Event::WindowEvent { event: WindowEvent::Resized(new_size), .. } => {
                ctx.resize(new_size.width, new_size.height);
                matrix_stack.set_ortho(new_size.width as f32, new_size.height as f32);
            }
            Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                matrix_stack.set_identity();
                matrix_stack.translate(glam::Vec3::new(400.0, 300.0, 0.0));
                
                uniform_buffer.update_one(&ctx, &matrix_stack.to_uniform());

                if let Ok(frame) = ctx.surface.as_ref().unwrap().get_current_texture() {
                    let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                    let mut encoder = ctx.create_encoder();

                    {
                        let mut pass = RenderPass::new(&mut encoder, &view, Some(wgpu::Color::BLACK));
                        pass.set_pipeline(&pipeline);
                        pass.set_bind_group(0, &bind_group);
                        pass.set_vertex_buffer(0, &vertex_buffer);
                        pass.draw(3);
                    }

                    ctx.submit(encoder);
                    frame.present();
                }
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                target.exit();
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    }).unwrap();
}
```

---

## Рецепты и частые сценарии

В этом разделе описано решение специфических задач, которые могут быть неочевидны при использовании базового API.

### 1. Аналог `glViewport` и ограничение области рисования

Часто возникает вопрос: *"Как изменить Viewport, чтобы рисовать только в части экрана?"*.

По умолчанию `RenderPass::new` устанавливает область рисования (viewport) на **всю** площадь целевой текстуры. Библиотека `EasyGPU` в текущей реализации не экспортирует метод `set_viewport`, считая, что в большинстве 2D-сценариев достаточно отсечения (Scissor Test).

**Вариант А: Использование `set_scissor` (Отсечение)**
Если ваша цель — ограничить вывод (например, отрисовка UI внутри окна), используйте метод `set_scissor`. В отличие от Viewport, это не масштабирует систему координат, а просто отбрасывает пиксели за пределами прямоугольника.

```rust
let mut pass = RenderPass::new(&mut encoder, &view, Some(wgpu::Color::BLACK));

// Рисуем фон на весь экран
pass.set_pipeline(&bg_pipeline);
pass.draw(6);

// Ограничиваем область рисования прямоугольником (x: 100, y: 100, w: 200, h: 200)
pass.set_scissor(100, 100, 200, 200);
pass.set_pipeline(&ui_pipeline);
pass.draw(6); // Будет видно только внутри квадрата 200x200
```

**Вариант Б: Модификация библиотеки для поддержки `set_viewport`**
Если вам необходимо именно масштабирование координат (например, для сплит-скрина), вам нужно добавить метод в `src/pass.rs`, так как поле `raw` (оригинальный `wgpu::RenderPass`) приватно.

Добавьте в `impl RenderPass`:
```rust
pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32, min_depth: f32, max_depth: f32) {
    self.raw.set_viewport(x, y, w, h, min_depth, max_depth);
}
```

### 2. Рендеринг в текстуру (Offscreen Rendering)

Для пост-эффектов или генерации текстур используется `Framebuffer`.

```rust
use crate::easygpu::Framebuffer;

// 1. Создание кадрового буфера и текстуры (где-то при инициализации)
let (offscreen_fb, target_texture) = Framebuffer::offscreen(&ctx, 800, 600, wgpu::TextureFormat::Rgba8Unorm);

// 2. В цикле рендеринга
{
    let mut encoder = ctx.create_encoder();
    
    // Проход 1: Рисуем сцену в текстуру
    {
        let mut pass = RenderPass::new(&mut encoder, &offscreen_fb.view, Some(wgpu::Color::BLUE));
        pass.set_pipeline(&scene_pipeline);
        pass.draw(3);
    }
    
    // Проход 2: Рисуем текстуру на экран (нужен bind_group с target_texture)
    let frame = ctx.surface.as_ref().unwrap().get_current_texture().unwrap();
    let screen_view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
    
    {
        let mut pass = RenderPass::new(&mut encoder, &screen_view, Some(wgpu::Color::BLACK));
        pass.set_pipeline(&post_process_pipeline); // Шейдер, который принимает текстуру
        pass.set_bind_group(0, &texture_bind_group); // BindGroup созданный из target_texture
        pass.draw(6); // Рисуем квад на весь экран
    }
    
    ctx.submit(encoder);
    frame.present();
}
```

### 3. Использование масок (Stencil)

Для вырезания сложных форм используется `Stencil Buffer`. Это требует создания пайплайна с поддержкой стенсила и специальной текстуры глубины/стенсила.

```rust
use crate::easygpu::{Mask, PipelineBuilder};

// 1. Пайплайн, который "пишет" маску
let mask_pipeline = PipelineBuilder::new(&ctx, shader_src)
    .with_stencil(Mask::write()) // Запишет 0xFF везде, где нарисуется геометрия
    .no_blend() // Обычно маску пишут без блендинга
    .build(ctx.config.format, &[]);

// 2. Пайплайн, который "читает" маску
let content_pipeline = PipelineBuilder::new(&ctx, shader_src)
    .with_stencil(Mask::read_equal()) // Нарисует пиксель только если в стенсиле 0xFF
    .build(ctx.config.format, &[]);

// Важно: RenderPass должен иметь depth_stencil_attachment при создании, 
// чего нет в текущем упрощенном RenderPass::new. 
// Для этого потребуется расширить метод new в pass.rs, добавив аргумент для depth_view.
```