use moonwalk::{MoonWalk, ObjectId};
use moonwalk_bootstrap::{Application, Runner, WindowSettings};
use glam::{Vec2, Vec4};

#[cfg(target_os = "android")]
use android_activity::AndroidApp;

const TEXT_COUNT: usize = 500;

struct TextBouncer {
    id: ObjectId,
    pos: Vec2,
    vel: Vec2,
    text: String,
}

struct TextApp {
    bouncers: Vec<TextBouncer>,
    screen_size: Vec2,
    fps_counter_id: Option<ObjectId>,
    time_acc: f32,
    frames: i32,
}

impl TextApp {
    fn new() -> Self {
        Self {
            bouncers: Vec::with_capacity(TEXT_COUNT),
            screen_size: Vec2::new(800.0, 600.0),
            fps_counter_id: None,
            time_acc: 0.0,
            frames: 0,
        }
    }
    
    fn rand(seed: usize) -> f32 {
        ((seed as f32 * 12.9898).sin() * 43758.5453).fract()
    }
}

impl Application for TextApp {
    fn on_start(&mut self, mw: &mut MoonWalk, viewport: Vec2) {
        println!("🚀 Loading Text App...");
        self.screen_size = viewport;

        let font_id = match mw.load_font("font.ttf") {
            Ok(id) => {
                println!("Font loaded!");
                id
            },
            Err(_) => {
                println!("Font not found, using system default (ID=0)");
                moonwalk::FontAsset(0) 
            }
        };

        let bg = mw.new_rect();
        mw.set_position(bg, Vec2::ZERO);
        mw.set_size(bg, viewport * 2.0);
        mw.set_color(bg, Vec4::new(0.05, 0.05, 0.1, 1.0));
        mw.set_z_index(bg, 0.0);

        for i in 0..TEXT_COUNT {
            let size = 12.0 + Self::rand(i) * 20.0;
            let text_content = format!("Text #{}", i);
            
            let id = mw.new_text(&text_content, font_id, size);
            
            let x = Self::rand(i + 1) * viewport.x;
            let y = Self::rand(i + 2) * viewport.y;
            
            let color = Vec4::new(
                0.5 + Self::rand(i + 3) * 0.5,
                0.5 + Self::rand(i + 4) * 0.5,
                0.5 + Self::rand(i + 5) * 0.5,
                1.0
            );

            mw.set_position(id, Vec2::new(x, y));
            mw.set_color(id, color);
            mw.set_z_index(id, 0.5);

            self.bouncers.push(TextBouncer {
                id,
                pos: Vec2::new(x, y),
                vel: Vec2::new(
                    (Self::rand(i + 6) - 0.5) * 200.0,
                    (Self::rand(i + 7) - 0.5) * 200.0
                ),
                text: text_content,
            });
        }

        let fps = mw.new_text("FPS: 0", font_id, 64.0);
        mw.set_position(fps, Vec2::new(20.0, 20.0));
        mw.set_color(fps, Vec4::new(1.0, 0.8, 0.2, 1.0));
        mw.set_z_index(fps, 1.0);
        
        self.fps_counter_id = Some(fps);
    }

    fn on_update(&mut self, dt: f32) {
        let w = self.screen_size.x;
        let h = self.screen_size.y;

        for b in &mut self.bouncers {
            b.pos += b.vel * dt;

            if b.pos.x < 0.0 || b.pos.x > w { b.vel.x *= -1.0; }
            if b.pos.y < 0.0 || b.pos.y > h { b.vel.y *= -1.0; }
        }

        self.frames += 1;
        self.time_acc += dt;
        if self.time_acc >= 1.0 {
            let fps = self.frames;
            println!("FPS: {}", fps);
            
            self.frames = 0;
            self.time_acc = 0.0;
        }
    }

    fn on_draw(&mut self, mw: &mut MoonWalk) {
        for b in &self.bouncers {
            mw.set_position(b.id, b.pos);
        }
        
        if let Some(id) = self.fps_counter_id {
            // mw.set_text(id, &format!("Obj: {}", TEXT_COUNT)); 
        }
    }

    fn on_resize(&mut self, mw: &mut MoonWalk, viewport: Vec2) {
        self.screen_size = viewport;
    }
}

#[cfg(not(target_os = "android"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = TextApp::new();
    let settings = WindowSettings::new("MoonWalk Text Stress", 1280.0, 720.0).resizable(true);
    Runner::run(app, settings)
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info)
    );
    
    let app = TextApp::new();
    let settings = WindowSettings::new("MoonWalk Android", 0.0, 0.0);
    Runner::run(app, settings, app).unwrap();
}

#[cfg(target_os = "android")]
fn main() {}
