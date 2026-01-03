// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use glam::{Vec2, Vec4};

use crate::gpu::Context;
use crate::error::MoonWalkError;
use crate::rendering::texture::Texture;
use crate::rendering::state::RenderState;
use crate::objects::ObjectId;
use crate::filters::FilterSystem;
use crate::path::VectorSystem;

/// Wgpu работает асинхронно поэтому нам нужно при вызове публичного api для
/// снапшота вернуть какой-то айди, добавить его в очередь (Как раз этой структуры)
/// и превратить в текстуру когда это возможно (В функции рендера)
struct SnapshotTask {
    target_id: u32,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

/// Структура рендерера. Она хранит контекст (gpu -> wgpu)
/// и состояние рендера (матричный стэк, храниоище объектов и так далее)
pub struct MoonRenderer {
    pub context: Context,
    pub state: RenderState,
    pub scale_factor: f32,
    pub filters: FilterSystem,
    pub text_engine: crate::textware::TextWare,
    pub vector_system: VectorSystem,

    // [WAIT DOC]
    snapshot_tasks: Vec<SnapshotTask>,

    offscreen: Option<crate::rendering::texture::Texture>,
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

        let filters = FilterSystem::new(&context)?;
        let vector_system = VectorSystem::new(&context)?;
        
        // Создаём состояние рендерера
        let state = RenderState::new(&context, width, height)?;

        let text_engine = crate::textware::TextWare::new(&context.device, &context.queue);

        Ok(Self {
            context, // Контекст gpu/wgpu
            state,   // Состояние рендерера
            scale_factor: 1.0,
            filters,
            text_engine,
            vector_system,

            // Обычно снапшотов очень мало, цифра 8 взята на всякий случай,
            // но тут хватило бы и 4
            snapshot_tasks: Vec::with_capacity(8),
            offscreen: None,
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

    /// Регистрирует пустую текстуру, возвращает её, добавляет в очередь 
    /// и запекает (Снапшотит/скриншотит) туда экран когда приходит время
    pub fn request_snapshot(&mut self, x: u32, y: u32, w: u32, h: u32) -> u32 {
        let format = self.context.config.format;
        let texture = crate::rendering::texture::Texture::create_empty(
            &self.context, w, h, format, "Snapshot Target"
        );

        // Регистрируем текстуру в состоянии чтобы добавить в очередь на снапшот
        // и потом вернуть 
        let id = self.state.add_texture(texture);

        // Запекание будет в конце кадра в функции render
        self.snapshot_tasks.push(
            SnapshotTask {
                target_id: id,
                x, y, w, h
            }
        );

        id
    }

    /// Эта функция берёт айди существующей текстуры и использует её как таргет
    /// для снапшота который ставит в очередь
    pub fn update_snapshot(&mut self, x: u32, y: u32, w: u32, h: u32, id: u32) {
        // Запекание будет в конце кадра в функции render
        self.snapshot_tasks.push(
            SnapshotTask {
                target_id: id,
                x, y, w, h
            }
        );
    }

    /// Функция для отправки всего на рендер
    pub fn render(&mut self) -> Result<(), MoonWalkError> {
        let width = self.context.config.width;
        let height = self.context.config.height;
        let format = self.context.config.format;

        let need_recreate = self.offscreen.as_ref()
            .map_or(true, |tex| tex.texture.width() != width || tex.texture.height() != height);

        if need_recreate {
            self.offscreen = Some(crate::rendering::texture::Texture::create_empty(
                &self.context,
                width,
                height,
                format,
                "Offscreen Target",
            ));
        }

        let offscreen_tex = self.offscreen.as_ref().unwrap();
        let render_target_view = &offscreen_tex.view; 

        let mut encoder = self.context.create_encoder();

        self.text_engine.prepare(&self.context.queue);
        let atlas_bg = self.text_engine.get_bind_group();

        // Здесь рисуется текущее состояние в буфер кадра
        self.state.draw(&self.context, &mut encoder, render_target_view, &mut self.text_engine, Some(&atlas_bg));
        
        if !self.snapshot_tasks.is_empty() {
            for task in &self.snapshot_tasks {
                if let Some(target_tex) = self.state.textures.get(&task.target_id) {
                    encoder.copy_texture_to_texture(
                        wgpu::TexelCopyTextureInfo {
                            texture: &offscreen_tex.texture,
                            mip_level: 0,
                            origin: wgpu::Origin3d {
                                x: task.x,
                                y: task.y,
                                z: 0
                            },
                            aspect: wgpu::TextureAspect::All,
                        },

                        wgpu::TexelCopyTextureInfo {
                            texture: &target_tex.texture,
                            mip_level: 0,
                            origin: wgpu::Origin3d::ZERO,
                            aspect: wgpu::TextureAspect::All,
                        },

                        wgpu::Extent3d {
                            width: task.w,
                            height: task.h,
                            depth_or_array_layers: 1,
                        }
                    );
                }
            }

            // Очищаем очередь задач после выполнения
            self.snapshot_tasks.clear();
        }

        let frame = self.context.surface.as_ref().unwrap().get_current_texture()
            .map_err(|e| MoonWalkError::SurfaceError(e))?;

        let surface_view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut blit_encoder = self.context.create_encoder();
        {
            let mut pass = crate::gpu::RenderPass::new(
                &mut blit_encoder,
                &surface_view,
                None
            );
            
            if let Some(pipeline) = self.state.shaders.get_pipeline(self.state.rect_shader) {
                pass.set_pipeline(pipeline);
                pass.set_bind_group(0, &self.state.proj_bind_group);
                
                self.state.batches.objects.blit(
                    &self.context,
                    &mut pass,
                    &offscreen_tex,
                    width, 
                    height
                );
            }
        }

        // Отправляем всё на рендер через контекст рендеринга
        self.context.queue.submit([encoder.finish(), blit_encoder.finish()]);

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

    pub fn apply_blur(&mut self, texture_id: u32, radius: f32, horizontal: bool) {
        if let Some(texture) = self.state.textures.get(&texture_id) {
            self.filters.apply_blur(&self.context, texture, radius, horizontal);
        }
    }

    pub fn apply_color_matrix(&mut self, texture_id: u32, matrix: [[f32; 4]; 4], offset: [f32; 4]) {
        if let Some(texture) = self.state.textures.get(&texture_id) {
            self.filters.apply_color_matrix(&self.context, texture, matrix, offset);
        }
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
    pub fn config_color2(&mut self, id: ObjectId, color2: Vec4) {
        self.state.store.config_color2(id, color2);
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
    pub fn set_effect(&mut self, id: ObjectId, effect_data: [f32; 2]) {
        self.state.store.config_effect_data(id, effect_data);
    }

    #[inline]
    pub fn register_texture(&mut self, texture: Texture) -> u32 {
        self.state.add_texture(texture)
    }

    #[inline]
    pub fn config_gradient_data(&mut self, id: ObjectId, gradient_data: [f32; 4]) {
        self.state.store.config_gradient_data(id, gradient_data);
    }

    // Специфично для прямоугольника
    #[inline]
    pub fn set_rounded(&mut self, id: ObjectId, radii: Vec4) {
        self.state.store.set_rounded(id, radii);
    }
}
