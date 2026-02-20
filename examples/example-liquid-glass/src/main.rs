use moonwalk::{MoonWalk, ObjectId};
use moonwalk_bootstrap::{Application, Runner, WindowSettings, TouchPhase};
use glam::{Vec2, Vec4};

#[cfg(target_os = "android")]
use android_activity::AndroidApp;

struct LiquidGlassApp {
    input_texture: u32,
    output_texture: u32,
    screen_rect_id: Option<ObjectId>, 
    screen_size: Vec2,
    glass_pos: Vec2,
    glass_size: Vec2,
}

impl LiquidGlassApp {
    fn new() -> Self {
        Self {
            input_texture: 0,
            output_texture: 0,
            screen_rect_id: None, 
            screen_size: Vec2::ZERO,
            glass_pos: Vec2::new(400.0, 300.0),
            glass_size: Vec2::new(200.0, 200.0),
        }
    }
}

impl Application for LiquidGlassApp {
    fn on_start(&mut self, mw: &mut MoonWalk, viewport: Vec2) {
        self.screen_size = viewport;

        let bg_texture = match mw.load_texture("assets/test.jpg") {
            Ok(id) => id,
            Err(_) => { 
                0
            }
        };

        let mut bg_container = mw.new_render_container(viewport.x as u32, viewport.y as u32);
        
        let bg_rect = bg_container.new_rect();
        bg_container.set_position(bg_rect, Vec2::ZERO);
        bg_container.set_size(bg_rect, viewport);
        
        if bg_texture > 0 {
            bg_container.set_texture(bg_rect, bg_texture);
        } else {
            let checker_size = 50.0;
            for i in 0..(viewport.x / checker_size) as i32 + 1 {
                for j in 0..(viewport.y / checker_size) as i32 + 1 {
                    let x = i as f32 * checker_size;
                    let y = j as f32 * checker_size;
                    
                    let color = if (i + j) % 2 == 0 {
                        Vec4::new(0.8, 0.8, 0.8, 1.0)
                    } else {
                        Vec4::new(0.3, 0.3, 0.3, 1.0)
                    };
                    
                    let tile = bg_container.new_rect();
                    bg_container.set_position(tile, Vec2::new(x, y));
                    bg_container.set_size(tile, Vec2::splat(checker_size));
                    bg_container.set_color(tile, color);
                }
            }
        }

        bg_container.draw(mw, Some(Vec4::ZERO));

        self.input_texture = bg_container.snapshot(mw, 0, 0, viewport.x as u32, viewport.y as u32);
        self.output_texture = bg_container.snapshot(mw, 0, 0, viewport.x as u32, viewport.y as u32);

        let screen_rect = mw.new_rect();
        mw.set_position(screen_rect, Vec2::ZERO);
        mw.set_size(screen_rect, viewport);
        mw.set_texture(screen_rect, self.output_texture);
        self.screen_rect_id = Some(screen_rect);
    }

    fn on_draw(&mut self, mw: &mut MoonWalk) {
        if self.input_texture == 0 || self.output_texture == 0 {
            return;
        }

        let physical_size = self.glass_size;
        let physical_pos = self.glass_pos;

        mw.liquid_glass(
            self.input_texture,
            self.output_texture,
            physical_size,
            physical_pos,
            Vec4::splat(30.0),
            19.0,    // refraction_height
            -20.0,   // refraction_amount
            0.20,    // depth_effect
            0.1,     // chromatic_aberration
            0.0, 
            1.2,     // gamma
            Vec4::new(1.0, 1.0, 1.0, 0.0),
        );
    }

    fn on_touch(&mut self, mw: &mut MoonWalk, phase: TouchPhase, position: Vec2) {
        match phase { 
            TouchPhase::Moved => {
                self.glass_pos = position; 
            },

            _ => {},
        }
    }

    fn on_resize(&mut self, mw: &mut MoonWalk, viewport: Vec2) {
        self.screen_size = viewport;
        if let Some(id) = self.screen_rect_id {
            mw.set_size(id, viewport);
        }
    }

    fn on_update(&mut self, _: f32) {

    }
}

#[cfg(not(target_os = "android"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = LiquidGlassApp::new();
    let settings = WindowSettings::new("MoonWalk Liquid Glass Demo", 800.0, 600.0).resizable(true);
    Runner::run(app, settings)
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info)
    );
    let app_instance = LiquidGlassApp::new();
    let settings = WindowSettings::new("MoonWalk Liquid Glass", 0.0, 0.0);
    Runner::run(app_instance, settings, app).unwrap();
}

#[cfg(target_os = "android")]
fn main() {}
