pub mod error;
pub mod font;
mod ffi_utils;
mod objects;
mod renderer;
mod rendering;

use glam::{Vec2, Vec4};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use wgpu::SurfaceError;

use crate::font::{FontId, FontSystem};
use crate::objects::{ShaderId, UniformValue};
use crate::renderer::Renderer;
use crate::error::MoonWalkError;

pub use objects::ObjectId;

pub struct MoonWalk {
    renderer: Renderer<'static>,
    font_system: FontSystem,
}

impl MoonWalk {
    pub fn new(
        window: &'static (impl HasWindowHandle + HasDisplayHandle + Send + Sync),
    ) -> Result<Self, error::MoonWalkError> {
        let renderer = pollster::block_on(Renderer::new(window))?;
        Ok(Self {
            renderer,
            font_system: FontSystem::new(),
        })
    }

    #[cfg(target_os = "android")]
    pub fn new_android(
        window: &ndk::native_window::NativeWindow,
        asset_manager: ndk::asset::AssetManager,
    ) -> Result<Self, error::MoonWalkError> {
        use raw_window_handle::{DisplayHandle, HasDisplayHandle, HasWindowHandle, WindowHandle};

        struct AndroidWindowWrapper {
            handle: raw_window_handle::RawWindowHandle,
        }

        unsafe impl HasWindowHandle for AndroidWindowWrapper {
            fn window_handle(&self) -> Result<WindowHandle<'_>, raw_window_handle::HandleError> {
                Ok(unsafe { WindowHandle::borrow_raw(self.handle) })
            }
        }

        unsafe impl HasDisplayHandle for AndroidWindowWrapper {
            fn display_handle(&self) -> Result<DisplayHandle<'_>, raw_window_handle::HandleError> {
                Ok(unsafe { DisplayHandle::borrow_raw(raw_window_handle::RawDisplayHandle::Android(
                    raw_window_handle::AndroidDisplayHandle::new()
                ))})
            }
        }

        let wrapper = Box::new(AndroidWindowWrapper { handle: window.raw_window_handle() });
        let static_wrapper: &'static AndroidWindowWrapper = Box::leak(wrapper);
        let renderer = pollster::block_on(Renderer::new(static_wrapper))?;
        
        Ok(Self {
            renderer,
            font_system: FontSystem::new(asset_manager),
        })
    }

    pub fn set_viewport(&mut self, width: u32, height: u32) {
        self.renderer.set_viewport(width, height);
    }

    pub fn render_frame(&mut self, clear_color: Vec4) -> Result<(), SurfaceError> {
        self.renderer
            .render_frame(&mut self.font_system, clear_color)
    }

    pub fn new_rect(&mut self) -> ObjectId {
        self.renderer.new_rect()
    }

    pub fn new_text(&mut self) -> ObjectId {
        self.renderer.new_text(self.font_system.cosmic_mut())
    }

    pub fn config_position(&mut self, id: ObjectId, pos: Vec2) {
        self.renderer.config_position(id, pos);
    }

    pub fn config_size(&mut self, id: ObjectId, size: Vec2) {
        self.renderer
            .config_size(id, size, self.font_system.cosmic_mut());
    }

    pub fn config_rotation(&mut self, id: ObjectId, angle_degrees: f32) {
        self.renderer.config_rotation(id, angle_degrees);
    }

    pub fn config_color(&mut self, id: ObjectId, color: Vec4) {
        self.renderer.config_color(id, color);
    }

    pub fn config_z_index(&mut self, id: ObjectId, z: f32) {
        self.renderer.config_z_index(id, z);
    }

    pub fn config_text(&mut self, id: ObjectId, text: &str) {
        self.renderer
            .config_text(id, text, &mut self.font_system);
    }

    pub fn load_font(&mut self, path: &str, size: f32) -> Result<FontId, MoonWalkError> {
        self.font_system.load_font(path, size)
    }

    pub fn load_font_from_bytes(&mut self, data: &[u8], name: &str, size: f32) -> Result<FontId, MoonWalkError> {
        self.font_system.load_font_from_bytes(data, name, size)
    }

    pub fn clear_font(&mut self, font_id: FontId) {
        self.font_system.clear_font(font_id);
    }

    pub fn config_font(&mut self, object_id: ObjectId, font_id: FontId) {
        self.renderer.config_font(object_id, font_id);
    }

    pub fn set_rounded(&mut self, object_id: ObjectId, radii: Vec4) {
        self.renderer.config_rounded(object_id, radii);
    }

    pub fn delete_object(&mut self, id: ObjectId) {
        self.renderer.delete_object(id);
    }

    pub fn clear_all_objects(&mut self) {
        self.renderer.clear_all_objects();
    }

    pub fn compile_shader(&mut self, shader_src: &str) -> Result<ShaderId, MoonWalkError> {
        self.renderer.compile_shader(shader_src)
    }

    pub fn set_object_shader(&mut self, object_id: ObjectId, shader_id: ShaderId) {
        self.renderer.set_object_shader(object_id, shader_id);
    }

    pub fn set_uniform(&mut self, id: ObjectId, name: String, value: UniformValue) {
        self.renderer.set_uniform(id, name, value);
    }

    pub fn new_bezier(&mut self) -> ObjectId {
        self.renderer.new_bezier()
    }

    pub fn set_bezier_points(&mut self, id: ObjectId, points: Vec<Vec2>) {
        self.renderer.set_bezier_points(id, points);
    }

    pub fn config_bezier_thickness(&mut self, id: ObjectId, thickness: f32) {
        self.renderer.config_bezier_thickness(id, thickness);
    }

    pub fn config_bezier_smooth(&mut self, id: ObjectId, smoothing: f32) {
        self.renderer.config_bezier_smooth(id, smoothing);
    }

    pub fn set_parent(&mut self, child: ObjectId, parent: ObjectId) {
        self.renderer.set_parent(child, parent);
    }

    pub fn set_masking(&mut self, id: ObjectId, enable: bool) {
        self.renderer.set_masking(id, enable);
    }
}

pub mod ffi {
    use super::{
        ffi_utils::{mat4_from_ptr, string_from_ptr},
        font::FontId,
        objects::UniformValue,
        MoonWalk,
    };

    use  glam::{Vec2, Vec3, Vec4};
    use raw_window_handle::{DisplayHandle, HasDisplayHandle, HasWindowHandle, WindowHandle};

    #[derive(Clone, Copy)]
    struct WindowHandleWrapper {
        window_handle: raw_window_handle::RawWindowHandle,
        display_handle: raw_window_handle::RawDisplayHandle,
    }
    unsafe impl Send for WindowHandleWrapper {}
    unsafe impl Sync for WindowHandleWrapper {}
    impl HasWindowHandle for WindowHandleWrapper {
        fn window_handle(&self) -> Result<WindowHandle<'_>, raw_window_handle::HandleError> {
            Ok(unsafe { WindowHandle::borrow_raw(self.window_handle) })
        }
    }
    impl HasDisplayHandle for WindowHandleWrapper {
        fn display_handle(&self) -> Result<DisplayHandle<'_>, raw_window_handle::HandleError> {
            Ok(unsafe { DisplayHandle::borrow_raw(self.display_handle) })
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn moonwalk_init(
        window_handle: *const raw_window_handle::RawWindowHandle,
        display_handle: *const raw_window_handle::RawDisplayHandle,
    ) -> *mut MoonWalk {
        if window_handle.is_null() || display_handle.is_null() {
            eprintln!("MoonWalk initialization failed: null handle provided");
            return std::ptr::null_mut();
        }
        
        let window_handle = *window_handle;
        let display_handle = *display_handle;
        
        let handle_wrapper = Box::new(WindowHandleWrapper {
            window_handle,
            display_handle,
        });
        let static_handle_wrapper: &'static WindowHandleWrapper = Box::leak(handle_wrapper);

        match MoonWalk::new(static_handle_wrapper) {
            Ok(state) => Box::into_raw(Box::new(state)),
            Err(e) => {
                eprintln!("MoonWalk initialization failed: {}", e);
                std::ptr::null_mut()
            }
        }
    }

    #[no_mangle]
    #[cfg(target_os = "android")]
    pub unsafe extern "C" fn moonwalk_init_android(
        window_ptr: *mut ndk_sys::ANativeWindow,
        asset_manager_ptr: *mut ndk_sys::AAssetManager,
    ) -> *mut MoonWalk {
        android_logger::init_once(
            android_logger::Config::default()
                .with_min_level(log::Level::Info)
                .with_tag("MoonWalk"),
        );
        
        if window_ptr.is_null() || asset_manager_ptr.is_null() {
            log::error!("MoonWalk Android initialization failed: null handle provided");
            return std::ptr::null_mut();
        }
        
        let window = ndk::native_window::NativeWindow::from_ptr(std::ptr::NonNull::new_unchecked(window_ptr));
        let asset_manager = ndk::asset::AssetManager::from_ptr(std::ptr::NonNull::new_unchecked(asset_manager_ptr));
        
        match MoonWalk::new_android(&window, asset_manager) {
            Ok(state) => Box::into_raw(Box::new(state)),
            Err(e) => {
                log::error!("MoonWalk Android initialization failed: {}", e);
                std::ptr::null_mut()
            }
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn moonwalk_shutdown(state_ptr: *mut MoonWalk) {
        if !state_ptr.is_null() {
            drop(Box::from_raw(state_ptr));
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn moonwalk_set_viewport(
        state_ptr: *mut MoonWalk,
        width: u32,
        height: u32,
    ) {
        if let Some(state) = state_ptr.as_mut() {
            state.set_viewport(width, height);
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn moonwalk_render_frame(
        state_ptr: *mut MoonWalk,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) {
        if let Some(state) = state_ptr.as_mut() {
            if let Err(e) = state.render_frame(Vec4::new(r, g, b, a)) {
                eprintln!("MoonWalk render failed: {}", e);
            }
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn moonwalk_new_rect(state_ptr: *mut MoonWalk) -> u32 {
        state_ptr.as_mut().map_or(0, |s| s.new_rect().to_u32())
    }

    #[no_mangle]
    pub unsafe extern "C" fn moonwalk_new_text(state_ptr: *mut MoonWalk) -> u32 {
        state_ptr.as_mut().map_or(0, |s| s.new_text().to_u32())
    }

    #[no_mangle]
    pub unsafe extern "C" fn moonwalk_config_position(
        state_ptr: *mut MoonWalk,
        id: u32,
        x: f32,
        y: f32,
    ) {
        if let Some(state) = state_ptr.as_mut() {
            state.config_position(id.into(), Vec2::new(x, y));
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn moonwalk_config_size(
        state_ptr: *mut MoonWalk,
        id: u32,
        width: f32,
        height: f32,
    ) {
        if let Some(state) = state_ptr.as_mut() {
            state.config_size(id.into(), Vec2::new(width, height));
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn moonwalk_config_rotation(
        state_ptr: *mut MoonWalk,
        id: u32,
        angle_degrees: f32,
    ) {
        if let Some(state) = state_ptr.as_mut() {
            state.config_rotation(id.into(), angle_degrees);
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn moonwalk_config_color(
        state_ptr: *mut MoonWalk,
        id: u32,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) {
        if let Some(state) = state_ptr.as_mut() {
            state.config_color(id.into(), Vec4::new(r, g, b, a));
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn moonwalk_config_z_index(state_ptr: *mut MoonWalk, id: u32, z: f32) {
        if let Some(state) = state_ptr.as_mut() {
            state.config_z_index(id.into(), z);
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn moonwalk_config_text(
        state_ptr: *mut MoonWalk,
        id: u32,
        text_ptr: *const libc::c_char,
    ) {
        if let Some(state) = state_ptr.as_mut() {
            if let Ok(text) = string_from_ptr(text_ptr) {
                state.config_text(id.into(), &text);
            }
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn moonwalk_load_font(
        state_ptr: *mut MoonWalk,
        path_ptr: *const libc::c_char,
        size: f32,
    ) -> u64 {
        if let Some(state) = state_ptr.as_mut() {
            if let Ok(path) = string_from_ptr(path_ptr) {
                match state.load_font(&path, size) {
                    Ok(font_id) => font_id.to_u64(),
                    Err(e) => {
                        eprintln!("Failed to load font from {}: {}", path, e);
                        0
                    }
                }
            } else {
                0
            }
        } else {
            0
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn moonwalk_clear_font(state_ptr: *mut MoonWalk, font_id: u64) {
        if let Some(state) = state_ptr.as_mut() {
            state.clear_font(FontId::from_u64(font_id));
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn moonwalk_config_font(
        state_ptr: *mut MoonWalk,
        object_id: u32,
        font_id: u64,
    ) {
        if let Some(state) = state_ptr.as_mut() {
            state.config_font(object_id.into(), FontId::from_u64(font_id));
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn moonwalk_set_rounded(
        state_ptr: *mut MoonWalk,
        object_id: u32,
        tl: f32,
        tr: f32,
        br: f32,
        bl: f32,
    ) {
        if let Some(state) = state_ptr.as_mut() {
            state.set_rounded(object_id.into(), Vec4::new(tl, tr, br, bl));
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn moonwalk_delete_object(state_ptr: *mut MoonWalk, id: u32) {
        if let Some(state) = state_ptr.as_mut() {
            state.delete_object(id.into());
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn moonwalk_clear_all(state_ptr: *mut MoonWalk) {
        if let Some(state) = state_ptr.as_mut() {
            state.clear_all_objects();
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn moonwalk_compile_shader(
        state_ptr: *mut MoonWalk,
        vs_src_ptr: *const libc::c_char,
        _fs_src_ptr: *const libc::c_char,
    ) -> u32 {
        if let Some(state) = state_ptr.as_mut() {
            if let Ok(shader_src) = string_from_ptr(vs_src_ptr) {
                match state.compile_shader(&shader_src) {
                    Ok(id) => id.to_u32(),
                    Err(e) => {
                        eprintln!("Shader compilation failed: {}", e);
                        0
                    }
                }
            } else {
                0
            }
        } else {
            0
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn moonwalk_set_object_shader(
        state_ptr: *mut MoonWalk,
        object_id: u32,
        shader_id: u32,
    ) {
        if let Some(state) = state_ptr.as_mut() {
            state.set_object_shader(object_id.into(), shader_id.into());
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn moonwalk_new_bezier(state_ptr: *mut MoonWalk) -> u32 {
        state_ptr.as_mut().map_or(0, |s| s.new_bezier().to_u32())
    }

    #[no_mangle]
    pub unsafe extern "C" fn moonwalk_set_bezier_points(
        state_ptr: *mut MoonWalk,
        id: u32,
        points_ptr: *const f32,
        points_count: usize,
    ) {
        if let Some(state) = state_ptr.as_mut() {
            let points_slice = std::slice::from_raw_parts(points_ptr, points_count * 2);
            let points: Vec<Vec2> = points_slice
                .chunks(2)
                .map(|chunk| Vec2::new(chunk[0], chunk[1]))
                .collect();
            state.set_bezier_points(id.into(), points);
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn moonwalk_config_bezier_thickness(
        state_ptr: *mut MoonWalk,
        id: u32,
        thickness: f32,
    ) {
        if let Some(state) = state_ptr.as_mut() {
            state.config_bezier_thickness(id.into(), thickness);
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn moonwalk_config_bezier_smooth(
        state_ptr: *mut MoonWalk,
        id: u32,
        smooth: f32,
    ) {
        if let Some(state) = state_ptr.as_mut() {
            state.config_bezier_smooth(id.into(), smooth);
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn moonwalk_set_parent(state_ptr: *mut MoonWalk, child: u32, parent: u32) {
        if let Some(s) = state_ptr.as_mut() {
            s.set_parent(child.into(), parent.into());
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn moonwalk_set_masking(state_ptr: *mut MoonWalk, id: u32, enable: bool) {
        if let Some(s) = state_ptr.as_mut() {
            s.set_masking(id.into(), enable);
        }
    }

    macro_rules! define_set_uniform {
        ($func_name:ident, $value_expr:expr, $($param:ident: $ptype:ty),*) => {
            #[no_mangle]
            pub unsafe extern "C" fn $func_name(state_ptr: *mut MoonWalk, id: u32, name: *const libc::c_char, $($param: $ptype),*) {
                if let Some(s) = state_ptr.as_mut() {
                    if let Ok(n) = string_from_ptr(name) {
                        s.set_uniform(id.into(), n, $value_expr);
                    }
                }
            }
        };
    }

    define_set_uniform!(moonwalk_set_uniform_int, UniformValue::Int(val), val: i32);
    define_set_uniform!(moonwalk_set_uniform_float, UniformValue::Float(val), val: f32);
    define_set_uniform!(moonwalk_set_uniform_vec2, UniformValue::Vec2(Vec2::new(x, y)), x: f32, y: f32);
    define_set_uniform!(moonwalk_set_uniform_vec3, UniformValue::Vec3(Vec3::new(x, y, z)), x: f32, y: f32, z: f32);
    define_set_uniform!(moonwalk_set_uniform_vec4, UniformValue::Vec4(Vec4::new(x, y, z, w)), x: f32, y: f32, z: f32, w: f32);
    define_set_uniform!(moonwalk_set_uniform_bool, UniformValue::Bool(val), val: bool);

    #[no_mangle]
    pub unsafe extern "C" fn moonwalk_set_uniform_mat4(
        state_ptr: *mut MoonWalk,
        id: u32,
        name: *const libc::c_char,
        mat_ptr: *const f32,
    ) {
        if let Some(s) = state_ptr.as_mut() {
            if let (Ok(n), Ok(m)) = (string_from_ptr(name), mat4_from_ptr(mat_ptr)) {
                s.set_uniform(id.into(), n, UniformValue::Mat4(m));
            }
        }
    }
}