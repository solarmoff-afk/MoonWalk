// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use std::collections::HashMap;

use moonwalk_backend::core::context::BackendContext;
use moonwalk_backend::pipeline::bind::RawBindGroup;
use moonwalk_backend::render::pass::RenderPass;
use moonwalk_backend::render::texture::BackendTexture;

use crate::batching::shapes::uber::UberBatch;
use crate::objects::store::ObjectStore;
use crate::ObjectId;

pub struct BatchGroup {
    pub objects: UberBatch,
}

impl BatchGroup {
    pub fn new(context: &mut BackendContext) -> Self {
        // [HACK]
        // Заменить на обработку ошибки
        Self {
            objects: UberBatch::new(context).expect("Context not created"),
        }
    }

    pub fn prepare(&mut self, context: &mut BackendContext, store: &ObjectStore, text_engine: &mut crate::textware::TextWare, objects_filter: Option<&Vec<ObjectId>>) {
        self.objects.prepare(context, store, text_engine, objects_filter);
    }
    
    pub fn render<'a>(
        &'a self,
        pass: &mut RenderPass<'a>,
        white_texture: &'a BackendTexture,
        textures: &'a HashMap<u32, BackendTexture>,
        atlas_bg: Option<&'a RawBindGroup>,
    ) {
        self.objects.render(pass, white_texture, textures, atlas_bg);
    }
}