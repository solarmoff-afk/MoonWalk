// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use mlua::{UserData, UserDataMethods};
use glam::{Vec2, Vec4};
use moonwalk::public::{Brush, BlendMode}; 

use super::MoonWalkLuaWrapper;

#[derive(Clone)]
pub struct LuaBrush(pub Brush);

impl UserData for LuaBrush {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("set_size", |_, this, v: f32| {
            this.0.size = v; Ok(())
        });
        
        methods.add_method_mut("set_color", |_, this, (r,g,b,a): (f32,f32,f32,f32)| {
            this.0.color = Vec4::new(r,g,b,a); Ok(())
        });
        
        methods.add_method_mut("set_hardness", |_, this, v: f32| {
            this.0.hardness = v; Ok(())
        });

        methods.add_method_mut("set_opacity", |_, this, v: f32| {
            this.0.opacity = v; Ok(())
        });

        methods.add_method_mut("set_spacing", |_, this, v: f32| {
            this.0.spacing = v; Ok(())
        });

        methods.add_method_mut("set_texture", |_, this, id: u32| {
            this.0.texture_id = id; Ok(())
        });

        methods.add_method_mut("set_angle", |_, this, v: f32| {
            this.0.angle = v; Ok(())
        });

        methods.add_method_mut("set_follow_dir", |_, this, v: bool| {
            this.0.follow_direction = v; Ok(())
        });

        methods.add_method_mut("set_roundness", |_, this, v: f32| {
            this.0.roundness = v; Ok(())
        });

        methods.add_method_mut("set_is_eraser", |_, this, v: bool| {
            this.0.is_eraser = v; Ok(())
        });
        
        methods.add_method_mut("set_jitter_pos", |_, this, v: f32| {
            this.0.jitter_position = v; Ok(())
        });

        methods.add_method_mut("set_jitter_size", |_, this, v: f32| {
            this.0.jitter_size = v; Ok(())
        });

        methods.add_method_mut("set_jitter_angle", |_, this, v: f32| {
            this.0.jitter_angle = v; Ok(())
        });

        methods.add_method_mut("set_jitter_opacity", |_, this, v: f32| {
            this.0.jitter_opacity = v; Ok(())
        });

        methods.add_method_mut("set_blend_mode", |_, this, mode: String| {
            this.0.blend_mode = match mode.as_str() {
                "add" => BlendMode::Add,
                "multiply" => BlendMode::Multiply,
                "screen" => BlendMode::Screen,
                "subtract" => BlendMode::Subtract,
                "eraser" => BlendMode::Eraser,
                _ => BlendMode::Normal,
            };
            
            Ok(())
        });
    }
}

pub fn register<'lua, M: UserDataMethods<'lua, MoonWalkLuaWrapper>>(methods: &mut M) {
    methods.add_method("new_brush", |_, this, ()| {
        Ok(LuaBrush(this.get().new_brush()))
    });

    methods.add_method_mut("draw_stroke", |_, this, (tid, brush, x1, y1, x2, y2): (u32, mlua::UserDataRef<LuaBrush>, f32, f32, f32, f32)| {
        this.get_mut().draw_stroke(tid, &brush.0, Vec2::new(x1, y1), Vec2::new(x2, y2));
        Ok(())
    });
    
    methods.add_method_mut("draw_stamp", |_, this, (tid, brush, x, y): (u32, mlua::UserDataRef<LuaBrush>, f32, f32)| {
        this.get_mut().draw_stamp(tid, &brush.0, Vec2::new(x, y));
        Ok(())
    });
}
