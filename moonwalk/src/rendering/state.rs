// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use bytemuck::{Pod, Zeroable};

use crate::easy_gpu::{Context, Buffer, MatrixStack, RenderPass};
use crate::batching::group::BatchGroup;
use crate::rendering::pipeline::ShaderStore;
use crate::objects::store::ObjectStore;
use crate::objects::ShaderId;
use crate::error::MoonWalkError;

/// Структура для единой юниформы под все шейдеры. Не передаём
/// матрицу модели для экономии передачи данных через шину.
///
/// [?] view_proj - Матрица вида и проекции
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
}

impl RenderState {
    pub fn new(
        ctx: &Context,
        width: u32,
        height: u32
    ) -> Result<Self, MoonWalkError> {
        // Создаём хранилище для шейдеров. Каждый шейдер это отдельный
        // конвейер для рендеринга.
        let mut shaders = ShaderStore::new(ctx);

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
        let proj_bind_group = shaders.get_proj_bind_group(ctx, &uniform_buffer.raw);

        Ok(Self {
            store: ObjectStore::new(),
            batches: BatchGroup::new(ctx),
            shaders,
            matrix_stack,
            uniform_buffer,
            proj_bind_group,
            rect_shader,
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
    }

    /// Функция для рисования всех объектов
    pub fn draw(&mut self, ctx: &Context, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        // Подготавливаем батчи
        self.batches.rects.prepare(ctx, &self.store);
        
        // Если объекты грязные (dirty) - снимаем флаг 
        // (так как изменения уже отрисованы)
        if self.store.dirty {
            self.store.dirty = false;
        }

        // Для Z индексом тоже снимаем флаг если он стоит
        // так как в prepare мы уже всё отсортировали 
        if self.store.z_dirty {
            self.store.z_dirty = false;
        }

        // Создаём проход рендера
        let mut pass = RenderPass::new(
            encoder,
            target,
            
            // Цвет заливки
            Some(wgpu::Color {
                r: 0.1,
                g: 0.1,
                b: 0.1,
                a: 1.0
            })
        );

        pass.set_bind_group(0, &self.proj_bind_group);

        // Проверяем конвейер рендера (Хардкод для прямоугольников)
        if let Some(pipeline) = self.shaders.get_pipeline(self.rect_shader) {
            // Устаналиваем пайплайн
            pass.set_pipeline(pipeline);
            
            // Отрисовываем прямоугольники
            self.batches.rects.render(&mut pass);
        }
    }
}