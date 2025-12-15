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
    fn on_start(&mut self, mw: &mut MoonWalk, viewport: Vec2) {
        println!("Loading Texture App...");
        self.screen_size = viewport;

        match mw.load_texture("test.png") {
            Ok(id) => {
                println!("Texture loaded with ID: {}", id);
                self.texture_id = id;
            },
            Err(e) => {
                eprintln!("Failed to load texture: {}", e);
            }
        }

        let bg = mw.new_rect();
        mw.set_position(bg, Vec2::ZERO);
        mw.set_size(bg, viewport * 2.0);
        mw.set_color(bg, Vec4::new(0.1, 0.1, 0.1, 1.0));
        mw.set_z_index(bg, 0.0);

        let sprite = mw.new_rect();
        self.sprite_id = Some(sprite);
 
        let size = 300.0;
        let pos = (viewport - size) * 0.5;

        mw.set_position(sprite, pos);
        mw.set_size(sprite, Vec2::splat(size));
        mw.set_color(sprite, Vec4::new(1.0, 0.6, 0.1, 1.0));
        mw.set_color2(sprite, Vec4::new(0.9, 0.3, 0.1, 1.0));
        mw.linear_gradient(sprite, Vec2::new(1.0, 0.0));
        mw.set_z_index(sprite, 0.5);
        
        if self.texture_id > 0 {
            mw.set_texture(sprite, self.texture_id);
        }

        mw.set_rounded(sprite, Vec4::splat(50.0)); 
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