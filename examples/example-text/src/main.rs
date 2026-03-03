use moonwalk::{MoonWalk, ObjectId};
use moonwalk_bootstrap::{Application, Runner, WindowSettings};
use glam::{Vec2, Vec4};

#[cfg(target_os = "android")]
use android_activity::AndroidApp;

struct TextApp {
    _angle: f32,
    screen_rect_id: Option<ObjectId>,
}

impl TextApp {
    fn new() -> Self {
        Self { 
            _angle: 0.0,
            screen_rect_id: None,
        }
    }
}

impl Application for TextApp { 
    fn on_start(&mut self, mw: &mut MoonWalk, viewport: Vec2) {
        let mars_texture_id = mw.load_texture("assets/test.jpg").unwrap(); 
        let regular_font = mw.load_font("assets/Hundo.ttf", "Hundo").unwrap();

        let content = mw.load_utf8("assets/text.txt").unwrap();

        let text_id = mw.new_text(content.as_str(), regular_font, 48.0); 
        mw.set_position(text_id, Vec2::new(10.0, 10.0)); 
        mw.set_color(text_id, Vec4::new(1.0, 1.0, 1.0, 1.0));
        
        let result_obj = mw.new_rect();
        // mw.set_position(result_obj, Vec2::new(100.0, 100.0));
        mw.set_size(result_obj, Vec2::new(viewport.x, viewport.y));
        mw.set_texture(result_obj, mars_texture_id);

        self.screen_rect_id = Some(result_obj);
    }

    fn on_resize(&mut self, mw: &mut MoonWalk, viewport: Vec2) {
        if let Some(id) = self.screen_rect_id {
            mw.set_size(id, viewport);
        }
    }
  
    fn on_update(&mut self, _dt: f32) {
    }

    fn on_pre_render(&mut self) -> Option<Vec4> {
        Some(Vec4::new(1.0, 1.0, 1.0, 1.0))
    } 

    fn on_draw(&mut self, _mw: &mut MoonWalk) {
        
    }
}

#[cfg(not(target_os = "android"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = TextApp::new();
    let settings = WindowSettings::new("MoonWalk Text Test", 800.0, 600.0).resizable(true);
    Runner::run(app, settings)
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info)
    );

    log::info!("MoonWalk: android_main started");

    let stress_app = TextApp::new();
    let settings = WindowSettings::new("MoonWalk Android", 0.0, 0.0);
    Runner::run(stress_app, settings, app).unwrap();
}

#[cfg(target_os = "android")]
fn main() {}
