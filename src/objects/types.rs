use cosmic_text::Buffer;
use glam::{Mat4, Vec2, Vec3, Vec4};
use rustc_hash::FxHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use crate::font::FontId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ObjectId(pub u32);
impl From<u32> for ObjectId { fn from(id: u32) -> Self { Self(id) } }
impl ObjectId { pub fn to_u32(self) -> u32 { self.0 } }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ShaderId(pub u32);
impl From<u32> for ShaderId { fn from(id: u32) -> Self { Self(id) } }
impl ShaderId { pub fn to_u32(self) -> u32 { self.0 } }

#[derive(Clone)]
pub enum UniformValue {
    Int(i32), Float(f32), Vec2(Vec2), Vec3(Vec3), Vec4(Vec4), Mat4(Mat4), Bool(bool),
}

#[derive(Clone, Default)]
pub struct Common {
    pub position: Vec2, pub size: Vec2, pub rotation: f32, pub color: Vec4,
    pub z: f32, pub shader: ShaderId, pub uniforms: HashMap<String, UniformValue>,
}

#[derive(Clone, Default)]
pub struct RectData {
    pub radii: Vec4,
}

pub struct TextData {
    pub buffer: Buffer,
    pub text: String,
    pub font_id: Option<FontId>,
}

pub enum Variant {
    Rect(RectData),
    Text(TextData),
}

pub struct Object {
    pub id: ObjectId,
    pub common: Common,
    pub variant: Variant,
}

pub fn hash_uniforms(uniforms: &HashMap<String, UniformValue>) -> u64 {
    let mut hasher = FxHasher::default();
    let mut keys: Vec<&String> = uniforms.keys().collect();
    
    keys.sort();

    for key in keys {
        key.hash(&mut hasher);
        
        match &uniforms[key] {
            UniformValue::Int(v) => v.hash(&mut hasher),
            UniformValue::Float(v) => v.to_bits().hash(&mut hasher),
            UniformValue::Vec2(v) => { v.x.to_bits().hash(&mut hasher); v.y.to_bits().hash(&mut hasher); }
            UniformValue::Vec3(v) => { v.x.to_bits().hash(&mut hasher); v.y.to_bits().hash(&mut hasher); v.z.to_bits().hash(&mut hasher); }
            UniformValue::Vec4(v) => { v.x.to_bits().hash(&mut hasher); v.y.to_bits().hash(&mut hasher); v.z.to_bits().hash(&mut hasher); v.w.to_bits().hash(&mut hasher); }
            UniformValue::Mat4(v) => v.to_cols_array().iter().for_each(|f| f.to_bits().hash(&mut hasher)),
            UniformValue::Bool(v) => v.hash(&mut hasher),
        }
    }

    hasher.finish()
}