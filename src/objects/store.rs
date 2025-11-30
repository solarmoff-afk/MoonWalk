use std::collections::HashMap;
use glam::{Vec2, Vec4};
use textware::{TextWare, FontId};
use crate::objects::{
    ObjectId, Object, Common, Variant, RectData, TextData, BezierData, ShaderId, UniformValue
};

pub struct ObjectStore {
    next_id: u32,
    objects: HashMap<ObjectId, Object>,
    dirty: bool,
    pub rect_shader: ShaderId,
    pub text_shader: ShaderId,
    pub bezier_shader: ShaderId,
}

impl ObjectStore {
    pub fn new(rect_s: ShaderId, text_s: ShaderId, bezier_s: ShaderId) -> Self {
        Self {
            next_id: 1,
            objects: HashMap::new(),
            dirty: true,
            rect_shader: rect_s,
            text_shader: text_s,
            bezier_shader: bezier_s,
        }
    }

    pub fn get_objects(&self) -> &HashMap<ObjectId, Object> {
        &self.objects
    }

    pub fn get_objects_mut(&mut self) -> &mut HashMap<ObjectId, Object> {
        &mut self.objects
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn reset_dirty(&mut self) {
        self.dirty = false;
    }

    pub fn get_default_shader_for(&self, object: &Object) -> ShaderId {
        if object.common.shader.to_u32() != 0 {
            return object.common.shader;
        }

        match object.variant {
            Variant::Rect(_) => self.rect_shader,
            Variant::Text(_) => self.text_shader,
            Variant::Bezier(_) => self.bezier_shader,
        }
    }

    fn new_id(&mut self) -> ObjectId {
        let id = self.next_id;
        self.next_id += 1;
        ObjectId(id)
    }

    pub fn new_rect(&mut self) -> ObjectId {
        let id = self.new_id();
        self.objects.insert(id, Object {
            id,
            common: Common::default(),
            variant: Variant::Rect(RectData::default()),
        });
        self.mark_dirty();
        id
    }

    pub fn new_text(&mut self, textware: &mut TextWare) -> ObjectId {
        let id = self.new_id();
        let default_font_size = 20.0;
        
        let text_obj = textware.create_text("", None, default_font_size, None);

        let mut common = Common::default();
        common.size = Vec2::new(100.0, default_font_size);

        self.objects.insert(id, Object {
            id,
            common,
            variant: Variant::Text(TextData {
                inner: text_obj,
                text_content: String::new(),
                font_size: default_font_size,
            }),
        });
        self.mark_dirty();
        id
    }

    pub fn new_bezier(&mut self) -> ObjectId {
        let id = self.new_id();
        self.objects.insert(id, Object {
            id,
            common: Common::default(),
            variant: Variant::Bezier(BezierData::default()),
        });
        self.mark_dirty();
        id
    }

    pub fn config_text(&mut self, id: ObjectId, content: &str, textware: &mut TextWare) {
        if let Some(obj) = self.objects.get_mut(&id) {
            if let Variant::Text(data) = &mut obj.variant {
                data.text_content = content.to_string();
                textware.update_text(&mut data.inner, content);
                self.mark_dirty();
            }
        }
    }

    pub fn config_size(&mut self, id: ObjectId, size: Vec2, textware: &mut TextWare) {
        if let Some(obj) = self.objects.get_mut(&id) {
            obj.common.size = size;

            if let Variant::Text(data) = &mut obj.variant {
                if (data.font_size - size.y).abs() > 0.001 {
                    data.font_size = size.y;
                    textware.resize_text(&mut data.inner, data.font_size, None);
                }
                
                textware.set_size(&mut data.inner, Some(size.x), Some(size.y));
            }

            self.mark_dirty();
        }
    }

    pub fn config_position(&mut self, id: ObjectId, pos: Vec2) {
        if let Some(obj) = self.objects.get_mut(&id) {
            obj.common.position = pos;
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
            if let Variant::Text(data) = &mut obj.variant {
                data.inner.color = color.to_array();
            }
            self.mark_dirty();
        }
    }

    pub fn config_z_index(&mut self, id: ObjectId, z: f32) {
        if let Some(obj) = self.objects.get_mut(&id) {
            obj.common.z = z;
            self.mark_dirty();
        }
    }

    pub fn config_rounded(&mut self, id: ObjectId, radii: Vec4) {
        if let Some(obj) = self.objects.get_mut(&id) {
            if let Variant::Rect(data) = &mut obj.variant {
                data.radii = radii;
                self.mark_dirty();
            }
        }
    }

    pub fn config_font(&mut self, id: ObjectId, font_id: FontId, textware: &mut TextWare) {
        if let Some(obj) = self.objects.get_mut(&id) {
            if let Variant::Text(data) = &mut obj.variant {
                let new_text = textware.create_text(&data.text_content, Some(font_id), data.font_size, None);
                data.inner = new_text;
                data.inner.color = obj.common.color.to_array();
                textware.set_size(&mut data.inner, Some(obj.common.size.x), Some(obj.common.size.y));
                
                self.mark_dirty();
            }
        }
    }

    pub fn set_bezier_points(&mut self, id: ObjectId, points: Vec<Vec2>) {
        if let Some(obj) = self.objects.get_mut(&id) {
            if let Variant::Bezier(data) = &mut obj.variant {
                data.points = points;
                self.mark_dirty();
            }
        }
    }

    pub fn config_bezier_thickness(&mut self, id: ObjectId, thickness: f32) {
        if let Some(obj) = self.objects.get_mut(&id) {
            if let Variant::Bezier(data) = &mut obj.variant {
                data.thickness = thickness;
                self.mark_dirty();
            }
        }
    }

    pub fn config_bezier_smooth(&mut self, id: ObjectId, smoothing: f32) {
        if let Some(obj) = self.objects.get_mut(&id) {
            if let Variant::Bezier(data) = &mut obj.variant {
                data.smoothing = smoothing;
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

    pub fn set_parent(&mut self, child_id: ObjectId, parent_id: ObjectId) {
        if let Some(child) = self.objects.get_mut(&child_id) {
            child.common.parent = Some(parent_id);
            self.mark_dirty();
        }
    }

    pub fn set_masking(&mut self, id: ObjectId, enable: bool) {
        if let Some(obj) = self.objects.get_mut(&id) {
            obj.common.mask_children = enable;
            self.mark_dirty();
        }
    }
}