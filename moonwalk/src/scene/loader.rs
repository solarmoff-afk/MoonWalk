// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use std::collections::HashMap;
use std::io::Cursor;
use glam::{Vec2, Vec3, Vec4};
use crate::MoonWalk;
use crate::scene::types::{MeshData, Vertex3D, Material};

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
        let ib = mw.create_index_buffer_u32(bytemuck::cast_slice(&mesh.indices));
        
        result.push(MeshData {
            vertex_buffer: vb,
            index_buffer: ib,
            index_count: mesh.indices.len() as u32,
            local_material: None,
        });
    }
    
    result
}

pub fn load_gltf(mw: &mut MoonWalk, path: &str) -> Vec<MeshData> {
    let (document, buffers, images) = gltf::import(path).expect("Failed to import GLTF");
    
    let mut texture_map: HashMap<usize, u32> = HashMap::new();
    
    for (i, image) in images.iter().enumerate() {
        let format = image.format;
        let (width, height) = (image.width, image.height);
        let pixels = &image.pixels;
        
        let rgba = match format {
            gltf::image::Format::R8G8B8 => {
                let mut data = Vec::with_capacity(pixels.len() / 3 * 4);
                
                for chunk in pixels.chunks(3) {
                    data.extend_from_slice(chunk);
                    data.push(255);
                }

                data
            },

            gltf::image::Format::R8G8B8A8 => pixels.clone(),
            
            _ => {
                vec![255; (width * height * 4) as usize] 
            }
        };
        
        if let Ok(tex) = crate::rendering::texture::Texture::from_raw(&mw.renderer.context, &rgba, width, height, "GLTF Texture") {
            let id = mw.renderer.state.add_texture(tex);
            texture_map.insert(i, id);
        }
    }

    let mut result = Vec::new();

    for mesh in document.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let mut vertices = Vec::new();
            
            let positions: Vec<[f32; 3]> = reader.read_positions().map(|iter| iter.collect()).unwrap_or_default();
            let normals: Vec<[f32; 3]> = reader.read_normals().map(|iter| iter.collect()).unwrap_or_default();
            let tex_coords: Vec<[f32; 2]> = reader.read_tex_coords(0).map(|read| read.into_f32().collect()).unwrap_or_default();
            let tangents: Vec<[f32; 4]> = reader.read_tangents().map(|iter| iter.collect()).unwrap_or_default();
            let indices: Vec<u32> = reader.read_indices().map(|read| read.into_u32().collect()).unwrap_or_default();
            
            if positions.is_empty() {
                continue;
            }

            let need_calc_tangents = tangents.is_empty()
                && !tex_coords.is_empty()
                && !normals.is_empty()
                && !indices.is_empty();
            
            let mut calculated_tangents = if need_calc_tangents {
                vec![Vec3::ZERO; positions.len()]
            } else {
                Vec::new()
            };

            if need_calc_tangents {
                calculate_tangents(&indices, &positions, &tex_coords, &mut calculated_tangents);
            }

            for i in 0..positions.len() {
                let pos = positions[i];
                let norm = if i < normals.len() {
                    normals[i]
                } else {
                    [0.0, 1.0, 0.0]
                };

                let uv = if i < tex_coords.len() {
                    tex_coords[i]
                } else {
                    [0.0, 0.0]
                };
                
                let tan = if i < tangents.len() {
                     let t = tangents[i];
                     [t[0], t[1], t[2]] 
                } else if need_calc_tangents {
                    calculated_tangents[i].to_array()
                } else {
                    [1.0, 0.0, 0.0]
                };

                vertices.push(Vertex3D {
                    position: pos,
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
                .and_then(|info| texture_map.get(&info.texture().source().index()))
                .cloned();
            
            let normal_id = material.normal_texture()
                .and_then(|info| texture_map.get(&info.texture().source().index()))
                .cloned();
            
            let mr_id = pbr.metallic_roughness_texture()
                .and_then(|info| texture_map.get(&info.texture().source().index()))
                .cloned();

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
            });
        }
    }
    
    result
}

fn calculate_tangents(indices: &[u32], positions: &[[f32; 3]], uvs: &[[f32; 2]], out_tangents: &mut [Vec3]) {
    for i in (0..indices.len()).step_by(3) {
        let i0 = indices[i] as usize;
        let i1 = indices[i+1] as usize;
        let i2 = indices[i+2] as usize;

        let v0 = Vec3::from(positions[i0]);
        let v1 = Vec3::from(positions[i1]);
        let v2 = Vec3::from(positions[i2]);

        let uv0 = Vec2::from(uvs[i0]);
        let uv1 = Vec2::from(uvs[i1]);
        let uv2 = Vec2::from(uvs[i2]);

        let delta_pos1 = v1 - v0;
        let delta_pos2 = v2 - v0;
        let delta_uv1 = uv1 - uv0;
        let delta_uv2 = uv2 - uv0;

        let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x + 0.0001);
        let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;

        out_tangents[i0] += tangent;
        out_tangents[i1] += tangent;
        out_tangents[i2] += tangent;
    }

    for t in out_tangents {
        *t = t.normalize_or_zero();
    }
}
