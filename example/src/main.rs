use moonwalk::{MoonWalk, ObjectId};
use moonwalk_bootstrap::{Application, Runner, WindowSettings};
use glam::{Vec2, Vec4};

#[cfg(target_os = "android")]
use log::LevelFilter;

const RECT_COUNT: usize = 700_000; 
const RECT_SIZE: f32 = 1.0;
const SPEED_BASE: f32 = 150.0;

struct Bouncer {
    id: ObjectId,
    pos: Vec2,
    vel: Vec2,
    color: Vec4,
    rot_speed: f32,
    angle: f32,
}

struct StressApp {
    bouncers: Vec<Bouncer>,
    screen_size: Vec2,
}

impl StressApp {
    fn new() -> Self {
        Self {
            bouncers: Vec::with_capacity(RECT_COUNT),
            screen_size: Vec2::new(1024.0, 768.0),
        }
    }

    fn pseudo_rand(seed: usize, offset: f32) -> f32 {
        ((seed as f32 * 12.9898 + offset).sin() * 43758.5453).fract()
    }
}

impl Application for StressApp {
    fn on_start(&mut self, mw: &mut MoonWalk, viewport: Vec2) {
        self.screen_size = viewport;
        
        #[cfg(target_os = "android")]
        log::info!("MoonWalk Start. Viewport: {:?}", viewport);

        let bg = mw.new_rect();
        mw.set_position(bg, Vec2::ZERO);
        mw.set_size(bg, viewport * 2.0);
        mw.set_color(bg, Vec4::new(0.0, 0.0, 0.0, 1.0));
        mw.set_z_index(bg, 0.0);

        for i in 0..RECT_COUNT {
            let id = mw.new_rect();
            
            let r = Self::pseudo_rand(i, 1.0);
            let g = Self::pseudo_rand(i, 2.0);
            let b = Self::pseudo_rand(i, 3.0);

            let start_x = Self::pseudo_rand(i, 4.0) * (viewport.x - RECT_SIZE);
            let start_y = Self::pseudo_rand(i, 5.0) * (viewport.y - RECT_SIZE);
            
            let vel_x = (Self::pseudo_rand(i, 6.0) - 0.5) * 2.0 * SPEED_BASE;
            let vel_y = (Self::pseudo_rand(i, 7.0) - 0.5) * 2.0 * SPEED_BASE;

            let color = Vec4::new(r, g, b, 1.0);
            let size = Vec2::splat(RECT_SIZE + Self::pseudo_rand(i, 8.0) * 15.0);

            mw.set_position(id, Vec2::new(start_x, start_y));
            mw.set_size(id, size);
            mw.set_color(id, color);
            mw.set_rounded(id, Vec4::splat(8.0));
            mw.set_z_index(id, 0.1 + (i as f32 / RECT_COUNT as f32)); 

            self.bouncers.push(Bouncer {
                id,
                pos: Vec2::new(start_x, start_y),
                vel: Vec2::new(vel_x, vel_y),
                color,
                rot_speed: (Self::pseudo_rand(i, 9.0) - 0.5) * 5.0,
                angle: 0.0,
            });
        }
    }

    fn on_update(&mut self, dt: f32) {
        let w = self.screen_size.x;
        let h = self.screen_size.y;

        for b in &mut self.bouncers {
            b.pos += b.vel * dt;
            b.angle += b.rot_speed * dt;

            if b.pos.x < 0.0 {
                b.pos.x = 0.0;
                b.vel.x = b.vel.x.abs();
            } else if b.pos.x > w {
                b.pos.x = w;
                b.vel.x = -b.vel.x.abs();
            }

            if b.pos.y < 0.0 {
                b.pos.y = 0.0;
                b.vel.y = b.vel.y.abs();
            } else if b.pos.y > h {
                b.pos.y = h;
                b.vel.y = -b.vel.y.abs();
            }
        }
    }

    fn on_draw(&mut self, mw: &mut MoonWalk) {
        for b in &self.bouncers {
            mw.set_position(b.id, b.pos);
            mw.set_rotation(b.id, b.angle);
        }
    }

    fn on_resize(&mut self, mw: &mut MoonWalk, viewport: Vec2) {
        self.screen_size = viewport;
    }
}

#[cfg(not(target_os = "android"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = StressApp::new();
    let settings = WindowSettings::new("MoonWalk Stress", 1280.0, 720.0).resizable(true);
    Runner::run(app, settings)
}

#[cfg(target_os = "android")]
use android_activity::AndroidApp;

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(LevelFilter::Info)
    );
    let stress_app = StressApp::new();
    let settings = WindowSettings::new("MoonWalk Android", 0.0, 0.0);
    Runner::run(stress_app, settings, app).unwrap();
}

#[cfg(target_os = "android")]
fn main() {}
