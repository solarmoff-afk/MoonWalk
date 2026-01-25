// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use std::io::Cursor;
use glam::{Vec2, Vec3, Vec4};
use moonwalk::{MoonWalk, VertexLayout, StepMode, VertexAttr, Format};
use moonwalk::rendering::custom::MoonBuffer;
use crate::core::types::Vertex3D;

pub struct MeshData {
    pub vertex_buffer: MoonBuffer,
    pub index_buffer: MoonBuffer,
    pub index_count: u32,
    pub local_material: Option<Material>,
    pub vertices: Vec<Vertex3D>,
    pub indices: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Material {
    pub albedo_id: Option<u32>,
    pub normal_id: Option<u32>,
    pub mr_id: Option<u32>, 
    pub base_color: Vec4,
    pub metallic: f32,
    pub roughness: f32,
    pub unlit: bool,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    pub model: [[f32; 4]; 4],
    pub normal_mat_0: [f32; 4],
    pub normal_mat_1: [f32; 4],
    pub normal_mat_2: [f32; 4],
    pub color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub unlit: f32,
}

pub fn vertex_layout() -> VertexLayout {
    VertexLayout::new()
        .stride(std::mem::size_of::<Vertex3D>() as u32)
        .step_mode(StepMode::Vertex)
        .add_attr(VertexAttr::new().format(Format::Float32x3).location(0).offset(0))  
        .add_attr(VertexAttr::new().format(Format::Float32x3).location(1).offset(12)) 
        .add_attr(VertexAttr::new().format(Format::Float32x2).location(2).offset(24)) 
        .add_attr(VertexAttr::new().format(Format::Float32x3).location(3).offset(32)) 
}

pub fn instance_layout() -> VertexLayout {
    VertexLayout::new()
        .stride(std::mem::size_of::<InstanceRaw>() as u32)
        .step_mode(StepMode::Instance)
        .add_attr(VertexAttr::new().format(Format::Float32x4).location(4).offset(0))
        .add_attr(VertexAttr::new().format(Format::Float32x4).location(5).offset(16))
        .add_attr(VertexAttr::new().format(Format::Float32x4).location(6).offset(32))
        .add_attr(VertexAttr::new().format(Format::Float32x4).location(7).offset(48))
        .add_attr(VertexAttr::new().format(Format::Float32x4).location(8).offset(64))
        .add_attr(VertexAttr::new().format(Format::Float32x4).location(9).offset(80))
        .add_attr(VertexAttr::new().format(Format::Float32x4).location(10).offset(96))
        .add_attr(VertexAttr::new().format(Format::Float32x4).location(11).offset(112))
        .add_attr(VertexAttr::new().format(Format::Float32).location(12).offset(128))
        .add_attr(VertexAttr::new().format(Format::Float32).location(13).offset(132))
}

pub fn load_obj(mw: &MoonWalk, obj_bytes: &[u8]) -> Vec<MeshData> {
    let mut reader = Cursor::new(obj_bytes);
    let (models, _) = tobj::load_obj_buf(
        &mut reader,
        &tobj::LoadOptions { 
            single_index: true, 
            triangulate: true, 
            ..Default::default() 
        },
        |_| Ok(Default::default())
    ).expect("Failed to load OBJ");

    let mut result = Vec::new();

    for model in models {
        let mesh = model.mesh;
        let mut vertices = Vec::new();
        let num_verts = mesh.positions.len() / 3;
        let mut tangents = vec![Vec3::ZERO; num_verts];

        for i in (0..mesh.indices.len()).step_by(3) {
            let i0 = mesh.indices[i] as usize;
            let i1 = mesh.indices[i+1] as usize;
            let i2 = mesh.indices[i+2] as usize;

            if i0 >= num_verts || i1 >= num_verts || i2 >= num_verts {
                continue;
            }

            let v0 = Vec3::from_slice(&mesh.positions[i0*3..i0*3+3]);
            let v1 = Vec3::from_slice(&mesh.positions[i1*3..i1*3+3]);
            let v2 = Vec3::from_slice(&mesh.positions[i2*3..i2*3+3]);

            let uv0 = if !mesh.texcoords.is_empty() {
                Vec2::from_slice(&mesh.texcoords[i0*2..i0*2+2])
            } else {
                Vec2::ZERO 
            };

            let uv1 = if !mesh.texcoords.is_empty() {
                Vec2::from_slice(&mesh.texcoords[i1*2..i1*2+2])
            } else {
                Vec2::ZERO 
            };

            let uv2 = if !mesh.texcoords.is_empty() {
                Vec2::from_slice(&mesh.texcoords[i2*2..i2*2+2]) 
            } else {
                Vec2::ZERO 
            };

            let delta_pos1 = v1 - v0;
            let delta_pos2 = v2 - v0;
            let delta_uv1 = uv1 - uv0;
            let delta_uv2 = uv2 - uv0;

            let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x + 0.0001);
            let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;

            tangents[i0] += tangent;
            tangents[i1] += tangent;
            tangents[i2] += tangent;
        }

        for i in 0..num_verts {
            let n = if !mesh.normals.is_empty() { 
                Vec3::from_slice(&mesh.normals[i*3..i*3+3]) 
            } else { 
                Vec3::Y 
            };
            
            let t = tangents[i];
            let tangent = (t - n * n.dot(t)).normalize_or_zero();

            vertices.push(Vertex3D {
                position: [mesh.positions[i*3], mesh.positions[i*3+1], mesh.positions[i*3+2]],
                normal: n.to_array(),
                
                uv: if !mesh.texcoords.is_empty() {
                    [mesh.texcoords[i*2], 1.0 - mesh.texcoords[i*2+1]]
                } else {
                    [0.0, 0.0]
                },

                tangent: tangent.to_array(),
            });
        }

        let vb = mw.create_vertex_buffer(bytemuck::cast_slice(&vertices));
        let indices = mesh.indices;
        let ib = mw.create_index_buffer_u32(bytemuck::cast_slice(&indices));
        
        result.push(MeshData {
            vertex_buffer: vb,
            index_buffer: ib,
            index_count: indices.len() as u32,
            local_material: None,
            vertices,
            indices,
        });
    }

    result
}

pub fn load_gltf_bytes(mw: &mut MoonWalk, bytes: &[u8]) -> Vec<MeshData> {
    let (document, buffers, images) = gltf::import_slice(bytes)
        .expect("Failed to load GLTF bytes");

    let mut texture_map = std::collections::HashMap::new();
    
    for (i, image) in images.iter().enumerate() {
        let (width, height) = (image.width, image.height);
        let rgba = match image.format {
            gltf::image::Format::R8G8B8 => {
                let pixels = &image.pixels;
                let mut data = Vec::with_capacity(pixels.len() / 3 * 4);
                for chunk in pixels.chunks(3) {
                    data.extend_from_slice(chunk);
                    data.push(255);
                }
                data
            },
            gltf::image::Format::R8G8B8A8 => image.pixels.clone(),
            _ => vec![255; (width * height * 4) as usize],
        };
        
        if let Ok(tex) = moonwalk::rendering::texture::Texture::from_raw(&mw.renderer.context, &rgba, width, height, "GLTF Tex") {
            let id = mw.renderer.state.add_texture(tex);
            texture_map.insert(i, id);
        }
    }

    let mut result = Vec::new();

    for mesh in document.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            
            let positions: Vec<[f32; 3]> = reader.read_positions()
                .map(|i| i.collect())
                .unwrap_or_default();

            if positions.is_empty() {
                continue;
            }
            
            let normals: Vec<[f32; 3]> = reader.read_normals()
                .map(|i| i.collect())
                .unwrap_or_default();
            
            let uvs: Vec<[f32; 2]> = reader.read_tex_coords(0)
                .map(|i| i.into_f32().collect())
                .unwrap_or_default();

            let tangents: Vec<[f32; 4]> = reader.read_tangents()
                .map(|i| i.collect())
                .unwrap_or_default();

            let indices: Vec<u32> = reader.read_indices().map(|i| i.into_u32()
                .collect())
                .unwrap_or_default();

            let mut vertices = Vec::with_capacity(positions.len());
            for i in 0..positions.len() {
                let norm = if i < normals.len() { 
                    normals[i] 
                } else { 
                    [0.0, 1.0, 0.0] 
                };

                let uv = if i < uvs.len() {
                    uvs[i]
                } else {
                    [0.0, 0.0]
                };

                let tan = if i < tangents.len() {
                    [tangents[i][0], tangents[i][1], tangents[i][2]]
                } else {
                    [1.0, 0.0, 0.0]
                };

                vertices.push(Vertex3D {
                    position: positions[i],
                    normal: norm,
                    uv,
                    tangent: tan,
                });
            }

            let vb = mw.create_vertex_buffer(bytemuck::cast_slice(&vertices));
            let ib = mw.create_index_buffer_u32(bytemuck::cast_slice(&indices));
            
            let material = primitive.material();
            let pbr = material.pbr_metallic_roughness();
            
            let albedo_id = pbr.base_color_texture()
                .and_then(|t| texture_map.get(&t.texture()
                .source().index())).copied();
            
            let normal_id = material.normal_texture().and_then(|t| texture_map
                .get(&t.texture().source().index())).copied();
            let mr_id = pbr.metallic_roughness_texture().and_then(|t| texture_map
                .get(&t.texture().source().index())).copied();

            let mat = Material {
                albedo_id,
                normal_id,
                mr_id,
                base_color: Vec4::from(pbr.base_color_factor()),
                metallic: pbr.metallic_factor(),
                roughness: pbr.roughness_factor(),
                unlit: false,
            };

            result.push(MeshData {
                vertex_buffer: vb,
                index_buffer: ib,
                index_count: indices.len() as u32,
                local_material: Some(mat),
                vertices,
                indices,
            });
        }
    }

    result
}
