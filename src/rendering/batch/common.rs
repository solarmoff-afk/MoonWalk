use crate::objects::{ObjectStore, Object, ShaderId};
use easy_gpu::Buffer;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum RenderLayer {
    Simple,
    Glyph,
}

pub struct BatchGroup {
    pub shader_id: ShaderId,
    pub vbo: Option<Buffer<u8>>, 
    pub vertex_count: usize,
    pub scissor: Option<[u32; 4]>,
    pub sort_key: f32,
    pub bind_group_uniforms: Option<wgpu::BindGroup>,
}

pub fn calculate_scissor(store: &ObjectStore, start_obj: &Object, screen_w: u32, screen_h: u32) -> Option<[u32; 4]> {
    let mut min_x = 0.0f32;
    let mut min_y = 0.0f32;
    let mut max_x = screen_w as f32;
    let mut max_y = screen_h as f32;
    
    let mut mask_found = false;
    let mut current_id = start_obj.common.parent;

    while let Some(pid) = current_id {
        if let Some(parent) = store.get_objects().get(&pid) {
            if parent.common.mask_children {
                mask_found = true;
                
                let px = parent.common.position.x;
                let py = parent.common.position.y;
                let pw = parent.common.size.x;
                let ph = parent.common.size.y;

                min_x = min_x.max(px);
                min_y = min_y.max(py);
                max_x = max_x.min(px + pw);
                max_y = max_y.min(py + ph);
            }
            current_id = parent.common.parent;
        } else {
            break;
        }
    }

    if !mask_found {
        return None;
    }

    let final_x = min_x.max(0.0).floor() as u32;
    let final_y = min_y.max(0.0).floor() as u32;
    let final_w = (max_x - min_x).max(0.0).ceil() as u32;
    let final_h = (max_y - min_y).max(0.0).ceil() as u32;

    let clamped_x = final_x.min(screen_w);
    let clamped_y = final_y.min(screen_h);
    let clamped_w = final_w.min(screen_w - clamped_x);
    let clamped_h = final_h.min(screen_h - clamped_y);

    if clamped_w == 0 || clamped_h == 0 {
        Some([0, 0, 0, 0])
    } else {
        Some([clamped_x, clamped_y, clamped_w, clamped_h])
    }
}