// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use mlua::{UserDataMethods, Error};

use super::MoonWalkLuaWrapper;

pub fn register<'lua, M: UserDataMethods<'lua, MoonWalkLuaWrapper>>(methods: &mut M) {
    methods.add_method_mut("load_texture", |_, this, path: String| {
        this.get_mut().load_texture(&path)
            .map_err(|e| Error::RuntimeError(e.to_string()))
    });

    methods.add_method_mut("remove_texture", |_, this, tex_id: u32| {
        this.get_mut().remove_texture(tex_id);
        Ok(())
    });

    methods.add_method_mut("load_font", |_, this, (path, name): (String, String)| {
        let font_asset = this.get_mut().load_font(&path, &name)
            .map_err(|e| Error::RuntimeError(e.to_string()))?;
        Ok(font_asset.0)
    });

    methods.add_method("get_texture_size", |_, this, id: u32| {
        let size = this.get().get_texture_size(id);
        Ok((size.x, size.y))
    });

    methods.add_method("get_texture_pixel", |_, this, (id, x, y): (u32, u32, u32)| {
        match this.get().get_texture_pixel(id, x, y) {
            Some(c) => Ok(Some(vec![c.x, c.y, c.z, c.w])),
            None => Ok(None),
        }
    });
}