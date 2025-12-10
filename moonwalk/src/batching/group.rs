// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use crate::easy_gpu::Context;
use crate::batching::shapes::uber::UberBatch;
use crate::objects::store::ObjectStore;

pub struct BatchGroup {
    pub objects: UberBatch,
}

impl BatchGroup {
    pub fn new(ctx: &Context) -> Self {
        Self {
            objects: UberBatch::new(ctx),
        }
    }

    pub fn prepare(&mut self, ctx: &Context, store: &ObjectStore) {
        self.objects.prepare(ctx, store);
    }
}