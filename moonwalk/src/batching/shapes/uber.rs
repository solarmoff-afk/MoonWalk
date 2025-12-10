// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use crate::easy_gpu::{Buffer, Context, RenderPass};
use crate::rendering::vertex::{QuadVertex, ObjectInstance};
use crate::objects::store::ObjectStore;
use crate::batching::common::BatchBuffer;

pub struct UberBatch {
    static_vbo: Buffer<QuadVertex>,
    static_ibo: Buffer<u32>,
    batch: BatchBuffer<ObjectInstance>,
}

impl UberBatch {
    pub fn new(ctx: &Context) -> Self {
        let static_vbo = Buffer::vertex(ctx, &QuadVertex::QUAD);
        let static_ibo = Buffer::<u32>::index(ctx, &QuadVertex::INDICES);

        Self {
            static_vbo,
            static_ibo,
            batch: BatchBuffer::new(),
        }
    }

    pub fn prepare(&mut self, ctx: &Context, store: &ObjectStore) {
        if !store.dirty {
            return;
        }

        self.batch.clear();
        
        for &global_id in store.rect_ids.iter() {
            let idx = global_id.index();

            if !store.alive[idx] {
                continue;
            }

            self.batch.push(ObjectInstance {
                // Упаковываем позицию и размер в один вектор
                // для оптимизации
                pos_size: [
                    store.positions[idx].x,
                    store.positions[idx].y,
                    store.sizes[idx].x,
                    store.sizes[idx].y,
                ],

                radii: store.rect_radii[idx].to_array(),

                // Временные значения
                uv: [0.0, 0.0, 1.0, 1.0], 
                type_id: 0,

                // Упаковываем z индекс и вращение
                extra: [
                    store.z_indices[idx],
                    store.rotations[idx],
                ],

                color: ObjectInstance::pack_color(store.colors[idx].to_array()),
            });
        }
        
        self.batch.sort();

        self.batch.upload(ctx);
    }

    pub fn render<'a>(&'a self, pass: &mut RenderPass<'a>) {
        if let Some(inst_buf) = &self.batch.gpu_buffer {
            let count = self.batch.cpu_buffer.len() as u32;
            
            if count > 0 {
                pass.set_vertex_buffer(0, &self.static_vbo);
                pass.set_vertex_buffer(1, inst_buf);
                pass.set_index_buffer(&self.static_ibo);
                pass.draw_indexed_instanced(6, count);
            }
        }
    }
}