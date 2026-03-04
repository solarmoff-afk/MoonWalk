// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

pub mod objects;
pub mod getter;
pub mod effects;

use glam::{Vec2, Vec4};
use moonwalk_backend::core::context::BackendContext;
use moonwalk_backend::core::encoder::BackendEncoder;
use moonwalk_backend::core::buffer::BackendBuffer;
use moonwalk_backend::render::pass::RenderPass;
use moonwalk_backend::render::texture::BackendTexture;
use moonwalk_backend::pipeline::bind::{BindGroup, RawBindGroup};
use moonwalk_backend::pipeline::types::{BlendMode, ShaderStage};

use crate::core::matrix::MatrixStack;
use crate::rendering::state::{GlobalUniform, RenderState};
use crate::rendering::batching::shapes::uber::UberBatch;
use crate::rendering::snapshot::ClippedSnapshot;
use crate::core::objects::store::ObjectStore;
use crate::{MoonWalk, perf_end, perf_start};
use crate::ObjectId;
use crate::FontAsset;
use crate::draw::text::FontId;
use crate::error::MoonWalkError;
use crate::core::objects::{ShaderId, TextureId};
use crate::utils::fuse::MoonFuse;

struct RenderPassDescriptor {
    pub color: Vec4,
    
    // Для своего пайлпайна есть CustomPaint, вместо этого даём возможность
    // выбрать из 5 пайлпайнов для бленд мода. Это защищает от переусложения
    // основного api методами для своих пайлпайнов
    pub blend_mode: BlendMode,

    pub objects: Vec<ObjectId>,

    pub is_empty: bool,
}

/// Структура поверхности рендера, определяет общий api для главного рендера
/// и рендер контейнеров
pub struct MoonSurface {
    pub store: ObjectStore,
    pub batch: UberBatch,
    pub proj_bind_group: RawBindGroup,
    pub target: BackendTexture,
    pub width: u32,
    pub height: u32,
    pub blend_mode: BlendMode,

    // Предохранитель
    pub fuse: MoonFuse,

    render_passes: Vec<RenderPassDescriptor>, 
}

impl MoonSurface {
    pub fn new(context: &mut BackendContext, width: u32, height: u32) -> Result<Self, MoonWalkError> {
        let format = context.get_format(); 

        let mut target = BackendTexture::new(width, height);
        target.config.set_format(format);
        target.create_render_target(context, width, height)?;

        let mut matrix_stack = MatrixStack::new();
        matrix_stack.set_ortho(width as f32, height as f32);
        
        let uniform_data = GlobalUniform {
            view_proj: matrix_stack.projection.to_cols_array_2d(),
        };

        let uniform_buffer = BackendBuffer::uniform(
            context, &uniform_data
        )?;
        
        let proj_layout = BindGroup::new()
            .add_uniform(0, ShaderStage::Vertex)
            .build(context)?;

        let proj_bind_group = BindGroup::create_uniform_bind_group(
            &proj_layout, context, &uniform_buffer, Some("Container Proj Bind Group")
        )?;

        Ok(Self {
            store: ObjectStore::new(),
            batch: UberBatch::new(context)?,
            proj_bind_group,
            target,
            width,
            height,
            blend_mode: BlendMode::Alpha,
            fuse: MoonFuse::new(),

            render_passes: Vec::new(), 
        })
    }

    /// Функция для создания прямоугольника и получения его ID.
    /// Важное предупреждение: НЕ СОЗДАВАЙТЕ ОБЪЕКТЫ КАЖДЫЙ КАДР
    /// ЕСЛИ ЭТО НЕ ВАША ПРЯМАЯ ЦЕЛЬ. После создания объекта он
    /// существует в кэше рендер движка и просто отправляется на
    /// отрисовку в момент вызове render_frame, вам нужно только
    /// создать объект один раз, получить его ID (структура ObjectId)
    /// и работать с ним используя методы конфигурации
    pub fn new_rect(&mut self) -> ObjectId {
        self.store.new_rect()
    }

    /// Эта функция создаёт текст. Рендеринг текстов менее производительный чем
    /// рендеринг прямоугольников, но чаще всего текстов и меньше чем прямоугольников
    /// (В играх и UI). Это не должно быть критичным, но нужно учитывать.
    /// В будущем могут быть работы по дополнительной оптимизации
    pub fn new_text(&mut self, content: &str, font: FontAsset, size: f32) -> ObjectId {
        let internal_id = FontId(font.0 as usize);
        self.store.new_text(content.to_string(), internal_id, size)
    }

    /// Установить режим смешивания поверхности
    pub fn set_blend_mode(&mut self, blend_mode: BlendMode) {
        self.blend_mode = blend_mode;
    }

    /// Этот метод позволяет создать новый проход рендера в поверхности.
    /// Базовое использование: несколько режимов смешивания (blend mode)
    /// для объектов. Не позволяет создавать свой пайплайн, так как
    /// это очент опасная фишка для основного рендера на поверхностях.
    /// Для своих пайлпайнов можно использовать CustomPaint. Работает просто,
    /// вызывается push_render_pass чтобы добавить проход рендера. Обычно
    /// это делается на какой-то отдельной поверхности для небольшого
    /// количнства объектов. Передаётся режим смешивания, цвет заливки
    /// (можно Vec4::ZERO) и объекты. Это Option куда можно завернуть вектор
    /// (Vec) через Some.
    /// Если не передать вообще (None) то будут использованы все объекты
    /// в ObjectStore этой поверхности
    /// 
    /// Пример:
    /// ```rust.ignore
    /// // Поверхность
    /// let surface = mw.new_surface(1024, 1024);
    ///
    /// let rect1 = surface.new_rect();
    /// surface.set_size(Vec2::new(50.0, 50.0));
    /// surface.set_color(Vec4::new(1.0, 0.0, 0.0, 0.5));
    ///
    /// let rect2 = surface.new_rect();
    /// surface.set_size(Vec2::new(60.0, 60.0));
    /// surface.set_color(Vec4::new(0.0, 0.0, 1.0, 0.5));
    ///
    /// // Создаём первый проход, прозрачный фон, сюда только rect1
    /// surface.push_render_pass(BlendMode::Alpha, Vec4::ZERO, Some(vec![rect1]));
    ///
    /// // Второй проход рендера, тоже прозрачный фон, другой режим смешивания и rect2
    /// surface.push_render_pass(BlendMode::Additive, Vec4::ZERO, Some(vec![rect2]));
    /// ```
    pub fn push_render_pass(&mut self, blend_mode: BlendMode, color: Vec4, objects: Option<Vec<ObjectId>>) {
        let objects_vec = objects.unwrap_or(vec![]);
        let is_empty = objects_vec.is_empty();

        self.render_passes.push(RenderPassDescriptor {
            color,
            blend_mode,
            objects: objects_vec,
            is_empty,
        });
    }

    /// Очищает все созданные в поверхности проходы рендера
    pub fn clear_render_passes(&mut self) {
        self.render_passes.clear();
    }

    /// Этот метод вызывает валидацию перед рендером
    pub fn pre_render(&self) {
        let object_count = self.store.rect_ids.len();
        self.fuse.validate_objects_count(object_count);
    }

    /// Отрисовать все объекты на surface
    pub fn render(&mut self, mw: &mut MoonWalk, clear_color: Option<Vec4>) -> Result<(), MoonWalkError> {
        // Перед началом нужно сделать валидации (цвета заливки и количества объектов) 
        // через предохранитель
        match clear_color {
            Some(color) => self.fuse.validate_color(color),
            None => {}
        };

        // Дополнительная валдиация перед рендером
        self.pre_render();
        
        let renderer = &mut mw.renderer;
        let context = &mut renderer.context;
        let text_engine = &mut renderer.text_engine;
        
        self.batch.prepare(context, &self.store, text_engine, None);

        // Заливаем глифы только после подготовки батчинга
        text_engine.prepare(context, &renderer.state); 
        let atlas_bg = text_engine.get_bind_group()?;
        
        let clear_color = clear_color.map(|c|
            Vec4::new(c.x, c.y, c.z, c.w));

        let mut encoder = BackendEncoder::new(context, "MoonSurface encoder")?;

        let mut pass = RenderPass::new(
            &mut encoder,
            &self.target,
            clear_color,
            "MoonSurface render pass"
        )?;

        let pipeline_id = Self::map_blend_mode(&renderer.state, self.blend_mode);
        
        if let Some(pipeline) = renderer.state.shaders.get_pipeline(pipeline_id) {
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &self.proj_bind_group);
            
            self.batch.render(
                &mut pass, 
                &renderer.state.white_texture,
                &renderer.state.textures,
                Some(&atlas_bg),
            );
        }

        drop(pass);

        encoder.submit_frame(context)?;

        Ok(())
    }

    ///
    pub fn multi_pass_render(&mut self, mw: &mut MoonWalk) -> Result<(), MoonWalkError> {
        let renderer = &mut mw.renderer;
        let context = &mut renderer.context;
        let text_engine = &mut renderer.text_engine;
         
        let mut encoder = BackendEncoder::new(context, "MoonSurface encoder")?;

        // В цикле проходимся по всем дескрипторам проходам рендера которые добавлены в
        // поверхность, создаём реальный RenderPass, устанавливаем ему нужный пайлайн
        // и рендерим в общую текстуру 

        let mut _debug_pass_index = 0;
        for pass in &self.render_passes{
            perf_start!(format!("Render pass: {}", _debug_pass_index));
                let mut objects_filter = Some(&pass.objects);
                if pass.is_empty {
                    objects_filter = None;
                }

                self.batch.prepare(context, &self.store, text_engine, objects_filter);

                // Заливаем глифы только после подготовки батчинга
                text_engine.prepare(context, &renderer.state);
                let atlas_bg = text_engine.get_bind_group()?;

                let mut render_pass = RenderPass::new(
                    &mut encoder,
                    &self.target,
                    Some(pass.color),
                    "MoonWalk render pass (multipass surface render)",
                )?;

                let pipeline_id = Self::map_blend_mode(&renderer.state, pass.blend_mode);
                
                if let Some(pipeline) = renderer.state.shaders.get_pipeline(pipeline_id) {
                    render_pass.set_pipeline(pipeline);
                    render_pass.set_bind_group(0, &self.proj_bind_group);
                    
                    self.batch.render(
                        &mut render_pass,
                        &renderer.state.white_texture,
                        &renderer.state.textures,
                        Some(&atlas_bg),
                    );
                }
            perf_end!(format!("Render pass: {}", _debug_pass_index));

            _debug_pass_index += 1;           
        }

        encoder.submit_frame(context)?;

        Ok(())
    }

    pub fn snapshot(&mut self, mw: &mut MoonWalk, position: Vec2, size: Vec2) -> Result<TextureId, MoonWalkError> {
        let renderer = &mut mw.renderer;
        let format = renderer.context.get_format();

        let mut snapshot_region = ClippedSnapshot::new(
            Vec2::new(position.x as f32, position.y as f32),
            Vec2::new(size.x as f32, size.y as f32),
        );

        snapshot_region.clip_snapshot(Vec2::new(
            self.width as f32,
            self.height as f32
        ));
        
        let mut snapshot_texture = BackendTexture::new(snapshot_region.size.x as u32, snapshot_region.size.y as u32);
        snapshot_texture.config.set_format(format);
        snapshot_texture.create_render_target(&mut renderer.context, snapshot_region.size.x as u32, snapshot_region.size.y as u32)?;

        let id = renderer.state.add_texture(snapshot_texture);
        let target_tex = match renderer.state.textures.get(&id) {
            Some(texture) => texture,
            None => return Err(MoonWalkError::BackendError("Texture not found".to_string()))
        };
        
        let mut encoder = BackendEncoder::new(&mut renderer.context, "Snapshot Encoder")?;

        let source_raw = self.target.get_raw()
            .ok_or(MoonWalkError::BackendError("Source texture not found".to_string()))?;
    
        let target_raw = target_tex.get_raw()
            .ok_or(MoonWalkError::BackendError("Target raw texture not found".to_string()))?;

        encoder.copy_texture_to_texture(
            snapshot_region.position.x as u32,
            snapshot_region.position.y as u32,
            snapshot_region.size.x as u32,
            snapshot_region.size.y as u32,

            source_raw,
            target_raw,
        )?;

        encoder.submit_frame(&mut renderer.context)?;

        Ok(TextureId::new(id))
    }

    pub fn update_snapshot(
        &mut self,
        mw: &mut MoonWalk,
        position: Vec2,
        size: Vec2,
        id: TextureId
    ) -> Result<(), MoonWalkError> {
        let renderer = &mut mw.renderer;
        
        let mut snapshot_region = ClippedSnapshot::new(
            Vec2::new(position.x as f32, position.y as f32),
            Vec2::new(size.x as f32, size.y as f32),
        );

        snapshot_region.clip_snapshot(Vec2::new(
            self.width as f32,
            self.height as f32
        ));

        let target_tex = renderer.state.textures.get(&id.0)
            .ok_or(MoonWalkError::BackendError("Texture not found".to_string()))?;
        
        let mut encoder = BackendEncoder::new(&mut renderer.context, "Update Snapshot Encoder")?;

        let source_raw = self.target.get_raw()
            .ok_or(MoonWalkError::BackendError("Source texture not found".to_string()))?;
    
        let target_raw = target_tex.get_raw()
            .ok_or(MoonWalkError::BackendError("Target raw texture not found".to_string()))?;

        encoder.copy_texture_to_texture(
            snapshot_region.position.x as u32,
            snapshot_region.position.y as u32,
            snapshot_region.size.x as u32,
            snapshot_region.size.y as u32,
            
            source_raw,
            target_raw,
        )?;

        encoder.submit_frame(&mut renderer.context)?;

        Ok(())
    }

    /// Принимает ссылку на состояние рендера и режим смешивания, после
    /// чего матчит режим смешивания в ShaderId который берёт из состояния
    /// рендера. Если пайплайн для этого режима не создан то возвращает
    /// дефолтное значение (Alpha blend mode)
    fn map_blend_mode(state: &RenderState, blend_mode: BlendMode) -> ShaderId {
        match blend_mode {
            BlendMode::Alpha => state.rect_shader,
            BlendMode::Additive => state.rect_shader_additive,
            BlendMode::Multiply => state.rect_shader_multiply,
            BlendMode::Screen => state.rect_shader_screen,
            BlendMode::Subtract => state.rect_shader_subtract,
            _ => state.rect_shader,
        }
    }
}
