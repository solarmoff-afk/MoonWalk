// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use mlua::{UserData, UserDataMethods, Error};
use glam::Vec4;
use moonwalk::path::PathBuilder;
use moonwalk::path::{LineCap, LineJoin, FillRule};

use super::MoonWalkLuaWrapper;

pub struct LuaPathBuilder(Option<PathBuilder>);

impl UserData for LuaPathBuilder {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        fn get_b<F, R>(this: &mut LuaPathBuilder, f: F) -> mlua::Result<R>
        where F: FnOnce(&mut PathBuilder) -> R {
            this.0.as_mut().map(f).ok_or_else(|| Error::RuntimeError("PathBuilder consumed".into()))
        }

        methods.add_method_mut("move_to", |_, this, (x, y)| get_b(this, |b| b.move_to(x, y)));
        methods.add_method_mut("line_to", |_, this, (x, y)| get_b(this, |b| b.line_to(x, y)));
        methods.add_method_mut("quad_to", |_, this, (cx, cy, x, y)| get_b(this, |b| b.quadratic_bezier_to(cx, cy, x, y)));
        methods.add_method_mut("cubic_to", |_, this, (cx1, cy1, cx2, cy2, x, y)| get_b(this, |b| b.cubic_bezier_to(cx1, cy1, cx2, cy2, x, y)));
        methods.add_method_mut("close", |_, this, ()| get_b(this, |b| b.close()));
        
        methods.add_method_mut("set_color", |_, this, (r, g, b, a)| {
            get_b(this, |builder| builder.set_color(Vec4::new(r, g, b, a)))
        });

        methods.add_method_mut("set_stroke", |_, this, w| get_b(this, |b| b.set_stroke(w)));
        
        methods.add_method_mut("set_line_cap", |_, this, s: String| {
            let cap = match s.as_str() { 
                "round" => LineCap::Round,
                "square" => LineCap::Square,
                _ => LineCap::Butt
            };

            get_b(this, |b| b.set_line_cap(cap))
        });
        
        methods.add_method_mut("set_line_join", |_, this, s: String| {
            let join = match s.as_str() {
                "round" => LineJoin::Round,
                "bevel" => LineJoin::Bevel,
                _ => LineJoin::Miter
            };
            
            get_b(this, |b| b.set_line_join(join))
        });

        methods.add_method_mut("set_fill_rule", |_, this, s: String| {
            let rule = match s.as_str() {
                "even_odd" => FillRule::EvenOdd,
                _ => FillRule::NonZero
            };

            get_b(this, |b| b.set_fill_rule(rule))
        });
        
        methods.add_method_mut("set_tolerance", |_, this, t: f32| get_b(this, |b| b.set_tolerance(t)));
    }
}

pub fn register<'lua, M: UserDataMethods<'lua, MoonWalkLuaWrapper>>(methods: &mut M) {
    methods.add_method("new_path_builder", |_, this, ()| {
        Ok(LuaPathBuilder(Some(this.get().new_path_builder())))
    });

    methods.add_method_mut("parse_svg", |_, this, (mut builder, data): (mlua::UserDataRefMut<LuaPathBuilder>, String)| {
         if let Some(b) = &mut builder.0 {
             this.get().parse_svg_path(b, &data).map_err(|e| Error::RuntimeError(e))?;
             Ok(())
         } else {
             Err(Error::RuntimeError("Builder consumed".into()))
         }
    });

    methods.add_method_mut("tessellate", |_, this, (mut builder, w, h): (mlua::UserDataRefMut<LuaPathBuilder>, u32, u32)| {
        let inner = builder.0.take().ok_or(Error::RuntimeError("Consumed".into()))?;
        Ok(inner.tessellate(this.get_mut(), w, h))
    });
    
    methods.add_method_mut("tessellate_to", |_, this, (mut builder, tid, w, h): (mlua::UserDataRefMut<LuaPathBuilder>, u32, u32, u32)| {
        let inner = builder.0.take().ok_or(Error::RuntimeError("Consumed".into()))?;
        inner.tessellate_to(this.get_mut(), tid, w, h);
        Ok(())
    });
}