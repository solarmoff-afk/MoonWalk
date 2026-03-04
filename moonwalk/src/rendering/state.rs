// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use bytemuck::{Pod, Zeroable};

use moonwalk_backend::core::buffer::BackendBuffer;
use moonwalk_backend::core::context::BackendContext;
use moonwalk_backend::core::encoder::BackendEncoder;
use moonwalk_backend::pipeline::bind::RawBindGroup;
use moonwalk_backend::pipeline::types::BlendMode;
use moonwalk_backend::render::pass::RenderPass;
use moonwalk_backend::render::texture::BackendTexture;

use std::collections::HashMap;
use glam::Vec4;

use crate::core::matrix::MatrixStack;

use crate::rendering::batching::group::BatchGroup;
use crate::rendering::pipeline::ShaderStore;

use crate::core::objects::store::ObjectStore;
use crate::core::objects::ShaderId;
use crate::error::MoonWalkError;
use crate::draw::text::TextWare;
use crate::{perf_end, perf_start};
use crate::MoonSurface;

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
    pub uniform_buffer: BackendBuffer<GlobalUniform>, // Буфер дла передачи данных в шейдер
    pub proj_bind_group: RawBindGroup,
    
    // Пайплайны для разных бленд модов
    pub rect_shader: ShaderId, // Стандартный пайплайн, alpha
    pub rect_shader_additive: ShaderId, // Аддитивное смешивание, идеально для теней
    pub rect_shader_multiply: ShaderId, // Умножение, затемняет фон пропорционально яркости источника
    pub rect_shader_screen: ShaderId, // Делает изображение светлее в отличии от rect_shader_multiply
    pub rect_shader_subtract: ShaderId, // Вычитание, уменьшает яркость фона на значение источника
    
    pub white_texture: BackendTexture,
    pub textures: HashMap<u32, BackendTexture>,
    next_texture_id: u32, 
}

impl RenderState {
    pub fn new(
        context: &mut BackendContext,
        width: u32,
        height: u32
    ) -> Result<Self, MoonWalkError> {
        use moonwalk_backend::core::buffer::BackendBuffer;

        let format = context.get_format();
        
        // Создаём хранилище для шейдеров. Каждый шейдер это отдельный
        // конвейер для рендеринга. По факту в основном рендеринге
        // есть только 1 шейдер, shape.wgsl, он же и используется
        // для рендеринга текста, система досталась в наследство
        let mut shaders = ShaderStore::new(context)?;

        // Создаём шейдер для прямоугольника.
        let alpha_pipeline = shaders.create_default_rect(context, format, BlendMode::Alpha)?;
        let additive_pipeline = shaders.create_default_rect(context, format, BlendMode::Additive)?;
        let multiply_pipeline = shaders.create_default_rect(context, format, BlendMode::Multiply)?;
        let screen_pipeline = shaders.create_default_rect(context, format, BlendMode::Screen)?;
        let subtract_pipeline = shaders.create_default_rect(context, format, BlendMode::Subtract)?;

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
        let uniform_buffer = BackendBuffer::uniform(context, &uniform_data)?;
        
        // Обновляем проекцию в ShaderStore и получаем bind group
        shaders.update_projection(context, &uniform_buffer);
        
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

        let mut white_texture = BackendTexture::new(1, 1);
        white_texture.config.set_format(context.get_format());
        white_texture.from_raw(context, &white_pixels, 1, 1)?;
        
        Ok(Self {
            store: ObjectStore::new(),
            batches: BatchGroup::new(context),
            shaders,
            matrix_stack,
            uniform_buffer,
            proj_bind_group,
            
            rect_shader: alpha_pipeline,
            rect_shader_additive: additive_pipeline,
            rect_shader_multiply: multiply_pipeline,
            rect_shader_screen: screen_pipeline,
            rect_shader_subtract: subtract_pipeline,

            white_texture,
            textures: HashMap::new(),
            next_texture_id: 1, // 0 занят под white_texture 
        })
    }

    /// Функция для обновления матрицы проекции. Вызывается при изменении размера
    /// окна через вьюпорт функцию из renderer (А она вызывается из публичного API)
    pub fn update_projection(&mut self, context: &mut BackendContext, width: f32, height: f32) {
        self.matrix_stack.set_ortho(width, height);

        let uniform_data = GlobalUniform {
            view_proj: self.matrix_stack.projection.to_cols_array_2d(),
        };
        
        self.uniform_buffer.update_one(context, &uniform_data);
        self.shaders.update_projection(context, &self.uniform_buffer);

        self.proj_bind_group = self.shaders.get_proj_bind_group()
            .expect("Projection bind group not initialized")
            .clone();
    }

    /// Функция для рисования всех объектов
    pub fn draw(
        &mut self,
        context: &mut BackendContext,
        encoder: &mut BackendEncoder,
        target: &BackendTexture,
        text_engine: &mut TextWare,
        // atlas_bg: Option<&RawBindGroup>,
        clear_color: Vec4,
        surface: &MoonSurface,
        blend_mode: BlendMode,
    ) -> Result<(), MoonWalkError> {
        // Подготавливаем батчи
        use moonwalk_backend::render::pass::RenderPass;

        self.batches.objects.prepare(context, &surface.store, text_engine, None);
        
        // После того как батч готов нужно залить все глифы на gpu
        text_engine.prepare(context, self);

        // Чтобы не менять сигнатуры переменная должна быть Option, для этого
        // делается Some(...)
        let atlas_bg = Some(text_engine.get_bind_group()?);

        // Если объекты грязные (dirty) - снимаем флаг 
        // (так как изменения уже отрисованы)
        if self.store.dirty {
            self.store.dirty = false;
        }

        // Создаём проход рендера
        perf_start!("[STATE]: Create render pass");
            let mut pass = RenderPass::new(
                encoder, 
                target,
                Some(clear_color),
                "MoonWalk render pass"   
            )?;
        perf_end!("[STATE]: Create render pass");

        pass.set_bind_group(0, &self.proj_bind_group);

        // Проверяем конвейер рендера по режиму смешивания
        let pipeline = match blend_mode {
            BlendMode::Alpha => self.rect_shader,
            BlendMode::Additive => self.rect_shader_additive,
            BlendMode::Multiply => self.rect_shader_multiply,
            BlendMode::Screen => self.rect_shader_screen,
            BlendMode::Subtract => self.rect_shader_subtract,
            _ => self.rect_shader,
        };

        if let Some(pipeline) = self.shaders.get_pipeline(pipeline) {
            // Устаналиваем пайплайн
            pass.set_pipeline(pipeline);
            
            // Отрисовываем прямоугольники
            self.batches.objects.render(&mut pass, &self.white_texture, &self.textures, atlas_bg);
        }

        Ok(())
    } 

    /// Загрузка текстуры в хэш карту (Передаются байты)
    pub fn load_texture(&mut self, context: &mut BackendContext, bytes: &[u8], _label: &str) -> Result<u32, MoonWalkError> {
        // [HACK]
        // Тут создаётся текстура пиксель на пиксель так как from_bytes должен
        // создать текстуру нужного размера, но для его использлвания
        // нужен уже существующий экземлпляр
        let mut texture = BackendTexture::new(1, 1);
        let format = context.get_format();
        texture.config.set_format(format);
        texture.from_bytes(context, bytes);
        
        let id = self.next_texture_id;
        
        self.textures.insert(id, texture);
        self.next_texture_id += 1;
        
        Ok(id)
    } 

    pub fn add_texture(&mut self, texture: BackendTexture) -> u32 {
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
    pub fn blit<'a>(
        &'a mut self,
        context: &mut BackendContext,
        pass: &mut RenderPass<'a>,
        texture: &'a BackendTexture
    ) -> Result<(), MoonWalkError> {
        if let Some(pipeline) = self.shaders.get_pipeline(self.rect_shader) {
            pass.set_pipeline(pipeline);
            
            pass.set_bind_group(0, &self.proj_bind_group);

            let size = context.get_size()?;
            
            self.batches.objects.blit(
                context, 
                pass, 
                texture, 
                size.x,
                size.y
            );
        }

        Ok(())
    } 
}
