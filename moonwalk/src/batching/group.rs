// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use std::collections::HashMap;

use crate::gpu::Context;
use crate::batching::shapes::uber::UberBatch;
use crate::objects::store::ObjectStore;
use crate::rendering::texture::Texture;

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

    pub fn render<'a>(&'a self, pass: &mut crate::gpu::RenderPass<'a>, white_texture: &'a Texture, textures: &'a HashMap<u32, Texture>) {
        self.objects.render(pass, white_texture, textures);
    }
}