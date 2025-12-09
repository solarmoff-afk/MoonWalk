// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use glam::{Mat4, Vec3};
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct MatrixUniform {
    pub model: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    pub projection: [[f32; 4]; 4],
}

pub struct MatrixStack {
    pub projection: Mat4,
    pub view: Mat4,
    pub model: Mat4,
    stack: Vec<Mat4>,
}

impl MatrixStack {
    pub fn new() -> Self {
        Self {
            projection: Mat4::IDENTITY,
            view: Mat4::IDENTITY,
            model: Mat4::IDENTITY,
            stack: Vec::with_capacity(16),
        }
    }

    pub fn set_ortho(&mut self, width: f32, height: f32) {
        self.projection = Mat4::orthographic_lh(0.0, width, height, 0.0, -1.0, 1.0);
    }

    pub fn set_identity(&mut self) {
        self.model = Mat4::IDENTITY;
    }

    pub fn push(&mut self) {
        self.stack.push(self.model);
    }

    pub fn pop(&mut self) {
        if let Some(m) = self.stack.pop() {
            self.model = m;
        }
    }

    pub fn translate(&mut self, pos: Vec3) {
        self.model = self.model * Mat4::from_translation(pos);
    }

    pub fn rotate_z(&mut self, angle_deg: f32) {
        self.model = self.model * Mat4::from_rotation_z(angle_deg.to_radians());
    }

    pub fn scale(&mut self, scale: Vec3) {
        self.model = self.model * Mat4::from_scale(scale);
    }

    pub fn to_uniform(&self) -> MatrixUniform {
        MatrixUniform {
            model: self.model.to_cols_array_2d(),
            view: self.view.to_cols_array_2d(),
            projection: self.projection.to_cols_array_2d(),
        }
    }
}