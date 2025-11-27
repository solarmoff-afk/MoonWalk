# <div align="center">[MoonWalk](https://www.youtube.com/watch?v=dQw4w9WgXcQ)</div>

![MoonWalk header](assets/header.png)

MoonWalk is a lightweight, cross-platform 2D rendering engine written in Rust and based on wgpu.

## Features
* Rounded rectangles
* Text (Via cosmic-text library)
* Shaders (Support for custom wgsl shaders and uniform variables)

## Features beta
* Bezier curves
* Android support

# Quick start
```rust
use moonwalk::MoonWalk;
use glam::{Vec2, Vec4};

// 1. Init
let mut mw = MoonWalk::new(&window).unwrap();

// 2. Create rect
let rect_id = mw.new_rect();
mw.config_position(rect_id, Vec2::new(100.0, 100.0));
mw.config_size(rect_id, Vec2::new(200.0, 50.0));
mw.config_color(rect_id, Vec4::new(1.0, 0.2, 0.2, 1.0));
mw.set_rounded(rect_id, Vec4::splat(15.0));

// 3. Create bezier curves
let bezier_id = mw.new_bezier();
mw.set_bezier_points(bezier_id, vec![
    Vec2::new(50.0, 300.0),  // Start
    Vec2::new(150.0, 200.0), // Control 1
    Vec2::new(250.0, 400.0), // Control 2
    Vec2::new(350.0, 300.0), // End
]);
mw.config_bezier_thickness(bezier_id, 5.0);
mw.config_bezier_smooth(bezier_id, 1.0);
mw.config_color(bezier_id, Vec4::new(0.0, 1.0, 0.5, 1.0));

// 4. Create text
let text_id = mw.new_text();
mw.config_text(text_id, "Hello World!");
mw.config_position(text_id, Vec2::new(100.0, 200.0));
mw.config_color(text_id, Vec4::ONE);

// Rendering
mw.render_frame(Vec4::new(0.1, 0.1, 0.1, 1.0)).unwrap();
```