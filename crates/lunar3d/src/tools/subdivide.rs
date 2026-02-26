// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use std::collections::HashMap;
use glam::{Vec3, Vec2};
use moonwalk::MoonWalk;

use crate::resources::{MeshData, Material};
use crate::core::types::Vertex3D;

pub fn subdivide(
    mw: &MoonWalk, 
    mesh: &MeshData, 
    iterations: usize
) -> MeshData {
    if iterations == 0 {
        return create_mesh_data(mw, mesh.vertices.clone(), mesh.indices.clone(), mesh.local_material.clone());
    }

    let mut current_vertices = mesh.vertices.clone();
    let mut current_indices = mesh.indices.clone();

    for _ in 0..iterations {
        let (next_verts, next_inds) = subdivide_once(&current_vertices, &current_indices);
        current_vertices = next_verts;
        current_indices = next_inds;
    }

    create_mesh_data(mw, current_vertices, current_indices, mesh.local_material.clone())
}

fn subdivide_once(vertices: &[Vertex3D], indices: &[u32]) -> (Vec<Vertex3D>, Vec<u32>) {
    let mut new_vertices = vertices.to_vec();
    let mut new_indices = Vec::with_capacity(indices.len() * 4);
    
    let mut midpoint_cache: HashMap<(u32, u32), u32> = HashMap::new();

    let mut get_midpoint = |i1: u32, i2: u32, verts: &mut Vec<Vertex3D>| -> u32 {
        let key = if i1 < i2 {
            (i1, i2)
        } else {
            (i2, i1)
        };
        
        if let Some(&idx) = midpoint_cache.get(&key) {
            return idx;
        }

        let v1 = &verts[i1 as usize];
        let v2 = &verts[i2 as usize];
        let mid_vert = interpolate(*v1, *v2, 0.5);
        
        let new_idx = verts.len() as u32;
        verts.push(mid_vert);
        midpoint_cache.insert(key, new_idx);
        
        new_idx
    };

    for chunk in indices.chunks(3) {
        if chunk.len() < 3 { break; }
        
        let i0 = chunk[0];
        let i1 = chunk[1];
        let i2 = chunk[2];

        let m01 = get_midpoint(i0, i1, &mut new_vertices);
        let m12 = get_midpoint(i1, i2, &mut new_vertices);
        let m20 = get_midpoint(i2, i0, &mut new_vertices);

        new_indices.push(i0); new_indices.push(m01); new_indices.push(m20);
        new_indices.push(i1); new_indices.push(m12); new_indices.push(m01);
        new_indices.push(i2); new_indices.push(m20); new_indices.push(m12);
        new_indices.push(m01); new_indices.push(m12); new_indices.push(m20);
    }

    (new_vertices, new_indices)
}

fn interpolate(v0: Vertex3D, v1: Vertex3D, t: f32) -> Vertex3D {
    let p = Vec3::from(v0.position).lerp(Vec3::from(v1.position), t);
    let n = Vec3::from(v0.normal).lerp(Vec3::from(v1.normal), t).normalize();
    let uv = Vec2::from(v0.uv).lerp(Vec2::from(v1.uv), t);
    let tan = Vec3::from(v0.tangent).lerp(Vec3::from(v1.tangent), t).normalize();
    
    Vertex3D {
        position: p.to_array(),
        normal: n.to_array(),
        uv: uv.to_array(),
        tangent: tan.to_array(),
    }
}

fn create_mesh_data(
    mw: &MoonWalk, 
    vertices: Vec<Vertex3D>, 
    indices: Vec<u32>, 
    mat: Option<Material>
) -> MeshData {
    let vb = mw.create_vertex_buffer(bytemuck::cast_slice(&vertices));
    let ib = mw.create_index_buffer_u32(bytemuck::cast_slice(&indices));
    
    MeshData {
        vertex_buffer: vb,
        index_buffer: ib,
        index_count: indices.len() as u32,
        local_material: mat,
        vertices,
        indices,
    }
}
