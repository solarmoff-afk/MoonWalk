// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use mlua::UserDataMethods;
use glam::{Vec2, Vec4};
use moonwalk::objects::ObjectId;
use moonwalk::TextAlign;

use super::MoonWalkLuaWrapper;

pub fn register<'lua, M: UserDataMethods<'lua, MoonWalkLuaWrapper>>(methods: &mut M) {
    methods.add_method_mut("new_rect", |_, this, ()| {
        Ok(this.get_mut().new_rect().0)
    });

    methods.add_method_mut("new_text", |_, this, (content, font_id, size): (String, u64, f32)| {
        let id = this.get_mut().new_text(&content, moonwalk::FontAsset(font_id), size);
        Ok(id.0)
    });

    methods.add_method_mut("remove", |_, this, id: usize| {
        this.get_mut().remove(ObjectId(id));
        Ok(())
    });

    methods.add_method_mut("remove_all", |_, this, ()| {
        this.get_mut().remove_all();
        Ok(())
    });

    methods.add_method_mut("set_position", |_, this, (id, x, y): (usize, f32, f32)| {
        this.get_mut().set_position(ObjectId(id), Vec2::new(x, y));
        Ok(())
    });

    methods.add_method_mut("set_size", |_, this, (id, w, h): (usize, f32, f32)| {
        this.get_mut().set_size(ObjectId(id), Vec2::new(w, h));
        Ok(())
    });

    methods.add_method_mut("set_rotation", |_, this, (id, rad): (usize, f32)| {
        this.get_mut().set_rotation(ObjectId(id), rad);
        Ok(())
    });

    methods.add_method_mut("set_color", |_, this, (id, r, g, b, a): (usize, f32, f32, f32, f32)| {
        this.get_mut().set_color(ObjectId(id), Vec4::new(r, g, b, a));
        Ok(())
    });

    methods.add_method_mut("set_color2", |_, this, (id, r, g, b, a): (usize, f32, f32, f32, f32)| {
        this.get_mut().set_color2(ObjectId(id), Vec4::new(r, g, b, a));
        Ok(())
    });

    methods.add_method_mut("linear_gradient", |_, this, (id, dx, dy): (usize, f32, f32)| {
        this.get_mut().linear_gradient(ObjectId(id), Vec2::new(dx, dy));
        Ok(())
    });

    methods.add_method_mut("radial_gradient", |_, this, (id, cx, cy, rx, ry): (usize, f32, f32, f32, f32)| {
        this.get_mut().radial_gradient(ObjectId(id), Vec2::new(cx, cy), Vec2::new(rx, ry));
        Ok(())
    });

    methods.add_method_mut("reset_gradient", |_, this, id: usize| {
        this.get_mut().reset_gradient(ObjectId(id));
        Ok(())
    });

    methods.add_method_mut("set_rounded", |_, this, (id, tl, tr, br, bl): (usize, f32, f32, f32, f32)| {
        this.get_mut().set_rounded(ObjectId(id), Vec4::new(tl, tr, br, bl));
        Ok(())
    });

    methods.add_method_mut("set_z_index", |_, this, (id, z): (usize, f32)| {
        this.get_mut().set_z_index(ObjectId(id), z);
        Ok(())
    });

    methods.add_method_mut("set_texture", |_, this, (id, tex_id): (usize, u32)| {
        this.get_mut().set_texture(ObjectId(id), tex_id);
        Ok(())
    });

    methods.add_method_mut("set_uv", |_, this, (id, x, y, w, h): (usize, f32, f32, f32, f32)| {
        this.get_mut().set_uv(ObjectId(id), [x, y, w, h]);
        Ok(())
    });

    methods.add_method_mut("set_effect", |_, this, (id, width, shadow): (usize, f32, f32)| {
        this.get_mut().set_effect(ObjectId(id), width, shadow);
        Ok(())
    });

    methods.add_method_mut("set_text", |_, this, (id, content): (usize, String)| {
        this.get_mut().set_text(ObjectId(id), &content);
        Ok(())
    });

    methods.add_method_mut("set_font_size", |_, this, (id, size): (usize, f32)| {
        this.get_mut().set_font_size(ObjectId(id), size);
        Ok(())
    });

    methods.add_method_mut("set_text_size", |_, this, (id, w, h): (usize, f32, f32)| {
        this.get_mut().set_text_size(ObjectId(id), w, h);
        Ok(())
    });

    methods.add_method_mut("set_text_align", |_, this, (id, align_str): (usize, String)| {
        let align = match align_str.as_str() {
            "center" => TextAlign::Center,
            "right" => TextAlign::Right,
            "justified" => TextAlign::Justified,
            _ => TextAlign::Left,
        };

        this.get_mut().set_text_align(ObjectId(id), align);
        Ok(())
    });

    methods.add_method_mut("measure_text", |_, this, (text, font_id, size, max_w): (String, u64, f32, f32)| {
        let size = this.get_mut().measure_text(&text, moonwalk::FontAsset(font_id), size, max_w);
        Ok((size.x, size.y))
    });

    methods.add_method_mut("set_hit_group", |_, this, (id, group): (usize, u16)| {
        this.get_mut().set_hit_group(ObjectId(id), group);
        Ok(())
    });

    methods.add_method("resolve_hit", |_, this, (x, y, w, h, group): (f32, f32, f32, f32, u16)| {
        let res = this.get().resolve_hit(Vec2::new(x, y), Vec2::new(w, h), group);
        Ok(res.map(|id| id.0))
    });

    methods.add_method("get_position", |_, this, id: usize| {
        let v = this.get().get_position(ObjectId(id));
        Ok((v.x, v.y))
    });

    methods.add_method("get_size", |_, this, id: usize| {
        let v = this.get().get_size(ObjectId(id));
        Ok((v.x, v.y))
    });

    methods.add_method("get_rotation", |_, this, id: usize| {
        Ok(this.get().get_rotation(ObjectId(id)))
    });

    methods.add_method("get_color", |_, this, id: usize| {
        let v = this.get().get_color(ObjectId(id));
        Ok((v.x, v.y, v.z, v.w))
    });

    methods.add_method("get_color2", |_, this, id: usize| {
        let v = this.get().get_color2(ObjectId(id));
        Ok((v.x, v.y, v.z, v.w))
    });

    methods.add_method("get_z_index", |_, this, id: usize| {
        Ok(this.get().get_z_index(ObjectId(id)))
    });

    methods.add_method("get_hit_group", |_, this, id: usize| {
        Ok(this.get().get_hit_group(ObjectId(id)))
    });

    methods.add_method("get_rounded", |_, this, id: usize| {
        let v = this.get().get_rounded(ObjectId(id));
        Ok((v.x, v.y, v.z, v.w))
    });

    methods.add_method("get_text", |_, this, id: usize| {
        Ok(this.get().get_text(ObjectId(id)).to_string())
    });

    methods.add_method("get_font_size", |_, this, id: usize| {
        Ok(this.get().get_font_size(ObjectId(id)))
    });

    methods.add_method("get_text_size", |_, this, id: usize| {
        let v = this.get().get_text_size(ObjectId(id));
        Ok((v.x, v.y))
    });

    methods.add_method("get_text_align", |_, this, id: usize| {
        let align = match this.get().get_text_align(ObjectId(id)) {
            TextAlign::Left => "left",
            TextAlign::Center => "center",
            TextAlign::Right => "right",
            TextAlign::Justified => "justified",
        };
        
        Ok(align)
    });

    methods.add_method("is_alive", |_, this, id: usize| {
        Ok(this.get().is_alive(ObjectId(id)))
    });
}