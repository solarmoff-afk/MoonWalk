use glam::Mat4;
use std::ffi::CStr;

pub unsafe fn string_from_ptr(ptr: *const libc::c_char) -> Result<String, ()> {
    if ptr.is_null() {
        return Err(());
    }
    
    CStr::from_ptr(ptr)
        .to_str()
        .map(|s| s.to_string())
        .map_err(|_| ())
}

pub unsafe fn mat4_from_ptr(ptr: *const f32) -> Result<Mat4, ()> {
    if ptr.is_null() {
        Err(())
    } else {
        let slice = std::slice::from_raw_parts(ptr, 16);
        Ok(Mat4::from_cols_slice(slice))
    }
}