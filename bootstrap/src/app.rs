// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk


use glam::{Vec2, Vec4};
use moonwalk::MoonWalk;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TouchPhase {
    Started,
    Moved,
    Ended,
    Cancelled,
}

pub trait Application {
    fn on_start(&mut self, mw: &mut MoonWalk, viewport: Vec2);

    fn on_update(&mut self, dt: f32);

    fn on_resize(&mut self, mw: &mut MoonWalk, new_viewport: Vec2);

    fn on_draw(&mut self, mw: &mut MoonWalk);

    fn on_exit(&mut self) {}

    /// Вызывается перед рендером, может вернуть цвет для очистки экрана
    fn on_pre_render(&mut self) -> Option<Vec4> {
        None
    }

    fn on_touch(&mut self, _moonwalk: &mut MoonWalk, _phase: TouchPhase, _position: Vec2) {}
}
