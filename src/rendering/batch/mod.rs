mod common;
mod rect;
mod text;
mod bezier;

use std::collections::HashMap;
use easy_gpu::{Context, MatrixStack, RenderPass};
use textware::TextWare;
use crate::objects::{ObjectStore, Variant, ShaderId, hash_uniforms, Object, ObjectId};
use crate::rendering::shader::ShaderStore;

use self::common::{BatchGroup, RenderLayer};
use self::rect::RectBatcher;
use self::text::TextBatcher;
use self::bezier::BezierBatcher;

pub struct BatchSystem {
    batches: HashMap<RenderLayer, Vec<BatchGroup>>,
    projection_buffer: Option<easy_gpu::Buffer<easy_gpu::matrix::MatrixUniform>>,
    projection_bind_group: Option<wgpu::BindGroup>,
}

impl BatchSystem {
    pub fn new() -> Self {
        Self {
            batches: HashMap::new(),
            projection_buffer: None,
            projection_bind_group: None,
        }
    }

    pub fn rebuild(
        &mut self,
        ctx: &Context,
        store: &mut ObjectStore,
        textware: &mut TextWare,
        screen_w: u32,
        screen_h: u32,
    ) {
        self.batches.clear();

        // Step 1: Group objects keys (using immutable access)
        // Key -> Vec<ObjectId>
        let mut map: HashMap<(RenderLayer, ShaderId, u64, Option<[u32; 4]>), Vec<ObjectId>> = HashMap::new();
        
        {
            let objects = store.get_objects();
            for obj in objects.values() {
                 let layer = match obj.variant {
                     Variant::Rect(_) => RenderLayer::Simple,
                     Variant::Text(_) => RenderLayer::Glyph,
                     Variant::Bezier(_) => RenderLayer::Simple,
                 };
                 let shader = store.get_default_shader_for(obj);
                 let u_hash = hash_uniforms(&obj.common.uniforms);
                 let scissor = common::calculate_scissor(store, obj, screen_w, screen_h);
                 
                 let key = if let Variant::Bezier(_) = obj.variant {
                     (layer, shader, obj.id.to_u32() as u64, scissor)
                 } else {
                     (layer, shader, u_hash, scissor)
                 };
                 
                 map.entry(key).or_insert(Vec::new()).push(obj.id);
            }
        }

        // Step 2: Build batches (need mutable access for Text)
        for (key, object_ids) in map {
            let (layer, shader_id, _, scissor) = key;
            
            // We need to collect objects. For Text we need &mut, for others & is enough.
            // But getting multiple &mut from HashMap is tricky safely without unsafe or splitting.
            // Since we process one batch at a time, we can just get mutable references one by one?
            // No, batchers take a slice.
            // We will use unsafe to get multiple mutable references from the store 
            // knowing that object_ids are unique and distinct.
            
            // Alternatively, for Rect/Bezier we can just use immutable store get.
            // For Text, we really need mutable access to update layout cache.
            
            let batch = match layer {
                RenderLayer::Simple => {
                    // Immutable access is fine
                    let objects_ref = store.get_objects();
                    let mut batch_objects: Vec<&Object> = Vec::with_capacity(object_ids.len());
                    for id in &object_ids {
                        if let Some(obj) = objects_ref.get(id) {
                            batch_objects.push(obj);
                        }
                    }
                    
                    if batch_objects.is_empty() { continue; }

                    let first_variant = &batch_objects[0].variant;
                    match first_variant {
                        Variant::Rect(_) => {
                            RectBatcher::build(ctx, shader_id, &batch_objects, scissor)
                        },
                        Variant::Bezier(_) => {
                            BezierBatcher::build(ctx, shader_id, &batch_objects, scissor)
                        },
                        _ => None
                    }
                },
                RenderLayer::Glyph => {
                     // Mutable access needed
                     let objects_map = store.get_objects_mut();
                     let mut batch_objects_mut: Vec<&mut Object> = Vec::with_capacity(object_ids.len());
                     
                     // SAFETY: We know ids are distinct because they come from keys of the hashmap (unique objects).
                     // However, Rust borrow checker doesn't know that.
                     // We will use raw pointers to bypass borrow checker for this specific slice creation.
                     let map_ptr = objects_map as *mut HashMap<ObjectId, Object>;
                     
                     for id in &object_ids {
                         unsafe {
                             if let Some(obj) = (*map_ptr).get_mut(id) {
                                 batch_objects_mut.push(obj);
                             }
                         }
                     }
                     
                     if batch_objects_mut.is_empty() { continue; }

                     TextBatcher::build(ctx, textware, shader_id, batch_objects_mut.as_mut_slice(), scissor)
                }
            };

            if let Some(b) = batch {
                self.batches.entry(layer).or_default().push(b);
            }
        }
        
        for list in self.batches.values_mut() {
            list.sort_by(|a, b| a.sort_key.partial_cmp(&b.sort_key).unwrap());
        }
    }

    pub fn render<'a>(
        &'a mut self,
        pass: &mut RenderPass<'a>,
        shader_store: &'a ShaderStore,
        textware: &'a TextWare,
        matrices: &MatrixStack,
        ctx: &Context,
    ) {
        let proj_uniform = matrices.to_uniform();
        
        if self.projection_buffer.is_none() {
            let buf = easy_gpu::Buffer::uniform(ctx, &proj_uniform);
            let bg = shader_store.get_proj_bind_group(ctx, &buf);
            self.projection_buffer = Some(buf);
            self.projection_bind_group = Some(bg);
        } else {
            self.projection_buffer.as_mut().unwrap().update_one(ctx, &proj_uniform);
        }
        
        let bind_0 = self.projection_bind_group.as_ref().unwrap();

        if let Some(list) = self.batches.get(&RenderLayer::Simple) {
            for batch in list {
                 if let Some(pipeline) = shader_store.get_pipeline(batch.shader_id) {
                     pass.set_pipeline(pipeline);
                     if let Some(rect) = batch.scissor {
                        pass.set_scissor(rect[0], rect[1], rect[2], rect[3]);
                     }
                     
                     if batch.bind_group_uniforms.is_some() {
                         if let Some(bg) = &batch.bind_group_uniforms {
                             pass.set_bind_group(0, bg);
                         }
                     } else {
                         pass.set_bind_group(0, bind_0);
                     }

                     if let Some(vbo) = &batch.vbo {
                         pass.set_vertex_buffer(0, vbo);
                     }
                     
                     pass.draw(batch.vertex_count as u32);
                 }
            }
        }

        if let Some(list) = self.batches.get(&RenderLayer::Glyph) {
             let atlas_bind_group = textware.get_bind_group();
             
             for batch in list {
                if let Some(pipeline) = shader_store.get_pipeline(batch.shader_id) {
                    pass.set_pipeline(pipeline);
                    if let Some(rect) = batch.scissor {
                        pass.set_scissor(rect[0], rect[1], rect[2], rect[3]);
                    }

                    pass.set_bind_group(0, bind_0);
                    pass.set_bind_group(1, atlas_bind_group);
                    
                    if let Some(vbo) = &batch.vbo {
                        pass.set_vertex_buffer(0, vbo);
                    }
                    pass.draw(batch.vertex_count as u32);
                }
             }
        }
    }
}