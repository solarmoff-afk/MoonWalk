// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use bytemuck::{Pod, Zeroable};
use std::collections::HashMap;
use glam::Vec4;

use crate::gpu::{Context, Buffer, MatrixStack, RenderPass};
use crate::batching::group::BatchGroup;
use crate::rendering::pipeline::ShaderStore;
use crate::rendering::texture::Texture;
use crate::objects::store::ObjectStore;
use crate::objects::ShaderId;
use crate::error::MoonWalkError;
use crate::textware::TextWare;

/// Структура для единой юниформы под все шейдеры. Не передаём
/// матрицу модели для экономии передачи данных через шину.
///
/// - [?] view_proj - Матрица вида и проекции
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GlobalUniform {
    pub view_proj: [[f32; 4]; 4],
}

pub struct RenderState {
    pub store: ObjectStore, // Хранилище объектов
    pub batches: BatchGroup, // Группа батчинга
    pub shaders: ShaderStore, // Хранилище шейдеров
    pub matrix_stack: MatrixStack, // Матричный стэк
    pub uniform_buffer: Buffer<GlobalUniform>, // Буфер дла передачи данных в шейдер
    pub proj_bind_group: wgpu::BindGroup,
    pub rect_shader: ShaderId, // Пайплайн для прямоугольника
    pub white_texture: Texture,
    pub textures: HashMap<u32, Texture>,
    next_texture_id: u32,
}

impl RenderState {
    pub fn new(
        ctx: &Context,
        width: u32,
        height: u32
    ) -> Result<Self, MoonWalkError> {
        // Создаём хранилище для шейдеров. Каждый шейдер это отдельный
        // конвейер для рендеринга.
        let mut shaders = ShaderStore::new(ctx)?;

        // Создаём шейдер для прямоугольника.
        let rect_shader = shaders.create_default_rect(ctx, ctx.config.format)?;
        
        // Создаём матричный стэк
        let mut matrix_stack = MatrixStack::new();
        
        // Задаём ортографическую проекцию на основе ширины
        // и высоты окна
        matrix_stack.set_ortho(width as f32, height as f32);
        
        // Создаём глобальные данные для передачи в шейдеры
        let uniform_data = GlobalUniform {
            view_proj: matrix_stack.projection.to_cols_array_2d(),
        };
        
        // Создаём буфер для шейдерных данных (Юниформ)
        let uniform_buffer = Buffer::uniform(ctx, &uniform_data);
        
        // Обновляем проекцию в ShaderStore и получаем bind group
        shaders.update_projection(ctx, &uniform_buffer.raw);
        
        // Получаем bind group из ShaderStore
        let proj_bind_group = shaders.get_proj_bind_group()
            .expect("Projection bind group not initialized")
            .clone();

        // [HACK]
        // движок использует 1 пайплайн (1 шейдер) для объектов и с текстурой и без
        // и для этого в шейдере [shaders/shape.wgsl] передаётся текстура, поэтому она нужна
        // даже когда объект просто цветной (без текстуры). Я решил сделать текстуру 1 на 1
        // пиксель с белым цветом (ВАЖНО! Чтобы цвет объекта не изменился)
        let white_pixels = vec![255, 255, 255, 255];
        let white_texture = Texture::from_raw(ctx, &white_pixels, 1, 1, "White Default")?;
        
        Ok(Self {
            store: ObjectStore::new(),
            batches: BatchGroup::new(ctx),
            shaders,
            matrix_stack,
            uniform_buffer,
            proj_bind_group,
            rect_shader,
            white_texture,
            textures: HashMap::new(),
            next_texture_id: 1, // 0 занят под white_texture
        })
    }

    /// Функция для обновления матрицы проекции. Вызывается при изменении размера
    /// окна через вьюпорт функцию из renderer (А она вызывается из публичного API)
    pub fn update_projection(&mut self, ctx: &Context, width: f32, height: f32) {
        self.matrix_stack.set_ortho(width, height);

        let uniform_data = GlobalUniform {
            view_proj: self.matrix_stack.projection.to_cols_array_2d(),
        };
        
        self.uniform_buffer.update_one(ctx, &uniform_data);
        self.shaders.update_projection(ctx, &self.uniform_buffer.raw);

        self.proj_bind_group = self.shaders.get_proj_bind_group()
            .expect("Projection bind group not initialized")
            .clone();
    }

    /// Функция для рисования всех объектов
    pub fn draw(&mut self, ctx: &Context, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView, text_engine: &mut TextWare, atlas_bg: Option<&wgpu::BindGroup>, clear_color: Vec4) {
        // Подготавливаем батчи
        self.batches.objects.prepare(ctx, &self.store, text_engine);
        
        // Если объекты грязные (dirty) - снимаем флаг 
        // (так как изменения уже отрисованы)
        if self.store.dirty {
            self.store.dirty = false;
        }

        // Создаём проход рендера
        let mut pass = RenderPass::new(
            encoder,
            target,
            
            // Цвет заливки
            Some(wgpu::Color {
                r: clear_color.x as f64,
                g: clear_color.y as f64,
                b: clear_color.z as f64,
                a: clear_color.w as f64
            })
        );

        pass.set_bind_group(0, &self.proj_bind_group);

        // Проверяем конвейер рендера (Хардкод для прямоугольников)
        if let Some(pipeline) = self.shaders.get_pipeline(self.rect_shader) {
            // Устаналиваем пайплайн
            pass.set_pipeline(pipeline);
            
            // Отрисовываем прямоугольники
            self.batches.objects.render(&mut pass, &self.white_texture, &self.textures, atlas_bg);
        }
    }

    /// Загрузка текстуры в хэш карту (Передаются байты)
    pub fn load_texture(&mut self, ctx: &Context, bytes: &[u8], label: &str) -> Result<u32, MoonWalkError> {
        let texture = Texture::from_bytes(ctx, bytes, label)?;
        let id = self.next_texture_id;
        
        self.textures.insert(id, texture);
        self.next_texture_id += 1;
        
        Ok(id)
    }

    /// Этот метод добавляет текстуру в HashMap в состоянии
    pub fn add_texture(&mut self, texture: Texture) -> u32 {
        let id = self.next_texture_id;
        
        self.textures.insert(id, texture);
        self.next_texture_id += 1;
        
        id
    }

    /// Удаляет текстуру из HashMap. При удалении ключа текстура автоматически
    /// выходит из области видимости и wgpu очищает VRAM
    pub fn remove_texture(&mut self, texture_id: u32) {
        if self.textures.get(&texture_id).is_some() {
            self.textures.remove(&texture_id);
        }
    }

    /// [WAIT DOC]
    pub fn blit<'a>(&'a mut self, ctx: &Context, pass: &mut RenderPass<'a>, texture: &'a Texture) {
        if let Some(pipeline) = self.shaders.get_pipeline(self.rect_shader) {
            pass.set_pipeline(pipeline);
            
            pass.set_bind_group(0, &self.proj_bind_group);
            
            self.batches.objects.blit(
                ctx, 
                pass, 
                texture, 
                ctx.config.width, 
                ctx.config.height
            );
        }
    }
}
