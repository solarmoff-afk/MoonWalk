// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use bytemuck::{Pod, Zeroable};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MeshId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LightId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowQuality {
    Off, // Нет теней
    Low, // Низкое качество
    Medium, // Среднее качество
    High, // Высокое качество
    Ultra, // Ультра качество
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightingModel {
    Pbr, // Стандартное освещение
    Phong, // Простое освещение (выглядит пластиково, но легче для железа)
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex3D {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub tangent: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, Default)]
pub struct Light {
    pub position: [f32; 3],
    pub _pad1: f32,
    pub color: [f32; 3],
    pub intensity: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub(crate) struct GlobalUniform {
    pub view_proj: [[f32; 4]; 4],
    pub camera_pos: [f32; 3],
    pub num_lights: u32,
    pub lights: [Light; crate::core::config::MAX_LIGHTS],
    pub ambient_color: [f32; 3],
    pub shadows_enabled: f32,
    pub light_view_projs: [[[f32; 4]; 4]; crate::core::config::MAX_SHADOWS],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub(crate) struct ShadowUniform {
    pub light_view_proj: [[f32; 4]; 4],
    pub atlas_offset: [f32; 2],
    pub atlas_scale: f32,
    pub _pad: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub(crate) struct MaterialFlags {
    pub use_albedo_map: u32,
    pub use_normal_map: u32,
    pub use_mr_map: u32,
    pub _pad: u32,
}
