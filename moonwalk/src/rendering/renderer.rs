// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use moonwalk_backend::pipeline::types::BlendMode;
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
use crate::filters::FilterSystem;
use crate::draw::path::VectorSystem;
use crate::{MoonSurface, debug_println};
use crate::draw::painting::PaintingSystem;

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
    pub text_engine: crate::draw::text::TextWare,
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
        let mut state = RenderState::new(&mut context, width, height)?;

        let text_engine = crate::draw::text::TextWare::new(&mut state, &mut context)?;

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
        let mut texture = BackendTexture::new(w, h);
        texture.config.set_format(self.context.get_format());
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
    pub fn render(
        &mut self,
        clear_color: Vec4,
        surface: &mut MoonSurface,
        blend_mode: BlendMode,
    ) -> Result<(), MoonWalkError> {
        // Валидация цвета заливки
        surface.fuse.validate_color(clear_color);

        // [TODO] [MAYBE] [HACK] [FIXME]
        // Сейчас тут используется заглушка для текущего количества шрифтов так
        // как текстовая система скоро должна быть переписана, поэтому пока так
        // чтобы не плодить код который скоро сдохнет
        surface.fuse.validate_resource_count(self.state.textures.len(), 1);

        // Валидация количества объектов
        surface.pre_render();

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

        // Здесь рисуется текущее состояние в буфер кадра
        self.state.draw(
            &mut self.context,
            &mut encoder, 
            &offscreen_tex,
            &mut self.text_engine, 
            // Some(atlas_bg?),
            clear_color,
            surface,
            blend_mode,
        );
        
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
            self.filters.apply_blur(&mut self.context, texture, radius, horizontal);
        }
    }

    pub fn apply_liquid_glass(
        &mut self,
        texture_id: u32,
        output_texture_id: u32,
        size: Vec2,
        offset: Vec2,
        corner_radius: Vec4,
        refraction_height: f32,
        refraction_amount: f32,
        depth_effect: f32,
        chromatic_aberration: f32,
        rotation: f32,
        gamma: f32,
        color: Vec4, // TODO
    ) {
        if let Some(target) = self.state.textures.get(&texture_id) {
            if let Some(output) = self.state.textures.get(&output_texture_id) {
                self.filters.apply_liquid_glass(
                    &mut self.context,
                    target,
                    output,
                    [size.x, size.y],
                    [offset.x, offset.y],
                    [corner_radius.x, corner_radius.y, corner_radius.z, corner_radius.w],
                    refraction_height,
                    refraction_amount,
                    depth_effect,
                    chromatic_aberration,
                    rotation, 
                    gamma,
                    [color.x, color.y, color.z, color.w],
                );
            }
        }
    }

    pub fn apply_liquid_glass_mask(
        &mut self,
        texture_id: u32,
        mask_texture_id: u32,
        output_texture_id: u32,
        size: Vec2,
        offset: Vec2, 
        refraction_amount: f32,
        refraction_height: f32, 
        depth_effect: f32,
        chromatic_aberration: f32,
        tolerance: f32,
        gamma: f32,
    ) {
        if let Some(target) = self.state.textures.get(&texture_id) {
            if let Some(mask) = self.state.textures.get(&mask_texture_id) {
                if let Some(output) = self.state.textures.get(&output_texture_id) {
                    self.filters.apply_liquid_glass_mask(
                        &mut self.context,
                        target,
                        mask,
                        output,
                        [size.x, size.y],
                        [offset.x, offset.y],
                        refraction_height,
                        refraction_amount,
                        depth_effect,
                        chromatic_aberration,
                        tolerance,
                        gamma,
                    );
                }
            }
        }
    }

    pub fn apply_mesh_gradient(
        &mut self,
        texture_id: u32,
        colors: [Vec4; 9],
        pos: [Vec4; 9],
        noise_intensity: f32,
        warp_strength: f32,
        warp_phase: f32,
        gamma: f32,
        normal_blend_mode: bool,
    ) {
        if let Some(target) = self.state.textures.get(&texture_id) {
            self.filters.apply_mesh_gradient(
                &mut self.context,
                target,
                colors,
                pos,
                noise_intensity,
                warp_strength,
                warp_phase,
                gamma,
                normal_blend_mode,
            );
        }
    }

    pub fn apply_sdf_mask(&mut self, texture_id: u32, radius: f32, hardness: f32) {
        if let Some(texture) = self.state.textures.get_mut(&texture_id) {
            self.filters.apply_sdf_mask(&mut self.context, texture, radius, hardness);
        }
    }

    pub fn apply_color_matrix(&mut self, texture_id: u32, matrix: [[f32; 4]; 4], offset: [f32; 4]) {
        if let Some(texture) = self.state.textures.get_mut(&texture_id) {
            self.filters.apply_color_matrix(&mut self.context, texture, matrix, offset);
        }
    }
    
    pub fn apply_chromakey(&mut self, texture_id: u32, key_color: [f32; 3], tolerance: f32) {
        if let Some(texture) = self.state.textures.get_mut(&texture_id) {
            self.filters.apply_chromakey(&mut self.context, texture, key_color, tolerance);
        }
    }

    pub fn apply_stencil(&mut self, target_id: u32, mask_id: u32, invert: bool) {
        if let Some(target) = self.state.textures.get(&target_id) {
            if let Some(mask) = self.state.textures.get(&mask_id) {
                self.filters.apply_stencil(&mut self.context, target, mask, invert);
            }
        }
    }

    #[inline]
    pub fn register_texture(&mut self, texture: BackendTexture) -> u32 {
        self.state.add_texture(texture)
    }

    #[inline]
    pub fn remove_texture(&mut self, texture_id: u32) {
        self.state.remove_texture(texture_id)
    }
}
