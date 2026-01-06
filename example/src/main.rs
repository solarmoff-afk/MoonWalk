// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use moonwalk::{MoonWalk, ObjectId};
use moonwalk_bootstrap::{Application, Runner, WindowSettings};
use glam::{Vec2, Vec4};

#[cfg(target_os = "android")]
use android_activity::AndroidApp;

struct TextureApp {
    sprite_id: Option<ObjectId>,
    texture_id: u32,
    screen_size: Vec2,
    angle: f32,
}

impl TextureApp {
    fn new() -> Self {
        Self {
            sprite_id: None,
            texture_id: 0,
            screen_size: Vec2::new(800.0, 600.0),
            angle: 0.0,
        }
    }
}

impl Application for TextureApp {
    // fn on_start(&mut self, mw: &mut MoonWalk, viewport: Vec2) {
    //     println!("Loading Texture App...");
    //     self.screen_size = viewport;

    //     match mw.load_texture("assets/test.jpg") {
    //         Ok(id) => {
    //             println!("Texture loaded with ID: {}", id);
    //             self.texture_id = id;
    //         },
    //         Err(e) => {
    //             eprintln!("Failed to load texture: {}", e);
    //         }
    //     }

    //     let bg = mw.new_rect();
    //     mw.set_position(bg, Vec2::ZERO);
    //     mw.set_size(bg, viewport * 2.0);
    //     mw.set_color(bg, Vec4::new(0.1, 0.1, 0.1, 1.0));
    //     mw.set_z_index(bg, 0.0);

    //     let sprite = mw.new_rect();
    //     self.sprite_id = Some(sprite);
 
    //     let size = 300.0;
    //     let pos = (viewport - size) * 0.5;

    //     mw.set_position(sprite, pos);
    //     mw.set_size(sprite, Vec2::splat(size));
    //     // mw.set_color(sprite, Vec4::new(1.0, 0.6, 0.1, 1.0));
    //     mw.set_color2(sprite, Vec4::new(0.9, 0.3, 0.1, 1.0));
    //     // mw.linear_gradient(sprite, Vec2::new(1.0, 0.0));
    //     mw.set_z_index(sprite, 0.5);
    //     mw.set_effect(sprite, 10.0, 0.0);
        
    //     mw.blur_texture(self.texture_id, 10.0, true);
    //     mw.blur_texture(self.texture_id, 10.0, false);

    //     if self.texture_id > 0 {
    //         mw.set_texture(sprite, self.texture_id);
    //     }

    //     mw.set_rounded(sprite, Vec4::splat(50.0));

    //     let mut pb = mw.new_path_builder();
    //     pb.set_color(Vec4::new(1.0, 0.0, 0.0, 1.0));
    //     // pb.move_to(10.0, 10.0);
    //     // pb.line_to(100.0, 10.0);
    //     // pb.line_to(50.0, 100.0);
    //     // pb.close();

    //     mw.parse_svg_path(&mut pb, "M 10 10 L 90 10 L 50 90 Z").unwrap();

    //     let tex_id = pb.tessellate(mw, 200, 200);

    //     let id = mw.new_rect();
    //     mw.set_texture(id, tex_id);

    //     if let Ok(font) = mw.load_font("assets/Hundo.ttf", "BenchFont") {
    //         let id2 = mw.new_text("MoonWalk UI", font, 16.0);
    //         mw.set_position(id2, Vec2::new(20.0, 50.0));
    //     } else {
    //          println!("Note: Text tests will be skipped (no assets/font.ttf)");
    //     } 
    // }

    fn on_start(&mut self, mw: &mut MoonWalk, viewport: Vec2) {
        let mars_texture_id = mw.load_texture("assets/mars.jpg").unwrap(); 
        let font = mw.load_font("assets/Hundo.ttf", "Hundo").unwrap();

        let width = 600;
        let height = 300;
        let mut container = mw.new_render_container(width, height);
        
        let text_id = container.new_text("MARS", font, 86.0); 
        container.config_position(text_id, Vec2::new(10.0, 10.0)); 
        container.config_color(text_id, Vec4::new(1.0, 1.0, 1.0, 1.0));
        container.draw(mw, Some(Vec4::ZERO));
    
        let mask_id = container.snapshot(mw, 0, 0, width, height);
        mw.apply_mask(mars_texture_id, mask_id, true);
         
        let bg = mw.new_rect();
        mw.set_size(bg, viewport);
        mw.set_color(bg, Vec4::new(0.0, 0.0, 0.0, 1.0));
        
        let result_obj = mw.new_rect();
        mw.set_position(result_obj, Vec2::new(100.0, 100.0));
        mw.set_size(result_obj, Vec2::new(width as f32, height as f32));
        mw.set_texture(result_obj, mars_texture_id);

        mw.save_texture(mars_texture_id, "assets/output.png").unwrap();
    }

    fn on_update(&mut self, dt: f32) {
        self.angle += dt * 1.0;
    }

    fn on_draw(&mut self, mw: &mut MoonWalk) {
        if let Some(id) = self.sprite_id {
            mw.set_rotation(id, self.angle);
        }
    }

    fn on_resize(&mut self, mw: &mut MoonWalk, viewport: Vec2) {
        self.screen_size = viewport;
        
        if let Some(id) = self.sprite_id {
             let size = 300.0;
             let pos = (viewport - size) * 0.5;
             mw.set_position(id, pos);
        }
    }
}

#[cfg(not(target_os = "android"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = TextureApp::new();
    let settings = WindowSettings::new("MoonWalk Texture Test", 800.0, 600.0).resizable(true);
    Runner::run(app, settings)
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info)
    );

    log::info!("MoonWalk: android_main started");

    let stress_app = TextureApp::new();
    let settings = WindowSettings::new("MoonWalk Android", 0.0, 0.0);
    Runner::run(stress_app, settings, app).unwrap();
}

#[cfg(target_os = "android")]
fn main() {}
