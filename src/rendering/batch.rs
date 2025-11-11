use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2, Vec3, Vec4};
use std::collections::HashMap;
use wgpu::util::DeviceExt;

use crate::objects::{hash_uniforms, Object, ObjectStore, RectData, ShaderId, TextData, BezierData, UniformValue, Variant};
use crate::rendering::glyph_cache::{get_cache_key, GlyphCache};
use crate::font::FontSystem;

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct RectVertex {
    position: [f32; 3],
    color: [f32; 4],
    local_pos: [f32; 2],
    rect_size: [f32; 2],
    radii: [f32; 4],
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct TextVertex {
    position: [f32; 3],
    color: [f32; 4],
    tex_coords: [f32; 2],
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum RenderPass {
    Simple,
    Glyph,
}

#[allow(dead_code)]
pub struct BatchGroup {
    pub shader_id: ShaderId,
    pub uniforms: HashMap<String, UniformValue>,
    pub storage_buffers: HashMap<u32, wgpu::Buffer>,
    pub vbo: wgpu::Buffer,
    pub vertex_count: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct BezierUniforms {
    resolution: [f32; 2],
    thickness: f32,
    smoothing: f32,
    curve_color: [f32; 4],
    point_count: u32,
    _padding: [u32; 3],
}

pub fn rebuild_batch_groups(
    device: &wgpu::Device,
    object_store: &ObjectStore,
    glyph_cache: &mut GlyphCache,
    font_system: &mut FontSystem,
    width: u32,
    height: u32,
) -> HashMap<RenderPass, Vec<BatchGroup>> {
    let mut grouped_objects: HashMap<(RenderPass, ShaderId, u64), Vec<&Object>> = HashMap::new();
    
    for object in object_store.get_objects().values() {
        let pass = match object.variant {
            Variant::Rect(_) => RenderPass::Simple,
            Variant::Text(_) => RenderPass::Glyph,
            Variant::Bezier(_) => RenderPass::Simple,
        };
        
        let shader_id = object_store.get_default_shader_for(object);
        let uniform_hash = hash_uniforms(&object.common.uniforms);
        
        grouped_objects
            .entry((pass, shader_id, uniform_hash))
            .or_default()
            .push(object);
    }

    let mut all_batches: HashMap<RenderPass, Vec<BatchGroup>> = HashMap::new();

    for ((pass, shader_id, _), objects) in grouped_objects {
        if objects.is_empty() {
            continue;
        }

        let uniforms = objects[0].common.uniforms.clone();
        
        match pass {
            RenderPass::Simple => {
                let mut rect_vertices = Vec::<RectVertex>::new();
                
                for obj in &objects {
                    if let Variant::Rect(data) = &obj.variant {
                        append_rect_vertices(obj, data, &mut rect_vertices);
                    }
                }
                
                if !rect_vertices.is_empty() {
                    let vertex_count = rect_vertices.len();
                    let batch = create_batch_gpu_objects(device, bytemuck::cast_slice(&rect_vertices), shader_id, uniforms.clone(), vertex_count);
                    all_batches.entry(pass).or_default().push(batch); 
                }

                for obj in &objects {
                    if let Variant::Bezier(data) = &obj.variant {
                        let batch = create_bezier_batch(device, obj, data, shader_id, uniforms.clone(), width, height);
                        all_batches.entry(pass).or_default().push(batch);
                    }
                }
            }

            RenderPass::Glyph => {
                let mut vertices = Vec::<TextVertex>::new();
                for obj in &objects {
                    if let Variant::Text(data) = &obj.variant {
                        append_text_vertices(obj, data, &mut vertices, glyph_cache, font_system);
                    }
                }
            
                if !vertices.is_empty() {
                    let vertex_count = vertices.len();
                    let batch = create_batch_gpu_objects(device, bytemuck::cast_slice(&vertices), shader_id, uniforms, vertex_count);
                    all_batches.entry(pass).or_default().push(batch);
                }
            }
        }
    }
    all_batches
}

fn append_rect_vertices(obj: &Object, data: &RectData, vertices: &mut Vec<RectVertex>) {
    let model = Mat4::from_translation(Vec3::new(obj.common.position.x, obj.common.position.y, 0.0))
        * Mat4::from_rotation_z(obj.common.rotation.to_radians());

    let size = obj.common.size;
    let half_size = size * 0.5;
    let c = obj.common.color.to_array();
    let r = data.radii.to_array();

    let positions = [
        model * Vec4::new(-half_size.x,  half_size.y, 0.0, 1.0),
        model * Vec4::new(-half_size.x, -half_size.y, 0.0, 1.0),
        model * Vec4::new( half_size.x, -half_size.y, 0.0, 1.0),
        model * Vec4::new( half_size.x,  half_size.y, 0.0, 1.0),
    ];
    
    let local_positions = [
        Vec2::new(0.0, size.y), Vec2::new(0.0, 0.0),
        Vec2::new(size.x, 0.0), Vec2::new(size.x, size.y),
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
}

fn append_text_vertices(
    obj: &Object,
    text_data: &TextData,
    vertices: &mut Vec<TextVertex>,
    glyph_cache: &mut GlyphCache,
    font_system: &mut FontSystem,
) {
    let model = Mat4::from_translation(Vec3::new(obj.common.position.x, obj.common.position.y, 0.0))
        * Mat4::from_rotation_z(obj.common.rotation.to_radians());

    let cosmic_fs = font_system.cosmic_mut();
    let c = obj.common.color.to_array();

    for run in text_data.buffer.layout_runs() {
        for glyph in run.glyphs.iter() {
            let cache_key = get_cache_key(glyph);
            let Some((image, uv_rect)) = glyph_cache.get_glyph(cache_key, cosmic_fs) else { continue; };

            let left = glyph.x + image.placement.left as f32;
            let top = run.line_y - image.placement.top as f32;
            let w = image.placement.width as f32;
            let h = image.placement.height as f32;
            
            let (uv_left, uv_top, uv_w, uv_h) = uv_rect;

            let positions = [
                model * Vec4::new(left, top + h, 0.0, 1.0),
                model * Vec4::new(left + w, top, 0.0, 1.0),
                model * Vec4::new(left, top, 0.0, 1.0),
                model * Vec4::new(left + w, top + h, 0.0, 1.0),
            ];

            let uvs = [
                Vec2::new(uv_left, uv_top + uv_h), Vec2::new(uv_left + uv_w, uv_top),
                Vec2::new(uv_left, uv_top), Vec2::new(uv_left + uv_w, uv_top + uv_h),
            ];

            let indices = [0, 2, 1, 0, 1, 3];
            for &i in &indices {
                 vertices.push(TextVertex {
                    position: [positions[i].x, positions[i].y, obj.common.z],
                    color: c,
                    tex_coords: uvs[i].to_array(),
                });
            }
        }
    }
}

fn create_batch_gpu_objects(device: &wgpu::Device, data: &[u8], shader_id: ShaderId, uniforms: HashMap<String, UniformValue>, vertex_count: usize) -> BatchGroup {
    let vbo = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: data,
        usage: wgpu::BufferUsages::VERTEX,
    });
    
    BatchGroup {
        shader_id,
        uniforms,
        storage_buffers: HashMap::new(),
        vbo,
        vertex_count,
    }
}

pub fn release_batch_groups(groups: &mut HashMap<RenderPass, Vec<BatchGroup>>) {
    for pass_groups in groups.values_mut() {
        for group in pass_groups {
            group.vbo.destroy();
        }
    }
    groups.clear();
}

fn create_bezier_batch(device: &wgpu::Device, obj: &Object, data: &BezierData, shader_id: ShaderId, uniforms_map: HashMap<String, UniformValue>, width: u32, height: u32) -> BatchGroup {
    let vbo = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Dummy Bezier VBO"),
        size: 0, 
        usage: wgpu::BufferUsages::VERTEX,
        mapped_at_creation: false,
    });
    
    let uniform_data = BezierUniforms {
        resolution: [width as f32, height as f32],
        thickness: data.thickness,
        smoothing: data.smoothing,
        curve_color: obj.common.color.to_array(),
        point_count: data.points.len() as u32,
        _padding: [0; 3],
    };

    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Bezier Uniform Buffer"),
        contents: bytemuck::cast_slice(&[uniform_data]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    
    let points_data: Vec<[f32; 2]> = data.points.iter().map(|p| [p.x, p.y]).collect();
    let storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Bezier Points Storage Buffer"),
        contents: bytemuck::cast_slice(&points_data),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    });
    
    let mut storage_buffers = HashMap::new();

    storage_buffers.insert(0, uniform_buffer);
    storage_buffers.insert(1, storage_buffer);

    BatchGroup {
        shader_id,
        uniforms: uniforms_map,
        storage_buffers,
        vbo,
        vertex_count: 3,}
} 