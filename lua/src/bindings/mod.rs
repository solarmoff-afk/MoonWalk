// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

pub mod objects;
pub mod resources;
pub mod filters;
pub mod brush;
pub mod path;
pub mod container;
pub mod lunar;

use mlua::UserData;
use moonwalk::MoonWalk;

#[derive(Clone, Copy)]
pub struct MoonWalkLuaWrapper(pub *mut MoonWalk);

impl MoonWalkLuaWrapper {
    fn get_mut(&self) -> &mut MoonWalk {
        unsafe { &mut *self.0 }
    }
    
    fn get(&self) -> &MoonWalk {
        unsafe { &*self.0 }
    }
}

impl UserData for MoonWalkLuaWrapper {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        objects::register(methods);
        resources::register(methods);
        filters::register(methods);
        brush::register(methods);
        path::register(methods);
        container::register(methods);
        lunar::register(methods);

        methods.add_method_mut("set_viewport", |_, this, (w, h): (u32, u32)| {
            this.get_mut().set_viewport(w, h);
            Ok(())
        });

        methods.add_method_mut("set_scale_factor", |_, this, scale: f32| {
            this.get_mut().set_scale_factor(scale);
            Ok(())
        });
        
        methods.add_method("get_window_size", |_, this, ()| {
            let size = this.get().get_window_size();
            Ok((size.x, size.y))
        });

        methods.add_method("get_scale_factor", |_, this, ()| {
            Ok(this.get().get_scale_factor())
        });

        methods.add_method_mut("render_frame", |_, this, (r, g, b, a): (f32, f32, f32, f32)| {
            let _ = this.get_mut().render_frame(glam::Vec4::new(r, g, b, a));
            Ok(())
        });

        methods.add_method_mut("set_vsync", |_, this, vsync: bool| {
            this.get_mut().set_vsync(vsync);
            Ok(())
        });

        methods.add_method_mut("snapshot", |_, this, (x, y, w, h): (f32, f32, f32, f32)| {
            let id = this.get_mut().snapshot(glam::Vec2::new(x, y), glam::Vec2::new(w, h));
            Ok(id)
        });

        methods.add_method_mut("update_snapshot", |_, this, (x, y, w, h, id): (f32, f32, f32, f32, u32)| {
            this.get_mut().update_snapshot(glam::Vec2::new(x, y), glam::Vec2::new(w, h), id);
            Ok(())
        });

        methods.add_method_mut("save_texture", |_, this, (id, path): (u32, String)| {
            this.get_mut().save_texture(id, &path).map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
            Ok(())
        });
    }
}