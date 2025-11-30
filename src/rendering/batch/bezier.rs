use easy_gpu::Context;
use easy_gpu::Buffer;
use bytemuck::{Pod, Zeroable};
use crate::objects::{Object, Variant, ShaderId};
use super::common::BatchGroup;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct BezierUniforms {
    resolution: [f32; 2],
    thickness: f32,
    smoothing: f32,
    curve_color: [f32; 4],
    point_count: u32,
    _pad: [u32; 3],
}

pub struct BezierBatcher;

impl BezierBatcher {
    pub fn build(ctx: &Context, shader_id: ShaderId, objects: &[&Object], scissor: Option<[u32; 4]>) -> Option<BatchGroup> {
        if objects.is_empty() {
            return None;
        }
        
        let obj = objects[0];
        if let Variant::Bezier(data) = &obj.variant {
            let resolution = [ctx.config.width as f32, ctx.config.height as f32];
            let uniforms = BezierUniforms {
                resolution,
                thickness: data.thickness,
                smoothing: data.smoothing,
                curve_color: obj.common.color.to_array(),
                point_count: data.points.len() as u32,
                _pad: [0; 3],
            };
            
            let uniform_buffer = Buffer::uniform(ctx, &uniforms);
            
            let points_raw: Vec<[f32; 2]> = data.points.iter().map(|v| v.to_array()).collect();
            let storage_buffer = Buffer::storage(ctx, &points_raw);
            
            let layout = ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Bezier Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ]
            });
            
            let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform_buffer.raw.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: storage_buffer.raw.as_entire_binding(),
                    }
                ],
                label: Some("Bezier BindGroup"),
            });
            
            return Some(BatchGroup {
                shader_id,
                vbo: None,
                vertex_count: 3,
                scissor,
                sort_key: obj.common.z,
                bind_group_uniforms: Some(bind_group),
            });
        }
        
        None
    }
}