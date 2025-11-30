use easy_gpu::{Context, Buffer};
use textware::TextWare;
use glam::{Mat4, Vec3};
use bytemuck::{Pod, Zeroable};
use crate::objects::{Object, Variant, ShaderId};
use super::common::BatchGroup;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct TextVertexPacked {
    position: [f32; 3],
    color: [f32; 4],
    uv: [f32; 2],
}

pub struct TextBatcher;

impl TextBatcher {
    pub fn build(
        ctx: &Context, 
        textware: &mut TextWare, 
        shader_id: ShaderId, 
        objects: &mut [&mut Object], 
        scissor: Option<[u32; 4]>
    ) -> Option<BatchGroup> {
        let mut all_vertices = Vec::new();
        let mut z_sum = 0.0;

        for obj in objects {
            if let Variant::Text(data) = &mut obj.variant {
                let mesh = textware.generate_mesh(&mut data.inner);
                
                let model = Mat4::from_translation(Vec3::new(obj.common.position.x, obj.common.position.y, 0.0))
                    * Mat4::from_rotation_z(obj.common.rotation.to_radians());
                
                for idx in &mesh.indices {
                     let v = &mesh.vertices[*idx as usize];
                     let pos = model.transform_point3(glam::Vec3::from(v.position));
                     
                     all_vertices.push(TextVertexPacked {
                         position: pos.to_array(),
                         color: obj.common.color.to_array(),
                         uv: v.uv,
                     });
                }
                
                z_sum += obj.common.z;
            }
        }

        if all_vertices.is_empty() {
            return None;
        }

        let byte_slice: &[u8] = bytemuck::cast_slice(&all_vertices);
        let vbo = Buffer::vertex(ctx, byte_slice);

        Some(BatchGroup {
            shader_id,
            vbo: Some(vbo),
            vertex_count: all_vertices.len(),
            scissor,
            sort_key: z_sum / all_vertices.len() as f32,
            bind_group_uniforms: None,
        })
    }
}