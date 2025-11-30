use std::sync::Arc;
use glam::{Vec4, Vec2};
use easy_gpu::{Context, MatrixStack};
use textware::{TextWare, FontId};

use crate::error::MoonWalkError;
use crate::objects::{ObjectStore, ObjectId, UniformValue, ShaderId};
use crate::rendering::shader::ShaderStore;
use crate::rendering::batch::BatchSystem;

pub struct Renderer {
    context: Context,
    textware: TextWare,
    object_store: ObjectStore,
    shader_store: ShaderStore,
    batch_system: BatchSystem,
    matrix_stack: MatrixStack,
}

impl Renderer {
    pub async fn new(
        window: impl raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle + Send + Sync + 'static,
        width: u32,
        height: u32
    ) -> Result<Self, MoonWalkError> {
        let window = Arc::new(window);
        let context = Context::new(window, width, height).await;
        let queue = &context.queue;
        let device = &context.device;

        let textware = TextWare::new(device, queue);
        Self::init(context, textware)
    }

    #[cfg(target_os = "android")]
    pub async fn new_android(
        window: impl raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle + Send + Sync + 'static,
        asset_manager: ndk::asset::AssetManager,
        width: u32, 
        height: u32
    ) -> Result<Self, MoonWalkError> {
        let window = Arc::new(window);
        let context = Context::new(window, width, height).await;
        let textware = TextWare::new(&context.device, &context.queue, asset_manager);
        Self::init(context, textware)
    }

    fn init(context: Context, textware: TextWare) -> Result<Self, MoonWalkError> {
        let mut shader_store = ShaderStore::new(&context);
        let rect_s = shader_store.create_default_rect(&context, context.config.format)?;
        let text_s = shader_store.create_default_text(&context, context.config.format)?;
        let bez_s = shader_store.create_default_bezier(&context, context.config.format)?;

        let object_store = ObjectStore::new(rect_s, text_s, bez_s);
        let batch_system = BatchSystem::new();
        let mut matrix_stack = MatrixStack::new();
        
        matrix_stack.set_ortho(context.config.width as f32, context.config.height as f32);

        Ok(Self {
            context,
            textware,
            object_store,
            shader_store,
            batch_system,
            matrix_stack,
        })
    }

    pub fn set_viewport(&mut self, width: u32, height: u32) {
        self.context.resize(width, height);
        self.matrix_stack.set_ortho(width as f32, height as f32);
        self.object_store.mark_dirty();
    }

    pub fn render_frame(&mut self, clear_color: Vec4) -> Result<(), wgpu::SurfaceError> {
        self.textware.prepare(&self.context.queue);

        if self.object_store.is_dirty() {
            self.batch_system.rebuild(
                &self.context, 
                &mut self.object_store, 
                &mut self.textware, 
                self.context.config.width, 
                self.context.config.height
            );
            self.object_store.reset_dirty();
        }

        let mut encoder = self.context.create_encoder();
        let output = self.context.surface.as_ref().unwrap().get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let cc = wgpu::Color { r: clear_color.x as f64, g: clear_color.y as f64, b: clear_color.z as f64, a: clear_color.w as f64 };
        
        let mut pass = easy_gpu::RenderPass::new(&mut encoder, &view, Some(cc));
        
        self.batch_system.render(&mut pass, &self.shader_store, &self.textware, &self.matrix_stack, &self.context);

        drop(pass);
        self.context.submit(encoder);
        output.present();

        Ok(())
    }

    pub fn new_rect(&mut self) -> ObjectId {
        self.object_store.new_rect()
    }

    pub fn new_text(&mut self) -> ObjectId {
        self.object_store.new_text(&mut self.textware)
    }

    pub fn new_bezier(&mut self) -> ObjectId {
        self.object_store.new_bezier()
    }
    
    pub fn config_size(&mut self, id: ObjectId, size: glam::Vec2, _unused: ()) {
        self.object_store.config_size(id, size, &mut self.textware);
    }
    
    pub fn config_text(&mut self, id: ObjectId, content: &str, _unused: ()) {
        self.object_store.config_text(id, content, &mut self.textware);
    }

    pub fn load_font_bytes(&mut self, data: &[u8], name: &str) -> Result<FontId, MoonWalkError> {
        self.textware.load_font_bytes(data, name).map_err(|e| MoonWalkError::TextError(e))
    }
    
    pub fn config_position(&mut self, id: ObjectId, pos: glam::Vec2) {
        self.object_store.config_position(id, pos);
    }

    pub fn config_rotation(&mut self, id: ObjectId, a: f32) {
        self.object_store.config_rotation(id, a);
    }

    pub fn config_color(&mut self, id: ObjectId, c: Vec4) {
        self.object_store.config_color(id, c);
    }

    pub fn config_z_index(&mut self, id: ObjectId, z: f32) {
        self.object_store.config_z_index(id, z);
    }
    
    pub fn config_rounded(&mut self, id: ObjectId, radii: Vec4) {
        self.object_store.config_rounded(id, radii);
    }
    
    pub fn delete_object(&mut self, id: ObjectId) {
        self.object_store.delete_object(id);
    }
    
    pub fn clear_all_objects(&mut self) {
        self.object_store.clear_all();
    }
    
    pub fn compile_shader(&mut self, src: &str) -> Result<ShaderId, MoonWalkError> {
        self.shader_store.compile_shader(&self.context, src, self.context.config.format)
    }
    
    pub fn set_object_shader(&mut self, id: ObjectId, shader_id: ShaderId) {
        self.object_store.set_object_shader(id, shader_id);
    }
    
    pub fn set_uniform(&mut self, id: ObjectId, name: String, value: UniformValue) {
        self.object_store.set_uniform(id, name, value);
    }
    
    pub fn set_bezier_points(&mut self, id: ObjectId, points: Vec<Vec2>) {
        self.object_store.set_bezier_points(id, points);
    }
    
    pub fn config_bezier_thickness(&mut self, id: ObjectId, thickness: f32) {
        self.object_store.config_bezier_thickness(id, thickness);
    }
    
    pub fn config_bezier_smooth(&mut self, id: ObjectId, smoothing: f32) {
        self.object_store.config_bezier_smooth(id, smoothing);
    }
    
    pub fn set_parent(&mut self, child: ObjectId, parent: ObjectId) {
        self.object_store.set_parent(child, parent);
    }
    
    pub fn set_masking(&mut self, id: ObjectId, enable: bool) {
        self.object_store.set_masking(id, enable);
    }
    
    pub fn config_font(&mut self, id: ObjectId, font_id: FontId) {
        self.object_store.config_font(id, font_id, &mut self.textware);
    }
}