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
    pub scissor_rect: Option<[u32; 4]>,
    pub sort_key: f32,
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

fn calculate_scissor(
    store: &ObjectStore,
    start_obj: &Object,
    screen_w: u32,
    screen_h: u32
) -> Option<[u32; 4]> {
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

pub fn rebuild_batch_groups(
    device: &wgpu::Device,
    object_store: &ObjectStore,
    glyph_cache: &mut GlyphCache,
    font_system: &mut FontSystem,
    width: u32,
    height: u32,
) -> HashMap<RenderPass, Vec<BatchGroup>> {
    let mut grouped_objects: HashMap<(RenderPass, ShaderId, u64, Option<[u32; 4]>), Vec<&Object>> = HashMap::new();
    
    for object in object_store.get_objects().values() {
        let pass = match object.variant {
            Variant::Rect(_) => RenderPass::Simple,
            Variant::Text(_) => RenderPass::Glyph,
            Variant::Bezier(_) => RenderPass::Simple,
        };
        
        let shader_id = object_store.get_default_shader_for(object);
        let uniform_hash = hash_uniforms(&object.common.uniforms);
        let scissor = calculate_scissor(object_store, object, width, height);
        
        grouped_objects
            .entry((pass, shader_id, uniform_hash, scissor))
            .or_default()
            .push(object);
    }

    let mut all_batches: HashMap<RenderPass, Vec<BatchGroup>> = HashMap::new();

    for ((pass, shader_id, _, scissor), mut objects) in grouped_objects {
        if objects.is_empty() {
            continue;
        }

        objects.sort_by(|a, b| a.common.z.partial_cmp(&b.common.z).unwrap_or(std::cmp::Ordering::Equal));
        let uniforms = objects[0].common.uniforms.clone();
        
        match pass {
            RenderPass::Simple => {
                let mut rect_vertices = Vec::<RectVertex>::new();
                let mut rect_z_sum = 0.0;
                let mut rect_count = 0;
                
                for obj in &objects {
                    if let Variant::Rect(data) = &obj.variant {
                        append_rect_vertices(obj, data, &mut rect_vertices);
                        rect_z_sum += obj.common.z;
                        rect_count += 1;
                    }
                }
                
                if !rect_vertices.is_empty() {
                    let vertex_count = rect_vertices.len();
                    let vbo = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Vertex Buffer"),
                        contents: bytemuck::cast_slice(&rect_vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    });

                    let average_z = if rect_count > 0 { rect_z_sum / rect_count as f32 } else { 0.0 };

                    let batch = BatchGroup {
                        shader_id,
                        uniforms: uniforms.clone(),
                        storage_buffers: HashMap::new(),
                        vbo,
                        vertex_count,
                        scissor_rect: scissor,
                        sort_key: average_z,
                    };
                    all_batches.entry(pass).or_default().push(batch);
                }

                for obj in &objects {
                    if let Variant::Bezier(data) = &obj.variant {
                        let mut batch = create_bezier_batch(device, obj, data, shader_id, uniforms.clone(), width, height);
                        batch.scissor_rect = scissor;
                        batch.sort_key = obj.common.z;
                        all_batches.entry(pass).or_default().push(batch);
                    }
                }
            }

            RenderPass::Glyph => {
                let mut vertices = Vec::<TextVertex>::new();
                let mut text_z_sum = 0.0;
                let mut text_count = 0;

                for obj in &objects {
                    if let Variant::Text(data) = &obj.variant {
                        append_text_vertices(obj, data, &mut vertices, glyph_cache, font_system);
                        text_z_sum += obj.common.z;
                        text_count += 1;
                    }
                }
            
                if !vertices.is_empty() {
                    let vertex_count = vertices.len();
                    let vbo = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Text Vertex Buffer"),
                        contents: bytemuck::cast_slice(&vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    });

                    let average_z = if text_count > 0 { text_z_sum / text_count as f32 } else { 0.0 };

                    let batch = BatchGroup {
                        shader_id,
                        uniforms,
                        storage_buffers: HashMap::new(),
                        vbo,
                        vertex_count,
                        scissor_rect: scissor,
                        sort_key: average_z,
                    };
                    all_batches.entry(pass).or_default().push(batch);
                }
            }
        }
    }

    for groups in all_batches.values_mut() {
        groups.sort_by(|a, b| a.sort_key.partial_cmp(&b.sort_key).unwrap_or(std::cmp::Ordering::Equal));
    }

    all_batches
}

fn append_rect_vertices(obj: &Object, data: &RectData, vertices: &mut Vec<RectVertex>) {
    let size = obj.common.size;
    let half_size = size * 0.5;

    /*
        Нам необходимо вычислить центр в мировых кордах,
        позиция это верхний левый угол, центр это
        position + половина размера (half_size)
    */
    
    let center_x = obj.common.position.x + half_size.x;
    let center_y = obj.common.position.y + half_size.y;

    /*
        Внутри матрицы модели позиция указывается относительно
        центра чтобы не ломать к чертям вращение
    */

    let model = Mat4::from_translation(Vec3::new(center_x, center_y, 0.0))
        * Mat4::from_rotation_z(obj.common.rotation.to_radians());

    let c = obj.common.color.to_array();
    let r = data.radii.to_array();

    // -y это вверх, +y это вниз
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
}

fn append_text_vertices(
    obj: &Object,
    text_data: &TextData,
    vertices: &mut Vec<TextVertex>,
    glyph_cache: &mut GlyphCache,
    font_system: &mut FontSystem,
) {
    let size = obj.common.size;
    let half_size = size * 0.5;

    let center_x = obj.common.position.x + half_size.x;
    let center_y = obj.common.position.y + half_size.y;

    let model = Mat4::from_translation(Vec3::new(center_x, center_y, 0.0))
        * Mat4::from_rotation_z(obj.common.rotation.to_radians());

    let cosmic_fs = font_system.cosmic_mut();
    let c = obj.common.color.to_array();

    let offset_x = -half_size.x;
    let offset_y = -half_size.y;

    for run in text_data.buffer.layout_runs() {
        for glyph in run.glyphs.iter() {
            let cache_key = get_cache_key(glyph);
            let Some((image, uv_rect)) = glyph_cache.get_glyph(cache_key, cosmic_fs) else { continue; };

            let glyph_x = glyph.x + image.placement.left as f32;
            let glyph_y = run.line_y - image.placement.top as f32;
            
            let left = glyph_x + offset_x;
            let top = glyph_y + offset_y;
            
            let w = image.placement.width as f32;
            let h = image.placement.height as f32;
            
            let (uv_left, uv_top, uv_w, uv_h) = uv_rect;

            let positions = [
                model * Vec4::new(left, top + h, 0.0, 1.0),      // BL
                model * Vec4::new(left + w, top, 0.0, 1.0),      // TR (тут порядок индексов важен)
                model * Vec4::new(left, top, 0.0, 1.0),          // TL
                model * Vec4::new(left + w, top + h, 0.0, 1.0),  // BR
            ];

            let uvs = [
                Vec2::new(uv_left, uv_top + uv_h),
                Vec2::new(uv_left + uv_w, uv_top),
                Vec2::new(uv_left, uv_top), 
                Vec2::new(uv_left + uv_w, uv_top + uv_h),
            ];

            let positions_sorted = [
                model * Vec4::new(left, top + h, 0.0, 1.0),
                model * Vec4::new(left, top, 0.0, 1.0),
                model * Vec4::new(left + w, top, 0.0, 1.0),
                model * Vec4::new(left + w, top + h, 0.0, 1.0),
            ];
            
            let uvs_sorted = [
                Vec2::new(uv_left, uv_top + uv_h),
                Vec2::new(uv_left, uv_top),
                Vec2::new(uv_left + uv_w, uv_top),
                Vec2::new(uv_left + uv_w, uv_top + uv_h),
            ];

            let indices = [0, 1, 2, 0, 2, 3];
            for &i in &indices {
                vertices.push(TextVertex {
                    position: [positions_sorted[i].x, positions_sorted[i].y, obj.common.z],
                    color: c,
                    tex_coords: uvs_sorted[i].to_array(),
                });
            }
        }
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
        vertex_count: 3,
        scissor_rect: None,
        sort_key: 0.0,
    }
}