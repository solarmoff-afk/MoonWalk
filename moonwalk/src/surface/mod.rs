// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

pub mod objects;
pub mod getter;

use glam::{Vec2, Vec4};
use moonwalk_backend::core::context::BackendContext;
use moonwalk_backend::core::encoder::BackendEncoder;
use moonwalk_backend::core::buffer::BackendBuffer;
use moonwalk_backend::render::pass::RenderPass;
use moonwalk_backend::render::texture::BackendTexture;
use moonwalk_backend::pipeline::bind::{BindGroup, RawBindGroup};
use moonwalk_backend::pipeline::types::ShaderStage;
use moonwalk_backend::error::MoonBackendError;

use crate::gpu::MatrixStack;
use crate::rendering::state::GlobalUniform;
use crate::batching::shapes::uber::UberBatch;
use crate::rendering::snapshot::ClippedSnapshot;
use crate::objects::store::ObjectStore;
use crate::MoonWalk;
use crate::ObjectId;
use crate::FontAsset;
use crate::textware::FontId;
use crate::error::MoonWalkError;

/// Структура поверхности рендера, определяет общий api для главного рендера
/// и рендер контейнеров
pub struct MoonSurface {
    pub store: ObjectStore,
    pub batch: UberBatch,
    pub proj_bind_group: RawBindGroup,
    pub target: BackendTexture,
    pub width: u32,
    pub height: u32, 
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
        let internal_id = FontId(font.0);
        self.store.new_text(content.to_string(), internal_id, size)
    }

    /// Отрисовать все объекты на surface
    pub fn render(&mut self, mw: &mut MoonWalk, clear_color: Option<Vec4>) -> Result<(), MoonWalkError> {
        let renderer = &mut mw.renderer;
        let context = &mut renderer.context;
        let text_engine = &mut renderer.text_engine;
        
        self.batch.prepare(context, &self.store, text_engine);

        text_engine.prepare(context);
        let atlas_bg = text_engine.get_bind_group()?;
        
        let clear_color = clear_color.map(|c| Vec4::new(c.x, c.y, c.z, c.w));

        let mut encoder = BackendEncoder::new(context, "Render Container Encoder")?;

        let mut pass = RenderPass::new(
            &mut encoder,
            &self.target,
            clear_color,
            "Render Container Pass"
        )?;

        if let Some(pipeline) = renderer.state.shaders.get_pipeline(renderer.state.rect_shader) {
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

    pub fn snapshot(&mut self, mw: &mut MoonWalk, position: Vec2, size: Vec2) -> Result<u32, MoonWalkError> {
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
            None => return Err(MoonWalkError::BackendError("Context not found".to_string()))
        };
        
        let mut encoder = BackendEncoder::new(&mut renderer.context, "Snapshot Encoder")?;

        encoder.copy_texture_to_texture(
            snapshot_region.position.x as u32,
            snapshot_region.position.y as u32,
            snapshot_region.size.x as u32,
            snapshot_region.size.y as u32,
            &self.target.get_raw().expect("Target texture not inited"),
            &target_tex.get_raw().expect("Target texture not inited"),
        )?;

        encoder.submit_frame(&mut renderer.context)?;

        Ok(id)
    }

    pub fn update_snapshot(
        &mut self,
        mw: &mut MoonWalk,
        position: Vec2,
        size: Vec2,
        id: u32
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

        let target_tex = renderer.state.textures.get(&id).unwrap();
        
        let mut encoder = BackendEncoder::new(&mut renderer.context, "Update Snapshot Encoder")?;

        encoder.copy_texture_to_texture(
            snapshot_region.position.x as u32,
            snapshot_region.position.y as u32,
            snapshot_region.size.x as u32,
            snapshot_region.size.y as u32,
            &self.target.get_raw().unwrap(),
            &target_tex.get_raw().unwrap()
        )?;

        encoder.submit_frame(&mut renderer.context)?;

        Ok(())
    }
}