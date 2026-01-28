// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use mlua::UserDataMethods;
use glam::{Vec3, Vec4, Mat4};

use super::MoonWalkLuaWrapper;

pub fn register<'lua, M: UserDataMethods<'lua, MoonWalkLuaWrapper>>(methods: &mut M) {
    methods.add_method_mut("blur_texture", |_, this, (id, radius, horizontal): (u32, f32, bool)| {
        this.get_mut().blur_texture(id, radius, horizontal);
        Ok(())
    });

    methods.add_method_mut("brightness", |_, this, (id, val): (u32, f32)| {
        this.get_mut().brightness(id, val);
        Ok(())
    });

    methods.add_method_mut("contrast", |_, this, (id, val): (u32, f32)| {
        this.get_mut().contrast(id, val);
        Ok(())
    });

    methods.add_method_mut("saturation", |_, this, (id, val): (u32, f32)| {
        this.get_mut().saturation(id, val);
        Ok(())
    });

    methods.add_method_mut("hue_shift", |_, this, (id, deg): (u32, f32)| {
        this.get_mut().hue_shift(id, deg);
        Ok(())
    });

    methods.add_method_mut("chromakey", |_, this, (id, r, g, b, tol): (u32, f32, f32, f32, f32)| {
        this.get_mut().chromakey(id, Vec3::new(r, g, b), tol);
        Ok(())
    });

    methods.add_method_mut("apply_mask", |_, this, (target, mask, invert): (u32, u32, bool)| {
        this.get_mut().apply_mask(target, mask, invert);
        Ok(())
    });
    
    methods.add_method_mut("color_matrix", |_, this, (id, mat_vals, offset_vals): (u32, Vec<f32>, Vec<f32>)| {
        if mat_vals.len() != 16 || offset_vals.len() != 4 {
            return Err(mlua::Error::RuntimeError("Invalid matrix or offset size".into()));
        }

        let mat = Mat4::from_cols_array(&mat_vals.try_into().unwrap());
        let off = Vec4::from_slice(&offset_vals);
        
        this.get_mut().color_matrix(id, mat, off);
        Ok(())
    });
}