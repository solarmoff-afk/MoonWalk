// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use glam::{Vec3, Vec4};
use bytemuck::{Pod, Zeroable};
use crate::rendering::custom::MoonBuffer;

pub const MAX_LIGHTS: usize = 4;

/// [WAIT DOC]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MeshId(pub usize);

/// [WAIT DOC]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InstanceId(pub usize);

/// [WAIT DOC]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LightId(pub usize);

/// [WAIT DOC]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowQuality {
    Off,
    Low,
    Medium,
    High,
    Ultra,
}

/// [WAIT DOC]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightingModel {
    Pbr,
    Phong,
    Custom,
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
pub struct LightRaw {
    pub position: [f32; 3],
    pub _pad1: f32,
    pub color: [f32; 3],
    pub intensity: f32,
}

pub type Light = LightRaw;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub(crate) struct GlobalUniform {
    pub view_proj: [[f32; 4]; 4],
    pub camera_pos: [f32; 3],
    pub num_lights: u32,
    pub lights: [LightRaw; MAX_LIGHTS],
    pub ambient_color: [f32; 3],
    pub shadows_enabled: f32,
    pub light_view_proj: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub(crate) struct ShadowUniform {
    pub light_view_proj: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub(crate) struct MaterialFlags {
    pub use_albedo_map: u32,
    pub use_normal_map: u32,
    pub use_mr_map: u32,
    pub _pad: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub(crate) struct InstanceRaw {
    pub model: [[f32; 4]; 4],
    pub normal_mat_0: [f32; 4],
    pub normal_mat_1: [f32; 4],
    pub normal_mat_2: [f32; 4],
    pub color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub unlit: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Material {
    pub albedo_id: Option<u32>,
    pub normal_id: Option<u32>,
    pub mr_id: Option<u32>, 
    pub base_color: Vec4,
    pub metallic: f32,
    pub roughness: f32,
    pub unlit: bool,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            albedo_id: None,
            normal_id: None,
            mr_id: None,
            base_color: Vec4::ONE,
            metallic: 0.0,
            roughness: 0.5,
            unlit: false,
        }
    }
}

pub struct MeshData {
    pub vertex_buffer: MoonBuffer,
    pub index_buffer: MoonBuffer,
    pub index_count: u32,
    pub local_material: Option<Material>, 
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub(crate) struct BatchKey {
    pub mesh_id: usize,
    pub albedo_id: u32,
    pub normal_id: u32,
    pub mr_id: u32,
}

pub struct ModelInstance {
    pub mesh_id: usize,
    pub material: Option<Material>,
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}
