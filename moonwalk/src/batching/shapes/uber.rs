// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use moonwalk_backend::core::buffer::BackendBuffer;
use moonwalk_backend::core::context::BackendContext;
use moonwalk_backend::pipeline::bind::RawBindGroup;
use moonwalk_backend::render::pass::RenderPass;
use moonwalk_backend::render::texture::BackendTexture;

use crate::rendering::vertex::{QuadVertex, ObjectInstance};
use crate::objects::store::ObjectStore;
use crate::batching::common::BatchBuffer;
use crate::textware::TextWare;
use crate::MoonWalkError;
use crate::{perf_start, perf_end};
use crate::ObjectId;

#[derive(Debug, Clone, Copy)]
pub struct DrawCommand {
    pub texture_id: u32,
    pub start_index: u32,
    pub count: u32,
}

pub struct UberBatch {
    static_vbo: BackendBuffer<QuadVertex>,
    static_ibo: BackendBuffer<u32>,
    instance_vbo: Option<BackendBuffer<ObjectInstance>>,
    blit_vbo: BackendBuffer<ObjectInstance>,
    batch: BatchBuffer<ObjectInstance>,
    
    // Сохранение списка команд за кадр
    commands: Vec<DrawCommand>,
}

impl UberBatch {
    pub fn new(context: &mut BackendContext) -> Result<Self, MoonWalkError> {
        let static_vbo = BackendBuffer::vertex(context, &QuadVertex::QUAD)?;
        let static_ibo = BackendBuffer::<u32>::index(context, &QuadVertex::INDICES)?;
        
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

        let blit_vbo = BackendBuffer::vertex(
            context, &dummy_instance
        )?;
        
        Ok(Self {
            static_vbo,
            static_ibo,
            instance_vbo: None,
            blit_vbo,
            batch: BatchBuffer::new(),
            commands: Vec::with_capacity(32),
        })
    }    

    pub fn prepare(
        &mut self,
        context: &mut BackendContext,
        store: &ObjectStore,
        text_engine: &mut TextWare,
        objects_filter: Option<&Vec<ObjectId>>,
    ) {
        if !store.dirty {
            return;
        }

        self.batch.clear();
        self.commands.clear();
        
        // Сборка объектов для батча, сюда не попадают мёртвые объекты
        // либо объекты которых нет в фильтре объектов. Если фильтр
        // объектов пустой то в батч попадают все живые объекты
        for &global_id in store.rect_ids.iter() {
            let idx = global_id.index();

            if !store.alive[idx] {
                continue;
            }

            if let Some(objects) = objects_filter {
                if objects.contains(&ObjectId(idx)) {
                    continue;
                }
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
                
                // Опять таки, SoA архитектура не позволяет нормально удалять объекты,
                // поэтому для оптимизации (время на аллокации) и всего такого просто
                // помечаем объекты как живой/не живой и другой объект занимает его
                // место
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
                
                for (gx, gy, key) in glyphs.0 {
                    if let Some((image, uv_rect)) = text_engine.glyph_cache.get_glyph(key, &mut text_engine.font_system) {
                        let w = image.placement.width as f32;
                        let h = image.placement.height as f32;
                        let left = image.placement.left as f32;
                        let top = image.placement.top as f32;

                        let x = pos.x + gx + left;
                        let y = pos.y + gy - top;

                        let (u, v, uw, vh) = uv_rect;
                        let uv_arr = [u, v, uw, vh];

                        // Текст работает по принципу использования прямоугольников
                        // для глифов. В рендеринге нет ни одного объекта кроме
                        // прямоугольника, это некая фича которая позволяет оптимизировать
                        // это всё. Просто используем uv координаты и атлас в качестве
                        // текстуры
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
        
        // Это сортировка по z идексу если что
        perf_start!("[BATCH]: Sort");
            self.batch.sort();
        perf_end!("[BATCH]: Sort");

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
                vbo.update(context, &self.batch.cpu_buffer);
            } else {
                // [HACK] [UNWRAP]
                // Удалить unwrap
                self.instance_vbo = Some(BackendBuffer::vertex(context, &self.batch.cpu_buffer).unwrap());
            }
        }

        self.batch.upload(context);
    }

    pub fn render<'a>(
        &'a self,
        pass: &mut RenderPass<'a>,
        white_texture: &'a BackendTexture,
        textures: &'a std::collections::HashMap<u32, BackendTexture>,
        atlas_bind_group: Option<&'a RawBindGroup>,
    ) -> Result<(), MoonWalkError> {
        perf_start!("[BATCH]: Render");

        // Проверка есть ли данные для рендера
        if self.instance_vbo.is_none() || self.commands.is_empty() {
            return Ok(());
        }

        pass.set_vertex_buffer(0, &self.static_vbo);
        pass.set_vertex_buffer(1, self.instance_vbo.as_ref().unwrap());
        pass.set_index_buffer(&self.static_ibo);

        // [HACK]
        // HACK: White_texture это текстура размером 1 на 1 пиксель которая создаётся
        // в state.rs и этот 1 пиксель полностью белого цвета. Это нужно для того
        // что сэкономить время рендеринга из-за чего приходится жертвовать чистотой
        // кода
        
        // expect тут полностью оправдан, так как white_texture 100% существует,
        // в state.rs если бы при инициализации white_texture были бы проблемы
        // то ? вернул бы Err и всё упало ещё до первого вызова рендера батча
        let white_bg = &white_texture.get_raw().expect("White texture not inited").bind_group;

        for cmd in &self.commands {
            let bind_group = if cmd.texture_id == 0 {
                white_bg
            } else if cmd.texture_id == crate::textware::ATLAS_ID {
                atlas_bind_group.unwrap_or(white_bg)
            } else {
                match textures.get(&cmd.texture_id).and_then(|t| t.get_raw()) {
                    Some(raw) => &raw.bind_group,
                    None => white_bg,
                }
            };

            pass.set_bind_group(1, bind_group);
            pass.draw_indexed_instanced_extended(
                6,
                cmd.count,
                0,
                0,
                cmd.start_index
            );
        }

        perf_end!("[BATCH]: Render");

        Ok(())
    }

    /// Рисует текстуру на весь экран
    pub fn blit<'a>(
        &'a mut self, 
        context: &mut BackendContext, 
        pass: &mut RenderPass<'a>, 
        texture: &'a BackendTexture, 
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

        self.blit_vbo.update(context, &[instance]);

        // Отрисовка буфера
        pass.set_vertex_buffer(0, &self.static_vbo);
        pass.set_vertex_buffer(1, &self.blit_vbo);
        pass.set_index_buffer(&self.static_ibo);

        match texture.get_raw() {
            Some(raw) => {
                pass.set_bind_group(1, &raw.bind_group);
            },

            None => {
                eprintln!("Texture not init!");
            }
        };
        
        pass.draw_indexed_instanced_extended(
            6,
            1,
            0,
            0,
            0,
        );
    }
}