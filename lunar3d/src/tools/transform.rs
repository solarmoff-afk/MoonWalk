// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use glam::{Vec3, Vec4, Mat3, Mat4};
use moonwalk::MoonWalk;

use crate::resources::{MeshData, Material};
use crate::core::types::Vertex3D;

pub fn bake(
    mw: &MoonWalk, 
    mesh: &MeshData, 
    transform: Mat4
) -> MeshData {
    let mut new_vertices = mesh.vertices.clone();
    let normal_matrix = Mat3::from_mat4(transform);

    for v in &mut new_vertices {
        let pos = transform * Vec4::from((Vec3::from(v.position), 1.0));
        v.position = pos.truncate().to_array();

        let norm = normal_matrix * Vec3::from(v.normal);
        v.normal = norm.normalize_or_zero().to_array();
        
        let tan = normal_matrix * Vec3::from(v.tangent);
        v.tangent = tan.normalize_or_zero().to_array();
    }

    create_mesh_data(mw, new_vertices, mesh.indices.clone(), mesh.local_material.clone())
}

pub fn recalculate_normals(
    mw: &MoonWalk, 
    mesh: &MeshData
) -> MeshData {
    let mut new_vertices = mesh.vertices.clone();
    
    for v in &mut new_vertices {
        v.normal = [0.0; 3];
    }

    for i in (0..mesh.indices.len()).step_by(3) {
        let i0 = mesh.indices[i] as usize;
        let i1 = mesh.indices[i+1] as usize;
        let i2 = mesh.indices[i+2] as usize;

        if i0 >= new_vertices.len() || i1 >= new_vertices.len() || i2 >= new_vertices.len() {
            continue;
        }

        let p0 = Vec3::from(new_vertices[i0].position);
        let p1 = Vec3::from(new_vertices[i1].position);
        let p2 = Vec3::from(new_vertices[i2].position);

        let v1 = p1 - p0;
        let v2 = p2 - p0;
        let normal = v1.cross(v2).normalize_or_zero();

        let n0 = Vec3::from(new_vertices[i0].normal) + normal;
        let n1 = Vec3::from(new_vertices[i1].normal) + normal;
        let n2 = Vec3::from(new_vertices[i2].normal) + normal;

        new_vertices[i0].normal = n0.to_array();
        new_vertices[i1].normal = n1.to_array();
        new_vertices[i2].normal = n2.to_array();
    }

    for v in &mut new_vertices {
        let n = Vec3::from(v.normal).normalize_or_zero();
        v.normal = n.to_array();
        
        let t = Vec3::from(v.tangent);
        let ortho_tan = (t - n * n.dot(t)).normalize_or_zero();
        v.tangent = ortho_tan.to_array();
    }

    create_mesh_data(mw, new_vertices, mesh.indices.clone(), mesh.local_material.clone())
}

pub fn flip_faces(
    mw: &MoonWalk, 
    mesh: &MeshData
) -> MeshData {
    let mut new_vertices = mesh.vertices.clone();
    let mut new_indices = mesh.indices.clone();

    for v in &mut new_vertices {
        let n = Vec3::from(v.normal) * -1.0;
        v.normal = n.to_array();
    }

    for i in (0..new_indices.len()).step_by(3) {
        new_indices.swap(i + 1, i + 2);
    }

    create_mesh_data(mw, new_vertices, new_indices, mesh.local_material.clone())
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
