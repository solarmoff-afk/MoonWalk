// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

pub mod core;
pub mod render;
pub mod pipeline;
pub mod error;

pub struct BackendInstance {
    label: String,
}

impl BackendInstance {
    pub fn new(label: &str) -> Self {
        Self {
            // Тут String чтобы не думать о лайфтаймах при хранении str
            label: label.to_string(),
        }
    }
}