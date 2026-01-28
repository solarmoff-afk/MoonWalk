// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

mod bindings;

use mlua::{Lua, Function, Table, RegistryKey};
use moonwalk::MoonWalk;
use moonwalk_bootstrap::{Application, TouchPhase};
use glam::{Vec2, Vec4};
use std::rc::Rc;
use std::cell::RefCell;

use crate::bindings::MoonWalkLuaWrapper;

struct Callbacks {
    on_start: Option<RegistryKey>,
    on_update: Option<RegistryKey>,
    on_draw: Option<RegistryKey>,
    on_touch: Option<RegistryKey>,
    on_resize: Option<RegistryKey>,
    on_pre_render: Option<RegistryKey>,
    on_exit: Option<RegistryKey>,
}

pub struct MoonLua {
    lua: Lua,
    callbacks: Rc<RefCell<Callbacks>>,
}

impl MoonLua {
    pub fn new() -> mlua::Result<Self> {
        let lua = Lua::new();
        
        Ok(Self {
            lua,
            callbacks: Rc::new(RefCell::new(Callbacks {
                on_start: None,
                on_update: None,
                on_draw: None,
                on_touch: None,
                on_resize: None,
                on_pre_render: None,
                on_exit: None,
            })),
        })
    }

    pub fn init(&mut self) -> mlua::Result<()> {
        let callbacks = self.callbacks.clone();
        let lua = &self.lua;
        
        let bootstrap = lua.create_table()?;
        
        bootstrap.set("set_on_start", lua.create_function({
            let callbacks = callbacks.clone();
            move |lua, func: Function| {
                callbacks.borrow_mut().on_start = Some(lua.create_registry_value(func)?);
                Ok(())
            }
        })?)?;

        bootstrap.set("set_on_update", lua.create_function({
            let callbacks = callbacks.clone();
            move |lua, func: Function| {
                callbacks.borrow_mut().on_update = Some(lua.create_registry_value(func)?);
                Ok(())
            }
        })?)?;

        bootstrap.set("set_on_draw", lua.create_function({
            let callbacks = callbacks.clone();
            move |lua, func: Function| {
                callbacks.borrow_mut().on_draw = Some(lua.create_registry_value(func)?);
                Ok(())
            }
        })?)?;

        bootstrap.set("set_on_touch", lua.create_function({
            let callbacks = callbacks.clone();
            move |lua, func: Function| {
                callbacks.borrow_mut().on_touch = Some(lua.create_registry_value(func)?);
                Ok(())
            }
        })?)?;

        bootstrap.set("set_on_resize", lua.create_function({
            let callbacks = callbacks.clone();
            move |lua, func: Function| {
                callbacks.borrow_mut().on_resize = Some(lua.create_registry_value(func)?);
                Ok(())
            }
        })?)?;

        bootstrap.set("set_on_pre_render", lua.create_function({
            let callbacks = callbacks.clone();
            move |lua, func: Function| {
                callbacks.borrow_mut().on_pre_render = Some(lua.create_registry_value(func)?);
                Ok(())
            }
        })?)?;

        bootstrap.set("set_on_exit", lua.create_function({
            let callbacks = callbacks.clone();
            move |lua, func: Function| {
                callbacks.borrow_mut().on_exit = Some(lua.create_registry_value(func)?);
                Ok(())
            }
        })?)?;

        lua.globals().set("bootstrap", bootstrap)?;
        
        Ok(())
    }

    pub fn execute(&self, lua_code: &str) -> mlua::Result<()> {
        self.lua.load(lua_code).exec()
    }
}

impl Application for MoonLua {
    fn on_start(&mut self, mw: &mut MoonWalk, viewport: Vec2) {
        let callbacks = self.callbacks.borrow();
        if let Some(ref key) = callbacks.on_start {
            if let Ok(cb) = self.lua.registry_value::<Function>(key) {
                if let Ok(event) = self.lua.create_table() {
                    let _ = event.set("width", viewport.x);
                    let _ = event.set("height", viewport.y);
                    
                    let wrapper = MoonWalkLuaWrapper(mw as *mut MoonWalk);
                    
                    if let Err(e) = cb.call::<_, ()>((wrapper, event)) {
                        eprintln!("[Lua Error (on_start): {}", e);
                    }
                }
            }
        }
    }
    
    fn on_update(&mut self, dt: f32) {
        let callbacks = self.callbacks.borrow();
        if let Some(ref key) = callbacks.on_update {
            if let Ok(cb) = self.lua.registry_value::<Function>(key) {
                if let Err(e) = cb.call::<_, ()>(dt) {
                    eprintln!("Lua Error (on_update): {}", e);
                }
            }
        }
    }
    
    fn on_draw(&mut self, mw: &mut MoonWalk) {
        let callbacks = self.callbacks.borrow();
        if let Some(ref key) = callbacks.on_draw {
            if let Ok(cb) = self.lua.registry_value::<Function>(key) {
                let wrapper = MoonWalkLuaWrapper(mw as *mut MoonWalk);
                if let Err(e) = cb.call::<_, ()>((wrapper,)) {
                    eprintln!("Lua Error (on_draw): {}", e);
                }
            }
        }
    }
    
    fn on_resize(&mut self, mw: &mut MoonWalk, viewport: Vec2) {
        let callbacks = self.callbacks.borrow();
        if let Some(ref key) = callbacks.on_resize {
            if let Ok(cb) = self.lua.registry_value::<Function>(key) {
                if let Ok(event) = self.lua.create_table() {
                    let _ = event.set("width", viewport.x);
                    let _ = event.set("height", viewport.y);
                    
                    let wrapper = MoonWalkLuaWrapper(mw as *mut MoonWalk);
                    if let Err(e) = cb.call::<_, ()>((wrapper, event)) {
                        eprintln!("Lua Error (on_resize): {}", e);
                    }
                }
            }
        }
    }
    
    fn on_pre_render(&mut self) -> Option<Vec4> {
        let callbacks = self.callbacks.borrow();
        if let Some(ref key) = callbacks.on_pre_render {
            if let Ok(cb) = self.lua.registry_value::<Function>(key) {
                if let Ok(event) = self.lua.create_table() {
                    match cb.call::<_, Option<Table>>((event,)) {
                        Ok(result) => {
                            if let Some(table) = result {
                                let r: f32 = table.get("r").unwrap_or(0.0);
                                let g: f32 = table.get("g").unwrap_or(0.0);
                                let b: f32 = table.get("b").unwrap_or(0.0);
                                let a: f32 = table.get("a").unwrap_or(1.0);
                                return Some(Vec4::new(r, g, b, a));
                            }
                        },
                        Err(e) => {
                            eprintln!("Lua Error (on_pre_render): {}", e);
                        }
                    }
                }
            }
        }
        None
    }
    
    fn on_touch(&mut self, mw: &mut MoonWalk, phase: TouchPhase, position: Vec2) {
        let callbacks = self.callbacks.borrow();
        if let Some(ref key) = callbacks.on_touch {
            if let Ok(cb) = self.lua.registry_value::<Function>(key) {
                if let Ok(event) = self.lua.create_table() {
                    let _ = event.set("x", position.x);
                    let _ = event.set("y", position.y);
                    
                    let phase_str = match phase {
                        TouchPhase::Started => "started",
                        TouchPhase::Moved => "moved",
                        TouchPhase::Ended => "ended",
                        TouchPhase::Cancelled => "cancelled",
                    };
                    let _ = event.set("phase", phase_str);
                    
                    let wrapper = MoonWalkLuaWrapper(mw as *mut MoonWalk);
                    if let Err(e) = cb.call::<_, ()>((wrapper, event)) {
                        eprintln!("Lua Error (on_touch): {}", e);
                    }
                }
            }
        }
    }
    
    fn on_exit(&mut self) {
        let callbacks = self.callbacks.borrow();
        if let Some(ref key) = callbacks.on_exit {
            if let Ok(cb) = self.lua.registry_value::<Function>(key) {
                if let Err(e) = cb.call::<_, ()>(()) {
                    eprintln!("Lua Error (on_exit): {}", e);
                }
            }
        }
    }
}
