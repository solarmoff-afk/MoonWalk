pub mod types;

pub use self::types::*;

use std::collections::HashMap;
use glam::{Vec2, Vec4};
use cosmic_text::{Attrs, Buffer, Family, FontSystem as CosmicFontSystem, Metrics};
use crate::font::{FontId, FontSystem};

pub struct ObjectStore {
    next_id: u32,
    objects: HashMap<ObjectId, Object>,
    dirty: bool,
    default_rect_shader: ShaderId,
    default_text_shader: ShaderId,
    default_bezier_shader: ShaderId,
}

impl ObjectStore {
    pub fn new(default_rect_shader: ShaderId, default_text_shader: ShaderId, default_bezier_shader: ShaderId) -> Self {
        Self {
            next_id: 1,
            objects: HashMap::new(),
            dirty: true,
            default_rect_shader,
            default_text_shader,
            default_bezier_shader,
        }
    }

    pub fn get_objects(&self) -> &HashMap<ObjectId, Object> {
        &self.objects
    }

    pub fn get_default_shader_for(&self, object: &Object) -> ShaderId {
        if object.common.shader.to_u32() != 0 {
            return object.common.shader;
        }
        
        match object.variant {
            Variant::Rect(_) => self.default_rect_shader,
            Variant::Text(_) => self.default_text_shader,
            Variant::Bezier(_) => self.default_bezier_shader,
        }
    }

    fn new_id(&mut self) -> ObjectId {
        let id = self.next_id;
        self.next_id += 1;
        ObjectId::from(id)
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn new_rect(&mut self) -> ObjectId {
        let id = self.new_id();
        
        let object = Object {
            id,
            common: Common::default(),
            variant: Variant::Rect(RectData::default()),
        };
        
        self.objects.insert(id, object);
        self.mark_dirty();
        id
    }

    pub fn new_text(&mut self, font_system: &mut CosmicFontSystem) -> ObjectId {
        let mut buffer = Buffer::new(font_system, Metrics::new(16.0, 20.0));
        let id = self.new_id();
        
        buffer.set_size(font_system, 100.0, 20.0);
        buffer.shape_until_scroll(font_system);

        let mut common = Common::default();
        common.size = Vec2::new(100.0, 20.0);

        let object = Object {
            id,
            common,
            variant: Variant::Text(TextData {
                buffer,
                text: String::new(),
                font_id: None,
            }),
        };
        
        self.objects.insert(id, object);
        self.mark_dirty();
        id
    }

    pub fn new_bezier(&mut self) -> ObjectId {
        let id = self.new_id();
        
        let object = Object {
            id,
            common: Common::default(),
            variant: Variant::Bezier(BezierData::default()),
        };
        
        self.objects.insert(id, object);
        self.mark_dirty();
        id
    }

    pub fn config_position(&mut self, id: ObjectId, pos: Vec2) {
        if let Some(obj) = self.objects.get_mut(&id) {
            obj.common.position = pos;
            self.mark_dirty();
        }
    }

    pub fn config_size(&mut self, id: ObjectId, size: Vec2, font_system: &mut CosmicFontSystem) {
        if let Some(obj) = self.objects.get_mut(&id) {
            obj.common.size = size;
            
            if let Variant::Text(ref mut text_data) = obj.variant {
                text_data.buffer.set_size(font_system, size.x, size.y);
                text_data.buffer.shape_until_scroll(font_system);
            }
            
            self.mark_dirty();
        }
    }
    
    pub fn config_rotation(&mut self, id: ObjectId, angle_degrees: f32) {
        if let Some(obj) = self.objects.get_mut(&id) {
            obj.common.rotation = angle_degrees;
            self.mark_dirty();
        }
    }

    pub fn config_color(&mut self, id: ObjectId, color: Vec4) {
        if let Some(obj) = self.objects.get_mut(&id) {
            obj.common.color = color;
            self.mark_dirty();
        }
    }

    pub fn config_z_index(&mut self, id: ObjectId, z: f32) {
        if let Some(obj) = self.objects.get_mut(&id) {
            obj.common.z = z;
            self.mark_dirty();
        }
    }

    pub fn config_text(&mut self, id: ObjectId, text: &str, font_system: &mut FontSystem) {
        if let Some(obj) = self.objects.get_mut(&id) {
            if let Variant::Text(ref mut text_data) = obj.variant {
                let (family_name, size) = if let Some(font_id) = text_data.font_id {
                    font_system.get_font_info(font_id).unwrap_or_else(|| ("sans-serif".to_string(), 16.0))
                } else {
                    ("sans-serif".to_string(), 16.0)
                };

                let attrs = Attrs::new().family(Family::Name(&family_name));

                let cosmic_fs = font_system.cosmic_mut();
                
                text_data.buffer.set_metrics(cosmic_fs, Metrics::new(size, size * 1.25));
                text_data.buffer.set_text(cosmic_fs, text, attrs);
                text_data.buffer.shape_until_scroll(cosmic_fs);
                
                text_data.text = text.to_string();
                self.mark_dirty();
            }
        }
    }

    pub fn config_rounded(&mut self, id: ObjectId, radii: Vec4) {
        if let Some(obj) = self.objects.get_mut(&id) {
            if let Variant::Rect(ref mut data) = obj.variant {
                data.radii = radii;
                self.mark_dirty();
            }
        }
    }
    
    pub fn config_font(&mut self, id: ObjectId, font_id: FontId) {
        if let Some(obj) = self.objects.get_mut(&id) {
            if let Variant::Text(ref mut text_data) = obj.variant {
                text_data.font_id = Some(font_id);
                self.mark_dirty();
            }
        }
    }

    pub fn set_bezier_points(&mut self, id: ObjectId, points: Vec<Vec2>) {
        if let Some(obj) = self.objects.get_mut(&id) {
            if let Variant::Bezier(ref mut bezier_data) = obj.variant {
                bezier_data.points = points;
                self.mark_dirty();
            }
        }
    }

    pub fn config_bezier_thickness(&mut self, id: ObjectId, thickness: f32) {
        if let Some(obj) = self.objects.get_mut(&id) {
            if let Variant::Bezier(ref mut bezier_data) = obj.variant {
                bezier_data.thickness = thickness;
                self.mark_dirty();
            }
        }
    }

    pub fn config_bezier_smooth(&mut self, id: ObjectId, smoothing: f32) {
        if let Some(obj) = self.objects.get_mut(&id) {
            if let Variant::Bezier(ref mut bezier_data) = obj.variant {
                bezier_data.smoothing = smoothing;
                self.mark_dirty();
            }
        }
    }

    pub fn delete_object(&mut self, id: ObjectId) {
        if self.objects.remove(&id).is_some() {
            self.mark_dirty();
        }
    }

    pub fn clear_all(&mut self) {
        self.objects.clear();
        self.next_id = 1;
        self.mark_dirty();
    }
    
    pub fn set_object_shader(&mut self, id: ObjectId, shader_id: ShaderId) {
        if let Some(obj) = self.objects.get_mut(&id) {
            obj.common.shader = shader_id;
            self.mark_dirty();
        }
    }
    
    pub fn set_uniform(&mut self, id: ObjectId, name: String, value: UniformValue) {
        if let Some(obj) = self.objects.get_mut(&id) {
            obj.common.uniforms.insert(name, value);
            self.mark_dirty();
        }
    }
    
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
    
    pub fn reset_dirty(&mut self) {
        self.dirty = false;
    }
}