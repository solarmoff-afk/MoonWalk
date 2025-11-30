use easy_gpu::{Context, Buffer};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3, Vec4, Vec2};
use crate::objects::{Object, Variant, ShaderId};
use super::common::BatchGroup;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct RectVertex {
    position: [f32; 3],
    color: [f32; 4],
    local_pos: [f32; 2],
    rect_size: [f32; 2],
    radii: [f32; 4],
}

pub struct RectBatcher;

impl RectBatcher {
    pub fn build(ctx: &Context, shader_id: ShaderId, objects: &[&Object], scissor: Option<[u32; 4]>) -> Option<BatchGroup> {
        let mut vertices = Vec::new();
        let mut z_sum = 0.0;

        for obj in objects {
            if let Variant::Rect(data) = &obj.variant {
                let size = obj.common.size;
                let half_size = size * 0.5;

                let center_x = obj.common.position.x + half_size.x;
                let center_y = obj.common.position.y + half_size.y;

                let model = Mat4::from_translation(Vec3::new(center_x, center_y, 0.0))
                    * Mat4::from_rotation_z(obj.common.rotation.to_radians());

                let c = obj.common.color.to_array();
                let r = data.radii.to_array();

                let positions = [
                    model * Vec4::new(-half_size.x,  half_size.y, 0.0, 1.0),
                    model * Vec4::new(-half_size.x, -half_size.y, 0.0, 1.0),
                    model * Vec4::new( half_size.x, -half_size.y, 0.0, 1.0),
                    model * Vec4::new( half_size.x,  half_size.y, 0.0, 1.0),
                ];
                
                let local_positions = [
                    Vec2::new(0.0, size.y),
                    Vec2::new(0.0, 0.0),
                    Vec2::new(size.x, 0.0),
                    Vec2::new(size.x, size.y),
                ];

                let indices = [0, 1, 2, 0, 2, 3];
                for &i in &indices {
                    vertices.push(RectVertex {
                        position: [positions[i].x, positions[i].y, obj.common.z],
                        color: c,
                        local_pos: local_positions[i].to_array(),
                        rect_size: size.to_array(),
                        radii: r,
                    });
                }

                z_sum += obj.common.z;
            }
        }
        
        if vertices.is_empty() {
            return None;
        }
        
        let byte_slice: &[u8] = bytemuck::cast_slice(&vertices);
        let vbo = Buffer::vertex(ctx, byte_slice);
        
        Some(BatchGroup {
            shader_id,
            vbo: Some(vbo),
            vertex_count: vertices.len(),
            scissor,
            sort_key: z_sum / objects.len() as f32,
            bind_group_uniforms: None,
        })
    }
}