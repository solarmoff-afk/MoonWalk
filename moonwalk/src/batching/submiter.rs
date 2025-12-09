// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use crate::easy_gpu::{Context, RenderPass};
use crate::objects::store::ObjectStore;

/// Интерфейс который обязан реализовать любой модуль батчинга 
pub trait Submiter {
    /// Читает данные из стора (Хранилще объектов) сортирует их
    /// и заливает в gpu буферы
    fn prepare(&mut self, ctx: &Context, store: &ObjectStore);

    /// Записывает команды отрисовки в RenderPass
    fn render<'a>(&'a self, pass: &mut RenderPass<'a>);
}