// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use std::collections::HashMap;

#[cfg(feature = "modern")]
use moonwalk_backend::core::context::BackendContext;

#[cfg(feature = "modern")]
use moonwalk_backend::pipeline::bind::BindGroup;

#[cfg(feature = "modern")]
use moonwalk_backend::pipeline::bind::RawBindGroup;

#[cfg(feature = "modern")]
use moonwalk_backend::render::pass::RenderPass;

#[cfg(feature = "modern")]
use moonwalk_backend::render::texture::BackendTexture;

#[cfg(not(feature = "modern"))]
use crate::gpu::Context;

use crate::batching::shapes::uber::UberBatch;
use crate::objects::store::ObjectStore;

#[cfg(not(feature = "modern"))]
use crate::rendering::texture::Texture;

pub struct BatchGroup {
    pub objects: UberBatch,
}

impl BatchGroup {
    #[cfg(feature = "modern")]
    pub fn new(context: &mut BackendContext) -> Self {
        // [HACK]
        // Заменить на обработку ошибки
        Self {
            objects: UberBatch::new(context).expect("Context not created"),
        }
    }

    #[cfg(not(feature = "modern"))]
    pub fn new(ctx: &Context) -> Self {
        Self {
            objects: UberBatch::new(ctx),
        }
    }

    #[cfg(feature = "modern")]
    pub fn prepare(&mut self, context: &mut BackendContext, store: &ObjectStore, text_engine: &mut crate::textware::TextWare) {
        self.objects.prepare(context, store, text_engine);
    }
    
    #[cfg(not(feature = "modern"))]
    pub fn prepare(&mut self, ctx: &Context, store: &ObjectStore, text_engine: &mut crate::textware::TextWare) {
        self.objects.prepare(ctx, store, text_engine);
    }

    #[cfg(feature = "modern")]
    pub fn render<'a>(
        &'a self,
        pass: &mut RenderPass<'a>,
        white_texture: &'a BackendTexture,
        textures: &'a HashMap<u32, BackendTexture>,
        atlas_bg: Option<&'a RawBindGroup>,
    ) {
        self.objects.render(pass, white_texture, textures, atlas_bg);
    }

    #[cfg(not(feature = "modern"))]
    pub fn render<'a>(
        &'a self,
        pass: &mut crate::gpu::RenderPass<'a>,
        white_texture: &'a Texture,
        textures: &'a HashMap<u32, Texture>,
        atlas_bg: Option<&'a wgpu::BindGroup>,
    ) {
        self.objects.render(pass, white_texture, textures, atlas_bg);
    }
}