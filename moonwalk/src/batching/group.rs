// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use crate::easy_gpu::Context;
use crate::batching::shapes::rect::RectBatch;
use crate::objects::store::ObjectStore;

pub struct BatchGroup {
    pub rects: RectBatch,
}

impl BatchGroup {
    pub fn new(ctx: &Context) -> Self {
        Self {
            rects: RectBatch::new(ctx),
        }
    }

    pub fn prepare(&mut self, ctx: &Context, store: &ObjectStore) {
        self.rects.prepare(ctx, store);
    }
}