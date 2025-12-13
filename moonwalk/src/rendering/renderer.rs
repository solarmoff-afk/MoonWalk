// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use glam::{Vec2, Vec4};

use crate::easy_gpu::Context;
use crate::error::MoonWalkError;
use crate::rendering::texture::Texture;
use crate::rendering::state::RenderState;
use crate::objects::ObjectId;

/// Структура рендерера. Она хранит контекст (easy_gpu -> wgpu)
/// и состояние рендера (матричный стэк, храниоище объектов и так далее)
pub struct MoonRenderer {
    pub context: Context,
    pub state: RenderState,
    pub scale_factor: f32,
}

impl MoonRenderer {
    /// В конструкуторе получаем окно и ширину/высоту. Конструктор
    /// в идеале вызывается только 1 раз при инициализации MoonWalk
    /// из публичного API
    pub fn new(
        window: &(impl HasWindowHandle + HasDisplayHandle),
        width: u32, height: u32
    ) -> Result<Self, MoonWalkError> {
        // Асинхронно создаём контекст рендеринга через pollster
        let context = pollster::block_on(
            Context::new(window, width, height)
        );
        
        // Создаём состояние рендерера
        let state = RenderState::new(&context, width, height)?;

        Ok(Self {
            context, // Контекст easy_gpu/wgpu
            state,   // Состояние рендерера
            scale_factor: 1.0,
        })
    }

    /// Обновляет DPI и пересчитывает проекцию
    pub fn set_scale_factor(&mut self, scale: f32) {
        self.scale_factor = scale;
        
        // Принудительно вызываем resize с текущими физическими размерами, 
        // чтобы пересчитать логическую матрицу
        let width = self.context.config.width;
        let height = self.context.config.height;
        
        self.resize(width, height);
    }

    /// Функция изменения размера холста для рисования,
    /// нужно передать только новую ширину и высоту
     pub fn resize(&mut self, width: u32, height: u32) {
        // Проверяем что ширина и высота НЕ НОЛЬ, иначе возможны
        // проблемы (Например, паника)
        if width > 0 && height > 0 {
            self.context.resize(width, height);
            
            let logical_w = width as f32 / self.scale_factor;
            let logical_h = height as f32 / self.scale_factor;

            self.state.update_projection(&self.context, logical_w, logical_h);
        }
    }

    /// Функция для отправки всего на рендер
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // Берём текущий кадр
        let frame = self.context.surface.as_ref().unwrap().get_current_texture()?;
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        // Создаём кодировщик
        let mut encoder = self.context.create_encoder();

        // Рисуем текущее состояние
        self.state.draw(&self.context, &mut encoder, &view);

        // Отправляем всё на рендер через контекст рендеринга
        self.context.submit(encoder);

        frame.present();
        Ok(())
    }

    /// На android после перезахода в приложение Surface (Хотс куда идёт рендер)
    /// удаляется (После выхода). Нам нужно пересоздавать его после повторного
    /// входа в приложение на android. Эта функция как раз пересоздаёт холст
    pub fn recreate_surface(
        &mut self,
        window: &(impl HasWindowHandle + HasDisplayHandle),
        width: u32, height: u32
    ) {
         self.context.recreate_surface(window, width, height);
    }

    /// Прокси методы

    #[inline]
    pub fn new_rect(&mut self) -> ObjectId {
        self.state.store.new_rect()
    }

    #[inline]
    pub fn config_position(&mut self, id: ObjectId, pos: Vec2) {
        self.state.store.config_position(id, pos);
    }

    #[inline]
    pub fn config_size(&mut self, id: ObjectId, size: Vec2) {
        self.state.store.config_size(id, size);
    }

    #[inline]
    pub fn config_color(&mut self, id: ObjectId, color: Vec4) {
        self.state.store.config_color(id, color);
    }

    #[inline]
    pub fn config_rotation(&mut self, id: ObjectId, radians: f32) {
        self.state.store.config_rotation(id, radians);
    }

    #[inline]
    pub fn set_z_index(&mut self, id: ObjectId, z: f32) {
        self.state.store.config_z_index(id, z);
    }

    #[inline]
    pub fn set_uv(&mut self, id: ObjectId, uv: [f32; 4]) {
        self.state.store.config_uv(id, uv);
    }

    #[inline]
    pub fn register_texture(&mut self, texture: Texture) -> u32 {
        self.state.add_texture(texture)
    }

    // Специфично для прямоугольника
    #[inline]
    pub fn set_rounded(&mut self, id: ObjectId, radii: Vec4) {
        self.state.store.set_rounded(id, radii);
    }
}