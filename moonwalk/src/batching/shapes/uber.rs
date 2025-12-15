// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use crate::gpu::{Buffer, Context, RenderPass};
use crate::rendering::vertex::{QuadVertex, ObjectInstance};
use crate::objects::store::ObjectStore;
use crate::batching::common::BatchBuffer;
use crate::rendering::texture::Texture;

#[derive(Debug, Clone, Copy)]
pub struct DrawCommand {
    pub texture_id: u32,
    pub start_index: u32,
    pub count: u32,
}

pub struct UberBatch {
    static_vbo: Buffer<QuadVertex>,
    static_ibo: Buffer<u32>,
    batch: BatchBuffer<ObjectInstance>,

    // Сохранение списка команд за кадр
    commands: Vec<DrawCommand>,
}

impl UberBatch {
    pub fn new(ctx: &Context) -> Self {
        let static_vbo = Buffer::vertex(ctx, &QuadVertex::QUAD);
        let static_ibo = Buffer::<u32>::index(ctx, &QuadVertex::INDICES);

        Self {
            static_vbo,
            static_ibo,
            batch: BatchBuffer::new(),
            commands: Vec::with_capacity(32),
        }
    }

    pub fn prepare(&mut self, ctx: &Context, store: &ObjectStore) {
        if !store.dirty {
            return;
        }

        self.batch.clear();
        self.commands.clear();
        
        for &global_id in store.rect_ids.iter() {
            let idx = global_id.index();

            if !store.alive[idx] {
                continue;
            }

            let tex_id = store.texture_ids[idx];

            self.batch.push(ObjectInstance {
                // Упаковываем позицию и размер в один вектор
                // для оптимизации
                pos_size: [
                    store.positions[idx].x,
                    store.positions[idx].y,
                    store.sizes[idx].x,
                    store.sizes[idx].y,
                ],

                radii: store.rect_radii_cache[idx],

                uv: store.uvs_cache[idx],

                type_id: tex_id, 

                // Упаковываем z индекс и вращение
                extra: [
                    store.z_indices[idx],
                    store.rotations[idx],
                ],

                color: store.colors_cache[idx],

                color2: store.colors2_cache[idx],

                gradient_data: store.gradient_data_cache[idx],
            });
        }
        
        self.batch.sort();

        if !self.batch.cpu_buffer.is_empty() {
            // Получение текстуры. Если 0 - просто объект без текстуры
            let mut current_tex = self.batch.cpu_buffer[0].type_id;
            
            let mut start = 0;
            let mut count = 0;

            for (i, instance) in self.batch.cpu_buffer.iter().enumerate() {
                // Если текстура сменилась то текущая команда закрывается
                if instance.type_id != current_tex {
                    self.commands.push(DrawCommand {
                        texture_id: current_tex,
                        start_index: start,
                        count,
                    });

                    // Начинается новая команда
                    current_tex = instance.type_id;
                    start = i as u32;
                    count = 0;
                }

                count += 1;
            }
            
            self.commands.push(DrawCommand {
                texture_id: current_tex,
                start_index: start,
                count,
            });
        }

        self.batch.upload(ctx);
    }

    pub fn render<'a>(
            &'a self,
            pass: &mut RenderPass<'a>,
            white_texture: &'a Texture,
            textures: &'a std::collections::HashMap<u32, Texture>,
        ) {
        if let Some(inst_buf) = &self.batch.gpu_buffer {
            if self.commands.is_empty() {
                return;
            }

            pass.set_vertex_buffer(0, &self.static_vbo);
            pass.set_vertex_buffer(1, inst_buf);
            pass.set_index_buffer(&self.static_ibo);

            for cmd in &self.commands {
                if cmd.texture_id == 0 {
                    pass.set_bind_group(1, &white_texture.bind_group);
                } else {
                    if let Some(tex) = textures.get(&cmd.texture_id) {
                        pass.set_bind_group(1, &tex.bind_group);
                    } else {
                        // Текстуры нет, а значит нужно вернуть белую текстуру
                        pass.set_bind_group(1, &white_texture.bind_group);
                    }
                }

                pass.draw_indexed_instanced_extended(
                    6, 
                    cmd.count, 
                    0, 
                    0, 
                    cmd.start_index
                );
            }
        }
    }
}