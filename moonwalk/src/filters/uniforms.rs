// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use bytemuck::{Pod, Zeroable};
use crevice::std430::{AsStd430, Vec2, Vec4};

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

#[derive(Clone, Copy, Debug, AsStd430)]
pub struct LiquidGlassUniform {
    // Геометрия
    pub size: Vec2,
    pub offset: Vec2,
    pub corner_radius: Vec4,
    pub resolution: Vec2,

    // Параметры жидкого стекла
    pub refraction_height: f32,
    pub refraction_amount: f32,
    pub depth_effect: f32,
    pub chromatic_aberration: f32,

    pub rotation: f32,
    pub gamma: f32,

    pub unused_color: Vec4,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct MeshGradientUniform {
    pub colors: [[f32; 4]; 9],
    pub positions: [[f32; 4]; 9],
    pub resolution: [f32; 2],
    pub noise_intensity: f32,
    pub warp_strength: f32,
    pub warp_phase: f32,
    pub gamma: f32,
    pub blend_mode: u32,
    pub pad0: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LiquidGlassMaskUniform {
    pub size: [f32; 2],
    pub offset: [f32; 2],
    pub resolution: [f32; 2],
    pub refraction_amount: f32,
    pub refraction_height: f32, 
    pub depth_effect: f32,
    pub chromatic_aberration: f32,
    pub tolerance: f32,
    pub gamma: f32,
    pub _pad: [f32; 4],
}
