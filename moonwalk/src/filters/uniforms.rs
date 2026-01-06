// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct DummyVertex {
    pub _dummy: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct BlurUniform {
    pub direction: [f32; 2],
    pub radius: f32,
    pub _pad: f32,
    pub resolution: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ColorMatrixUniform {
    pub matrix: [[f32; 4]; 4],
    pub offset: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct AdvancedUniform {
    pub key_color: [f32; 3],
    pub tolerance: f32,
    pub params: [f32; 4],
}
