use moonwalk::{MoonWalk, ObjectId, TextureId, FontAsset};
use moonwalk_bootstrap::{Application, Runner, WindowSettings};
use glam::{Vec2, Vec4};

#[cfg(target_os = "android")]
use android_activity::AndroidApp;

struct BezierApp {
    text_id: ObjectId,
    text2_id: ObjectId,
    curve_container_id: ObjectId,
    curve2_container_id: ObjectId,
    bg_id: ObjectId,
    font: Option<FontAsset>,
    curve_texture_id: Option<TextureId>,
    curve2_texture_id: Option<TextureId>,
    initialized: bool,
}

impl BezierApp {
    fn new() -> Self {
        Self {
            text_id: ObjectId(0),
            text2_id: ObjectId(0),
            curve_container_id: ObjectId(0),
            curve2_container_id: ObjectId(0),
            bg_id: ObjectId(0),
            font: None,
            curve_texture_id: None,
            curve2_texture_id: None,
            initialized: false,
        }
    }

    fn create_curve_texture(&mut self, mw: &mut MoonWalk, size: Vec2) -> TextureId {
        let mut pb = mw.new_path_builder();
        
        pb.set_color(Vec4::new(0.0, 1.0, 0.0, 1.0));
        pb.set_stroke(8.0);
        
        // S-образная кривая
        let start = Vec2::new(size.x * 0.2, size.y * 0.7);
        let end = Vec2::new(size.x * 0.8, size.y * 0.3);
        let ctrl1 = Vec2::new(size.x * 0.4, size.y * 0.2);
        let ctrl2 = Vec2::new(size.x * 0.6, size.y * 0.8);
        
        pb.move_to(start.x, start.y);
        pb.cubic_bezier_to(ctrl1.x, ctrl1.y, ctrl2.x, ctrl2.y, end.x, end.y);
        pb.end();
        
        let texture_id = pb.tessellate(mw, size.x as u32, size.y as u32);
        TextureId(texture_id)
    } 
}

impl Application for BezierApp {
    fn on_start(&mut self, mw: &mut MoonWalk, viewport: Vec2) {
        if self.font.is_none() {
            self.font = mw.load_font("assets/Roboto-Medium.ttf", "Roboto").ok();
        }

        if !self.initialized {
            self.bg_id = mw.new_rect();
            mw.set_color(self.bg_id, Vec4::new(0.05, 0.05, 0.05, 1.0));
            
            if let Some(font) = self.font {
                self.text_id = mw.new_text("PathBuilder", font, 64.0);
                mw.set_color(self.text_id, Vec4::new(1.0, 1.0, 1.0, 1.0));

                self.text2_id = mw.new_text("GpuPathBuilder", font, 64.0);
                mw.set_color(self.text2_id, Vec4::new(1.0, 1.0, 1.0, 1.0));
            }
            
            self.curve_container_id = mw.new_rect();
            mw.set_color(self.curve_container_id, Vec4::new(1.0, 1.0, 1.0, 1.0));
            mw.set_rounded(self.curve_container_id, Vec4::splat(8.0));
            
            self.curve2_container_id = mw.new_rect();
            mw.set_color(self.curve2_container_id, Vec4::new(1.0, 1.0, 1.0, 1.0));
            mw.set_rounded(self.curve2_container_id, Vec4::splat(8.0));
            
            self.initialized = true;
        }

        mw.set_position(self.bg_id, Vec2::ZERO);
        mw.set_size(self.bg_id, viewport);
        
        if let Some(font) = self.font {
            let text_size = mw.measure_text("PathBuilder", font, 64.0, viewport.x * 0.4);
            let text_pos = Vec2::new(50.0, 40.0);
            mw.set_position(self.text_id, text_pos);
            
            let text2_size = mw.measure_text("GpuPathBuilder", font, 64.0, viewport.x * 0.4);
            let text2_pos = Vec2::new(viewport.x - text2_size.x - 50.0, 40.0);
            mw.set_position(self.text2_id, text2_pos);
        }
        
        let container_size = Vec2::new(400.0, 300.0);
        let container_pos = Vec2::new(
            50.0,
            viewport.y * 0.5 - container_size.y * 0.5
        );
        
        mw.set_position(self.curve_container_id, container_pos);
        mw.set_size(self.curve_container_id, container_size);
        
        let container2_size = Vec2::new(400.0, 300.0);
        let container2_pos = Vec2::new(
            viewport.x - container2_size.x - 50.0,
            viewport.y * 0.5 - container2_size.y * 0.5
        );
        
        mw.set_position(self.curve2_container_id, container2_pos);
        mw.set_size(self.curve2_container_id, container2_size);
        
        let new_texture = self.create_curve_texture(mw, container_size);
        mw.set_texture(self.curve_container_id, new_texture);
        self.curve_texture_id = Some(new_texture);
         
        let new_texture2 = self.create_curve_texture(mw, container2_size);
        mw.set_texture(self.curve2_container_id, new_texture2);
        self.curve2_texture_id = Some(new_texture2);
    }

    fn on_update(&mut self, _dt: f32) {}

    fn on_draw(&mut self, _mw: &mut MoonWalk) {}

    fn on_resize(&mut self, mw: &mut MoonWalk, viewport: Vec2) {
        self.on_start(mw, viewport);
    }

    fn on_pre_render(&mut self) -> Option<Vec4> {
        Some(Vec4::new(0.05, 0.05, 0.05, 1.0))
    }
}

#[cfg(not(target_os = "android"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = BezierApp::new();
    let settings = WindowSettings::new("MoonWalk - PathBuilder vs GpuPathBuilder", 1200.0, 800.0)
        .resizable(true);
    Runner::run(app, settings)
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info)
    );
    let app = BezierApp::new();
    let settings = WindowSettings::new("MoonWalk Bezier", 0.0, 0.0);
    Runner::run(app, settings, app).unwrap();
}
