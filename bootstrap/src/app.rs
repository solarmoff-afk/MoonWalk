use glam::Vec2;
use moonwalk::MoonWalk;

pub trait Application {
    fn on_start(&mut self, mw: &mut MoonWalk, viewport: Vec2);

    fn on_update(&mut self, dt: f32);

    fn on_resize(&mut self, mw: &mut MoonWalk, new_viewport: Vec2);

    fn on_draw(&mut self, mw: &mut MoonWalk);

    fn on_exit(&mut self) {}
}