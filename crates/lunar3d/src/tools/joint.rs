// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use moonwalk::MoonWalk;
use crate::resources::{MeshData, Material};
use crate::core::types::Vertex3D;

pub fn joint_meshes(
    mw: &MoonWalk, 
    meshes: Vec<&MeshData>,
    target_material: Option<Material>
) -> Option<MeshData> {
    if meshes.is_empty() {
        return None;
    }

    let mut combined_vertices: Vec<Vertex3D> = Vec::new();
    let mut combined_indices: Vec<u32> = Vec::new();
    
    let mut v_count = 0;
    let mut i_count = 0;
    for m in &meshes {
        v_count += m.vertices.len();
        i_count += m.indices.len();
    }
    
    combined_vertices.reserve(v_count);
    combined_indices.reserve(i_count);
    
    let final_mat = if target_material.is_some() {
        target_material
    } else {
        meshes[0].local_material.clone()
    };

    for mesh in meshes {
        let offset = combined_vertices.len() as u32;
        
        combined_vertices.extend_from_slice(&mesh.vertices);
        
        for idx in &mesh.indices {
            combined_indices.push(idx + offset);
        }
    }
    
    if combined_vertices.is_empty() {
        return None;
    }

    let vb = mw.create_vertex_buffer(bytemuck::cast_slice(&combined_vertices));
    let ib = mw.create_index_buffer_u32(bytemuck::cast_slice(&combined_indices));
    
    Some(MeshData {
        vertex_buffer: vb,
        index_buffer: ib,
        index_count: combined_indices.len() as u32,
        local_material: final_mat,
        vertices: combined_vertices,
        indices: combined_indices,
    })
}