
use moonwalk::{MoonWalk, ObjectId, TextAlign, TextureId};
use moonwalk_bootstrap::{Application, Runner, WindowSettings, TouchPhase};
use glam::{Vec2, Vec4};

#[cfg(target_os = "android")]
use android_activity::AndroidApp;

struct LiquidGlassApp {
    input_texture: TextureId,
    output_texture: TextureId,
    mask_texture: TextureId,
    screen_rect_id: Option<ObjectId>,
    screen_size: Vec2,
    glass_center: Vec2,
}

impl LiquidGlassApp {
    fn new() -> Self {
        Self {
            input_texture: TextureId::new(0),
            output_texture: TextureId::new(0),
            mask_texture: TextureId::new(0),
            screen_rect_id: None,
            screen_size: Vec2::ZERO,
            glass_center: Vec2::new(400.0, 300.0),
        }
    }
}

impl Application for LiquidGlassApp {
    fn on_start(&mut self, mw: &mut MoonWalk, viewport: Vec2) {
        self.screen_size = viewport;
        self.glass_center = viewport * 0.5;

        let bg_texture = mw.load_texture("assets/test.jpg").unwrap();

        let mut bg_container = mw.new_render_container(viewport.x as u32, viewport.y as u32);
        let bg_rect = bg_container.new_rect();
        bg_container.set_position(bg_rect, Vec2::ZERO);
        bg_container.set_size(bg_rect, viewport);

        if bg_texture > 0 {
            bg_container.set_texture(bg_rect, bg_texture);
        } else {
            let cs = 50.0;

            for i in 0..((viewport.x / cs) as i32 + 2) {
                for j in 0..((viewport.y / cs) as i32 + 2) {
                    let color = if (i + j) % 2 == 0 {
                        Vec4::new(0.88, 0.88, 0.88, 1.0)
                    } else {
                        Vec4::new(0.22, 0.22, 0.22, 1.0)
                    };

                    let tile = bg_container.new_rect();
                    bg_container.set_position(tile, Vec2::new(i as f32 * cs, j as f32 * cs));
                    bg_container.set_size(tile, Vec2::splat(cs));
                    bg_container.set_color(tile, color);
                }
            }
        }
        bg_container.draw(mw, Some(Vec4::ZERO));

        self.input_texture = bg_container.snapshot(mw, 0, 0, viewport.x as u32, viewport.y as u32);
        self.output_texture = bg_container.snapshot(mw, 0, 0, viewport.x as u32, viewport.y as u32);

        let bg_under_glass = mw.new_rect();
        mw.set_position(bg_under_glass, Vec2::ZERO);
        mw.set_size(bg_under_glass, viewport);
        mw.set_texture(bg_under_glass, bg_texture);

        let mask_width = 700u32;
        let mask_height = 180u32;

        let mut mask_container = mw.new_render_container(mask_width, mask_height);

        let font = mw.load_font("assets/Hundo.ttf", "Hundo").unwrap();

        let text_id = mask_container.new_text("GLASS", font, 92.0);
        mask_container.config_position(text_id, Vec2::new(10.0, 10.0));
        mask_container.config_color(text_id, Vec4::new(1.0, 1.0, 1.0, 1.0));

        mask_container.draw(mw, Some(Vec4::ZERO));

        self.mask_texture = mask_container.snapshot(mw, 0, 0, mask_width, mask_height);

        mw.sdf_mask(self.mask_texture, 20.0, 1.0);

        mw.save_texture(self.mask_texture, "assets/mask_debug.png");

        let screen_rect = mw.new_rect();
        mw.set_position(screen_rect, Vec2::ZERO);
        mw.set_size(screen_rect, viewport);
        mw.set_texture(screen_rect, self.output_texture);
        self.screen_rect_id = Some(screen_rect);
    }

    fn on_draw(&mut self, mw: &mut MoonWalk) {
        if self.input_texture.0 == 0 || self.mask_texture.0 == 0 || self.output_texture.0 == 0 {
            return;
        }

        let glass_size = Vec2::new(520.0, 270.0);
        let glass_pos = self.glass_center - glass_size * 0.5;

        mw.liquid_glass_mask(
            self.input_texture,
            self.mask_texture,
            self.output_texture,
            glass_size,
            glass_pos,
            20.0,
            -20.0,
            0.20,
            0.1,
            1.2,
            0.06,
            Vec4::ZERO,
        );
    }

    fn on_touch(&mut self, _mw: &mut MoonWalk, phase: TouchPhase, position: Vec2) {
        if matches!(phase, TouchPhase::Started | TouchPhase::Moved) {
            self.glass_center = position;
        }
    }

    fn on_resize(&mut self, mw: &mut MoonWalk, viewport: Vec2) {
        self.screen_size = viewport;
        if let Some(id) = self.screen_rect_id {
            mw.set_size(id, viewport);
        }
    }

    fn on_update(&mut self, _dt: f32) {

    }
}

#[cfg(not(target_os = "android"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = LiquidGlassApp::new();
    let settings = WindowSettings::new("Liquid Glass — точно как MARS пример", 1280.0, 720.0)
        .resizable(true);
    
    Runner::run(app, settings)
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info)
    );

    let instance = LiquidGlassApp::new();
    let settings = WindowSettings::new("Liquid Glass", 0.0, 0.0);
    Runner::run(instance, settings, app).unwrap();
}

#[cfg(target_os = "android")]
fn main() {}
