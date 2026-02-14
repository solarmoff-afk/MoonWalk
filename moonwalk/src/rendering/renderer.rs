// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use glam::{Vec2, Vec4};

use moonwalk_backend::core::context::{BackendContext, BackendPresentMode};
use moonwalk_backend::core::encoder::BackendEncoder;
use moonwalk_backend::render::pass::RenderPass;
use moonwalk_backend::render::texture::BackendTexture;
use moonwalk_backend::core::context::SurfaceRenderer;

use crate::error::MoonWalkError;

use crate::rendering::snapshot::ClippedSnapshot;
use crate::rendering::state::RenderState;
use crate::objects::ObjectId;
use crate::filters::FilterSystem;
use crate::path::VectorSystem;
use crate::debug_println;
use crate::painting::PaintingSystem;

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

/// Структура рендерера. Она хранит контекст moonwalk_backend
/// и состояние рендера (матричный стэк, храниоище объектов и так далее)
pub struct MoonRenderer {
    pub context: BackendContext,
    surface_renderer: SurfaceRenderer,

    pub state: RenderState,
    pub scale_factor: f32,
    pub filters: FilterSystem,
    pub text_engine: crate::textware::TextWare,
    pub vector_system: VectorSystem,
    pub painting_system: PaintingSystem,

    // [WAIT DOC]
    snapshot_tasks: Vec<SnapshotTask>,

    offscreen: Option<BackendTexture>,
}

impl MoonRenderer {
    /// В конструкуторе получаем окно и ширину/высоту. Конструктор
    /// в идеале вызывается только 1 раз при инициализации MoonWalk
    /// из публичного API
    pub fn new(
        window: &(impl HasWindowHandle + HasDisplayHandle),
        width: u32, height: u32
    ) -> Result<Self, MoonWalkError> {
        // Создание контекст рендеринга
        let mut context = BackendContext::new();
        context.create_context_sync(window, width, height);

        let filters = FilterSystem::new(&mut context)?;
        
        // Система векторного рисования
        let vector_system = VectorSystem::new(&mut context)?;

        // Система растрового рисования
        let painting_system = PaintingSystem::new(&mut context)?;
        
        // Создаём состояние рендерера
        let state = RenderState::new(&mut context, width, height)?;

        let text_engine = crate::textware::TextWare::new(&mut context)?;

        Ok(Self {
            context, // Контекст gpu/wgpu
            surface_renderer: SurfaceRenderer::new(),

            state,   // Состояние рендерера
            scale_factor: 1.0,
            filters,
            text_engine,
            vector_system,
            painting_system,

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

        // [HACK] [UNWRAP]
        let size = self.context.get_size().expect("Context not found");
        
        let width = size.x;
        let height = size.y;
        
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

            self.state.update_projection(&mut self.context, logical_w, logical_h);
        }
    }

    /// Регистрирует пустую текстуру, возвращает её, добавляет в очередь 
    /// и запекает (Снапшотит/скриншотит) туда экран когда приходит время
    pub fn request_snapshot(&mut self, x: u32, y: u32, w: u32, h: u32) -> u32 {
        let format = self.context.get_format();
        
        let mut texture = BackendTexture::new(w, h);
        texture.create_render_target(&mut self.context, w, h);

        // Регистрируем текстуру в состоянии чтобы добавить в очередь на снапшот
        // и потом вернуть
        let id = self.state.add_texture(texture);

        // Запекание будет в конце кадра в функции render
        let mut snapshot_region = ClippedSnapshot::new(
            Vec2::new(x as f32, y as f32),
            Vec2::new(w as f32, h as f32)
        );

        // [HACK] [UNWRAP]
        let size = self.context.get_size().expect("Context not found");
        
        snapshot_region.clip_snapshot(Vec2::new(
            size.x as f32,
            size.y as f32,
        ));

        self.snapshot_tasks.push(
            SnapshotTask {
                target_id: id,
                x: snapshot_region.position.x as u32,
                y: snapshot_region.position.y as u32,
                w: snapshot_region.size.x as u32,
                h: snapshot_region.size.y as u32,
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
    pub fn render(&mut self, clear_color: Vec4) -> Result<(), MoonWalkError> {
        let size = self.context.get_size()?;
        let width = size.x;
        let height = size.y;
        
        let format = self.context.get_format();

        let need_recreate = self.offscreen.as_ref()
            .map_or(true, |tex| tex.width != width || tex.height != height);

        if need_recreate {
            let mut texture = BackendTexture::new(width, height);
            texture.config.set_format(format);
            texture.create_render_target(&mut self.context, width, height)?;

            self.offscreen = Some(texture);
        }

        let offscreen_tex = self.offscreen.as_ref().unwrap();

        let mut encoder = BackendEncoder::new(&mut self.context, "Render encoder")?;

        self.text_engine.prepare(&mut self.context);
        let atlas_bg = self.text_engine.get_bind_group();

        // Здесь рисуется текущее состояние в буфер кадра
        self.state.draw(&mut self.context, &mut encoder, &offscreen_tex, &mut self.text_engine, Some(&atlas_bg?), clear_color);
        
        if !self.snapshot_tasks.is_empty() {
            for task in &self.snapshot_tasks {
                if let Some(target_tex) = self.state.textures.get(&task.target_id) {
                    // [HACK] [UNWRAP]
                    encoder.copy_texture_to_texture(task.x, task.y, task.w, task.h, 
                        offscreen_tex.get_raw().unwrap(), target_tex.get_raw().unwrap()
                    )?;
                }
            }

            // Очищаем очередь задач после выполнения
            self.snapshot_tasks.clear();
        }

        let frame = self.surface_renderer.begin(&mut self.context)?;

        let mut blit_encoder = BackendEncoder::new(&mut self.context, "Blit encoder")?;
        {
            let mut pass = RenderPass::new(
                &mut blit_encoder,
                &frame,
                Some(clear_color),
                "Blit render pass",
            )?;
            
            if let Some(pipeline) = self.state.shaders.get_pipeline(self.state.rect_shader) {
                pass.set_pipeline(pipeline);
                pass.set_bind_group(0, &self.state.proj_bind_group);
                
                self.state.batches.objects.blit(
                    &mut self.context,
                    &mut pass,
                    &offscreen_tex,
                    (width as f32 / self.scale_factor) as u32,
                    (height as f32 / self.scale_factor) as u32
                );
            }
        }

        // Отправляем всё на рендер через контекст рендеринга
        encoder.submit_frame(&mut self.context)?;
        blit_encoder.submit_frame(&mut self.context)?;

        self.surface_renderer.end();

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

    /// Этот метод позволяет включить или выключить вертикальную
    /// синхронизацию
    pub fn set_vsync(&mut self, enable: bool) {
        debug_println!("Vsync: {}", enable);

        let mode = if enable {
            BackendPresentMode::Fifo
        } else {
            BackendPresentMode::AutoNoVsync
        };
        
        self.context.set_present_mode(mode);
    }

    pub fn apply_blur(&mut self, texture_id: u32, radius: f32, horizontal: bool) {
        if let Some(texture) = self.state.textures.get_mut(&texture_id) {
            debug_println!("Blur apply, texture found in state");
            self.filters.apply_blur(&mut self.context, texture, radius, horizontal);
        }
    }

    pub fn apply_color_matrix(&mut self, texture_id: u32, matrix: [[f32; 4]; 4], offset: [f32; 4]) {
        if let Some(texture) = self.state.textures.get_mut(&texture_id) {
            debug_println!("Color matrix apply, texture found in state");
            self.filters.apply_color_matrix(&mut self.context, texture, matrix, offset);
        }
    }
    
    pub fn apply_chromakey(&mut self, texture_id: u32, key_color: [f32; 3], tolerance: f32) {
        if let Some(texture) = self.state.textures.get_mut(&texture_id) {
            debug_println!("Chromakey apply, texture found in state");
            self.filters.apply_chromakey(&mut self.context, texture, key_color, tolerance);
        }
    }

    pub fn apply_stencil(&mut self, target_id: u32, mask_id: u32, invert: bool) {
        if let Some(target) = self.state.textures.get(&target_id) {
            if let Some(mask) = self.state.textures.get(&mask_id) {
                debug_println!("Stencil apply, textures found in state");
                self.filters.apply_stencil(&mut self.context, target, mask, invert);
            }
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
    pub fn register_texture(&mut self, texture: BackendTexture) -> u32 {
        self.state.add_texture(texture)
    }

    #[inline]
    pub fn remove_texture(&mut self, texture_id: u32) {
        self.state.remove_texture(texture_id)
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
