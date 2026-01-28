// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use mlua::{UserData, UserDataMethods};
use glam::{Vec2, Vec4};
use moonwalk::{TextAlign, FontAsset};
use moonwalk::rendering::container::RenderContainer;
use moonwalk::objects::ObjectId;

use super::MoonWalkLuaWrapper;

pub struct LuaRenderContainer(pub RenderContainer);

impl UserData for LuaRenderContainer {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("new_rect", |_, this, ()| {
            Ok(this.0.new_rect().0)
        });

        methods.add_method_mut("new_text", |_, this, (content, font_id, size): (String, u64, f32)| {
            let id = this.0.new_text(&content, FontAsset(font_id), size);
            Ok(id.0)
        });

        methods.add_method_mut("remove", |_, this, id: usize| {
            this.0.remove(ObjectId(id));
            Ok(())
        });

        methods.add_method_mut("set_position", |_, this, (id, x, y): (usize, f32, f32)| {
            this.0.set_position(ObjectId(id), Vec2::new(x, y));
            Ok(())
        });

        methods.add_method_mut("set_size", |_, this, (id, w, h): (usize, f32, f32)| {
            this.0.set_size(ObjectId(id), Vec2::new(w, h));
            Ok(())
        });

        methods.add_method_mut("set_rotation", |_, this, (id, rad): (usize, f32)| {
            this.0.set_rotation(ObjectId(id), rad);
            Ok(())
        });

        methods.add_method_mut("set_color", |_, this, (id, r, g, b, a): (usize, f32, f32, f32, f32)| {
            this.0.set_color(ObjectId(id), Vec4::new(r, g, b, a));
            Ok(())
        });

        methods.add_method_mut("set_color2", |_, this, (id, r, g, b, a): (usize, f32, f32, f32, f32)| {
            this.0.set_color2(ObjectId(id), Vec4::new(r, g, b, a));
            Ok(())
        });

        methods.add_method_mut("set_z_index", |_, this, (id, z): (usize, f32)| {
            this.0.set_z_index(ObjectId(id), z);
            Ok(())
        });

        methods.add_method_mut("set_uv", |_, this, (id, x, y, w, h): (usize, f32, f32, f32, f32)| {
            this.0.set_uv(ObjectId(id), [x, y, w, h]);
            Ok(())
        });

        methods.add_method_mut("set_rounded", |_, this, (id, tl, tr, br, bl): (usize, f32, f32, f32, f32)| {
            this.0.set_rounded(ObjectId(id), Vec4::new(tl, tr, br, bl));
            Ok(())
        });

        methods.add_method_mut("set_texture", |_, this, (id, tex_id): (usize, u32)| {
            this.0.set_texture(ObjectId(id), tex_id);
            Ok(())
        });

        methods.add_method_mut("set_gradient_data", |_, this, (id, p1, p2, p3, p4): (usize, f32, f32, f32, f32)| {
            this.0.set_gradient_data(ObjectId(id), [p1, p2, p3, p4]);
            Ok(())
        });

        methods.add_method_mut("set_effect", |_, this, (id, width, shadow): (usize, f32, f32)| {
            this.0.set_effect(ObjectId(id), width, shadow);
            Ok(())
        });

        methods.add_method_mut("set_text", |_, this, (id, content): (usize, String)| {
            this.0.set_text(ObjectId(id), &content);
            Ok(())
        });

        methods.add_method_mut("set_font_size", |_, this, (id, size): (usize, f32)| {
            this.0.set_font_size(ObjectId(id), size);
            Ok(())
        });

        methods.add_method_mut("set_text_size", |_, this, (id, w, h): (usize, f32, f32)| {
            this.0.set_text_size(ObjectId(id), w, h);
            Ok(())
        });

        methods.add_method_mut("set_text_align", |_, this, (id, align_str): (usize, String)| {
            let align = match align_str.as_str() {
                "center" => TextAlign::Center,
                "right" => TextAlign::Right,
                "justified" => TextAlign::Justified,
                _ => TextAlign::Left,
            };
            this.0.set_text_align(ObjectId(id), align);
            Ok(())
        });

        methods.add_method_mut("measure_text", |_, this, (mw, text, font_id, size, max_w): (mlua::UserDataRefMut<MoonWalkLuaWrapper>, String, u64, f32, f32)| {
            let size = this.0.measure_text(mw.get_mut(), &text, FontAsset(font_id), size, max_w);
            Ok((size.x, size.y))
        });

        methods.add_method_mut("draw", |_, this, (mw, r, g, b, a): (mlua::UserDataRefMut<MoonWalkLuaWrapper>, f32, f32, f32, f32)| {
            this.0.draw(mw.get_mut(), Some(Vec4::new(r, g, b, a)));
            Ok(())
        });

        methods.add_method_mut("snapshot", |_, this, (mw, x, y, w, h): (mlua::UserDataRefMut<MoonWalkLuaWrapper>, u32, u32, u32, u32)| {
            let id = this.0.snapshot(mw.get_mut(), x, y, w, h);
            Ok(id)
        });

        methods.add_method_mut("update_snapshot", |_, this, (mw, x, y, w, h, id): (mlua::UserDataRefMut<MoonWalkLuaWrapper>, u32, u32, u32, u32, u32)| {
            this.0.update_snapshot(mw.get_mut(), x, y, w, h, id);
            Ok(())
        });

        methods.add_method("get_position", |_, this, id: usize| {
            let v = this.0.get_position(ObjectId(id));
            Ok((v.x, v.y))
        });

        methods.add_method("get_size", |_, this, id: usize| {
            let v = this.0.get_size(ObjectId(id));
            Ok((v.x, v.y))
        });

        methods.add_method("get_color", |_, this, id: usize| {
            let v = this.0.get_color(ObjectId(id));
            Ok((v.x, v.y, v.z, v.w))
        });
        
        methods.add_method("get_color2", |_, this, id: usize| {
            let v = this.0.get_color2(ObjectId(id));
            Ok((v.x, v.y, v.z, v.w))
        });

        methods.add_method("get_rotation", |_, this, id: usize| {
            Ok(this.0.get_rotation(ObjectId(id)))
        });

        methods.add_method("get_z_index", |_, this, id: usize| {
            Ok(this.0.get_z_index(ObjectId(id)))
        });

        methods.add_method("get_rounded", |_, this, id: usize| {
            let v = this.0.get_rounded(ObjectId(id));
            Ok((v.x, v.y, v.z, v.w))
        });

        methods.add_method("get_text", |_, this, id: usize| {
            Ok(this.0.get_text(ObjectId(id)))
        });
        
        methods.add_method("get_font_size", |_, this, id: usize| {
            Ok(this.0.get_font_size(ObjectId(id)))
        });

        methods.add_method("get_text_size", |_, this, id: usize| {
            let v = this.0.get_text_size(ObjectId(id));
            Ok((v.x, v.y))
        });

        methods.add_method("get_text_align", |_, this, id: usize| {
            let align = match this.0.get_text_align(ObjectId(id)) {
                TextAlign::Left => "left",
                TextAlign::Center => "center",
                TextAlign::Right => "right",
                TextAlign::Justified => "justified",
            };
            
            Ok(align)
        });
    }
}

pub fn register<'lua, M: UserDataMethods<'lua, MoonWalkLuaWrapper>>(methods: &mut M) {
    methods.add_method("new_render_container", |_, this, (w, h): (u32, u32)| {
        let container = this.get().new_render_container(w, h);
        Ok(LuaRenderContainer(container))
    });
}