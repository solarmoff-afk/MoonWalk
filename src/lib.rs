#![allow(unsafe_code)]

mod error;
mod ffi_utils;
mod objects;
mod renderer;
mod rendering;
mod font;

pub mod ffi_functions;

use glam::{Mat4, Vec2, Vec3, Vec4};
use objects::UniformValue;
use renderer::Renderer;
use font::{FontSystem, FontId};
use raw_window_handle::{HasWindowHandle, HasDisplayHandle};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId(u32);
impl From<u32> for ObjectId { fn from(id: u32) -> Self { Self(id) } }
impl ObjectId { pub fn to_u32(&self) -> u32 { self.0 } }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShaderId(u32);
impl From<u32> for ShaderId { fn from(id: u32) -> Self { Self(id) } }
impl ShaderId { pub fn to_u32(&self) -> u32 { self.0 } }

pub struct MoonWalkState<'a> {
    renderer: Renderer<'a>,
    font_system: FontSystem,
}

impl<'a> MoonWalkState<'a> {
    /* 
        Создает новый экземпляр состояния мунволка
            window - Объект, реализующий HasWindowHandle и HasDisplayHandle
    */

    pub fn new(
        window: &'a (impl HasWindowHandle + HasDisplayHandle + Send + Sync)
    ) -> Result<Self, error::MoonWalkError> {
        let renderer = pollster::block_on(Renderer::new(window))?;
        let font_system = FontSystem::new();
        Ok(Self { renderer, font_system })
    }

    // Устанавливает размеры области вьюпорта
    pub fn set_viewport(&mut self, width: u32, height: u32) {
        self.renderer.set_viewport(width, height);
    }

    // Выполняет рендеринг одного кадра с указанным цветом фона
    pub fn render_frame(&mut self, background_color: Vec4) -> Result<(), renderer::RenderError> {
        self.renderer.render_frame(&mut self.font_system, background_color)
    }

    // Создает новый прямоугольник
    pub fn new_rect(&mut self) -> ObjectId {
        self.renderer.new_rect()
    }

    // Создает новый текст.
    pub fn new_text(&mut self) -> ObjectId {
        self.renderer.new_text(self.font_system.cosmic_mut())
    }

    // Задает позицию объекта
    pub fn config_position(&mut self, id: ObjectId, position: Vec2) {
        self.renderer.config_position(id, position);
    }

    // Задает размеры объекта
    pub fn config_size(&mut self, id: ObjectId, size: Vec2) {
        self.renderer.config_size(id, size, self.font_system.cosmic_mut());
    }
    
    // Задает вращение объекта в градусах
    pub fn config_rotation(&mut self, id: ObjectId, angle_degrees: f32) {
        self.renderer.config_rotation(id, angle_degrees);
    }

    // Задает цвет объекта
    pub fn config_color(&mut self, id: ObjectId, color: Vec4) {
        self.renderer.config_color(id, color);
    }
    
    // Задает Z индекс (порядок отрисовки) объекта
    pub fn config_z_index(&mut self, id: ObjectId, z_index: f32) {
        self.renderer.config_z_index(id, z_index);
    }
    
    // Устанавливает текст для текста (стандартная ситуация)
    pub fn config_text(&mut self, id: ObjectId, text: &str) {
        self.renderer.config_text(id, text, &mut self.font_system);
    }
    
    // Загружает шрифт из файла
    pub fn load_font(&mut self, path: &str, size: f32) -> Result<FontId, std::io::Error> {
        self.font_system.load_font(path, size)
    }

    // Удаляет ранее загруженный шрифт
    pub fn clear_font(&mut self, font_id: FontId) {
        self.font_system.clear_font(font_id);
    }

    // Применяет шрифт к текстовому объекту
    pub fn config_font(&mut self, object_id: ObjectId, font_id: FontId) {
        self.renderer.config_font(object_id, font_id);
    }

    // Устанавливает радиусы скругления углов для объекта
    //  corners - это Vec4 (верх-лево, верх-право, низ-право, нищ-лево)
    pub fn set_rounded(&mut self, object_id: ObjectId, corners: Vec4) {
        self.renderer.config_rounded(object_id, corners);
    }

    // Удаляет объект
    pub fn delete_object(&mut self, id: ObjectId) {
        self.renderer.delete_object(id);
    }

    // Удаляет все объекты со сцены
    pub fn clear_all(&mut self) {
        self.renderer.clear_all_objects();
    }
    
    // Компилирует шейдер
    pub fn compile_shader(&mut self, shader_source: &str) -> Result<ShaderId, renderer::ShaderError> {
        self.renderer.compile_shader(shader_source)
    }
    
    // Применяет шейдер к объекту
    pub fn set_object_shader(&mut self, object_id: ObjectId, shader_id: ShaderId) {
        self.renderer.set_object_shader(object_id, shader_id);
    }
    
    // Устанавливает uniform-переменную для объекта
    pub fn set_uniform(&mut self, id: ObjectId, name: String, value: UniformValue) {
        self.renderer.set_uniform(id, name, value);
    }
}