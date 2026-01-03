// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use crate::gpu::{Buffer, Context, RenderPass};
use crate::rendering::vertex::{QuadVertex, ObjectInstance};
use crate::rendering::texture::Texture;
use crate::objects::store::ObjectStore;
use crate::batching::common::BatchBuffer;
use crate::textware::TextWare;

#[derive(Debug, Clone, Copy)]
pub struct DrawCommand {
    pub texture_id: u32,
    pub start_index: u32,
    pub count: u32,
}

pub struct UberBatch {
    static_vbo: Buffer<QuadVertex>,
    static_ibo: Buffer<u32>,
    instance_vbo: Option<Buffer<ObjectInstance>>,
    blit_vbo: Buffer<ObjectInstance>,
    batch: BatchBuffer<ObjectInstance>,
    
    // Сохранение списка команд за кадр
    commands: Vec<DrawCommand>,
}

impl UberBatch {
    pub fn new(ctx: &Context) -> Self {
        let static_vbo = Buffer::vertex(ctx, &QuadVertex::QUAD);
        let static_ibo = Buffer::<u32>::index(ctx, &QuadVertex::INDICES);
        
        // Создаем буфер для blit с одним элементом
        let dummy_instance = [ObjectInstance {
            pos_size: [0.0; 4],
            uv: [0; 4],
            radii: [0; 4],
            gradient_data: [0; 4],
            extra: [0.0; 2],
            color: 0,
            color2: 0,
            type_id: 0,
            effect_data: [0; 2],
        }];
        let blit_vbo = Buffer::vertex(ctx, &dummy_instance);
        
        Self {
            static_vbo,
            static_ibo,
            instance_vbo: None,
            blit_vbo,
            batch: BatchBuffer::new(),
            commands: Vec::with_capacity(32),
        }
    }

    pub fn prepare(&mut self, ctx: &Context, store: &ObjectStore, text_engine: &mut TextWare) {
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

                effect_data: store.effect_data_cache[idx],
            });
        }

        // Отдельный цикл для батчинга глифов. Перед этим нужно точно знать что
        // атлас существует, иначе рендеринг просто бесполезен
        if let Some(atlas_id) = text_engine.atlas_id {
            for &global_id in store.text_ids.iter() {
                let idx = global_id.index();
                if !store.alive[idx] {
                    continue;
                }

                let text = &store.text_contents[idx];
                if text.is_empty() {
                    continue;
                }

                let align = store.text_aligns[idx];
                let glyphs = text_engine.collect_glyphs(
                    global_id.index() as u64,
                    text,
                    store.font_ids[idx],
                    store.font_sizes[idx],
                    store.text_bounds[idx].x,
                    store.text_bounds[idx].y,
                    align,
                );

                let pos = store.positions[idx];
                let color = store.colors_cache[idx];
                let z = store.z_indices[idx];
                let rot = store.rotations[idx];
                
                for (gx, gy, key) in glyphs {
                    if let Some((image, uv_rect)) = text_engine.glyph_cache.get_glyph(key, &mut text_engine.font_system) {
                        let w = image.placement.width as f32;
                        let h = image.placement.height as f32;
                        let left = image.placement.left as f32;
                        let top = image.placement.top as f32;

                        let x = pos.x + gx + left;
                        let y = pos.y + gy - top;

                        let (u, v, uw, vh) = uv_rect;
                        let uv_arr = [u, v, uw, vh];

                        // [MAYBE]
                        self.batch.push(ObjectInstance {
                            pos_size: [x, y, w, h],
                            uv: ObjectInstance::pack_uv(uv_arr),
                            radii: ObjectInstance::pack_radii([0.0; 4]),
                            gradient_data: store.gradient_data_cache[idx],
                            extra: [z, rot],
                            type_id: atlas_id, 
                            color: color,
                            color2: store.colors2_cache[idx],
                            effect_data: store.effect_data_cache[idx],
                        });
                    }
                }
            }
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

            // Обновляем или создаем буфер инстансов
            if let Some(vbo) = &mut self.instance_vbo {
                vbo.update(ctx, &self.batch.cpu_buffer);
            } else {
                self.instance_vbo = Some(Buffer::vertex(ctx, &self.batch.cpu_buffer));
            }
        }

        self.batch.upload(ctx);
    }

    pub fn render<'a>(
        &'a self,
        pass: &mut RenderPass<'a>,
        white_texture: &'a Texture,
        textures: &'a std::collections::HashMap<u32, Texture>,
        atlas_bind_group: Option<&'a wgpu::BindGroup>,
    ) {
        // Проверка есть ли данные для рендера
        if self.instance_vbo.is_none() || self.commands.is_empty() {
            return;
        }

        pass.set_vertex_buffer(0, &self.static_vbo);
        pass.set_vertex_buffer(1, self.instance_vbo.as_ref().unwrap());
        pass.set_index_buffer(&self.static_ibo);

        for cmd in &self.commands {
            if cmd.texture_id == 0 {
                pass.set_bind_group(1, &white_texture.bind_group);
            } else if cmd.texture_id == crate::textware::ATLAS_ID {
                if let Some(bg) = atlas_bind_group {
                    pass.set_bind_group(1, bg);
                } else {
                    // Если атлас потерялся, рисуем белым (чтобы не крашнулось)
                    pass.set_bind_group(1, &white_texture.bind_group);
                }
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

    /// Рисует текстуру на весь экран
    pub fn blit<'a>(
        &'a mut self, 
        ctx: &Context, 
        pass: &mut RenderPass<'a>, 
        texture: &'a Texture, 
        screen_width: u32, 
        screen_height: u32
    ) {
        let instance = ObjectInstance {
            pos_size: [0.0, 0.0, screen_width as f32, screen_height as f32],
            uv: ObjectInstance::pack_uv([0.0, 0.0, 1.0, 1.0]),
            radii: ObjectInstance::pack_radii([0.0; 4]),
            type_id: 1, 
            color: ObjectInstance::pack_color([1.0, 1.0, 1.0, 1.0]),
            color2: 0,
            gradient_data: ObjectInstance::pack_gradient([0.0, 0.0, -1.0, 0.0]),
            extra: [0.0, 0.0],
            effect_data: ObjectInstance::pack_effects(0.0, 0.0),
        };

        self.blit_vbo.update(ctx, &[instance]);

        // Отрисовка буфера
        pass.set_vertex_buffer(0, &self.static_vbo);
        pass.set_vertex_buffer(1, &self.blit_vbo);
        pass.set_index_buffer(&self.static_ibo);
        pass.set_bind_group(1, &texture.bind_group);
        pass.draw_indexed_instanced_extended(
            6,
            1,
            0,
            0,
            0,
        );
    }
}