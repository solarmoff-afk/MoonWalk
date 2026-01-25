// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use glam::{Vec3, Vec2};
use moonwalk::MoonWalk;
use crate::resources::{MeshData, Material};
use crate::core::types::Vertex3D;

struct Plane {
    point: Vec3,
    normal: Vec3,
}

impl Plane {
    fn new(point: Vec3, normal: Vec3) -> Self {
        Self { point, normal: normal.normalize() }
    }

    fn dist(&self, p: Vec3) -> f32 {
        self.normal.dot(p - self.point)
    }
}

pub fn cut_mesh(
    mw: &MoonWalk, 
    mesh: &MeshData, 
    point: Vec3,
    normal: Vec3,
    fill_cut: bool
) -> Option<(MeshData, MeshData)> {
    if mesh.vertices.is_empty() || mesh.indices.is_empty() {
        return None;
    }

    if normal == Vec3::ZERO {
        return None;
    }

    let plane = Plane::new(point, normal);
    
    let mut verts_a = Vec::new();
    let mut inds_a = Vec::new();
    let mut verts_b = Vec::new();
    let mut inds_b = Vec::new();
    
    let mut cut_edges_a = Vec::new();
    let mut cut_edges_b = Vec::new();

    for i in (0..mesh.indices.len()).step_by(3) {
        let i0 = mesh.indices[i] as usize;
        let i1 = mesh.indices[i+1] as usize;
        let i2 = mesh.indices[i+2] as usize;

        let v0 = mesh.vertices[i0];
        let v1 = mesh.vertices[i1];
        let v2 = mesh.vertices[i2];
        
        let p0 = Vec3::from(v0.position);
        let p1 = Vec3::from(v1.position);
        let p2 = Vec3::from(v2.position);
        
        let d0 = plane.dist(p0);
        let d1 = plane.dist(p1);
        let d2 = plane.dist(p2);
        
        let pos0 = d0 >= 0.0;
        let pos1 = d1 >= 0.0;
        let pos2 = d2 >= 0.0;
        
        if pos0 && pos1 && pos2 {
            add_tri(&mut verts_a, &mut inds_a, v0, v1, v2);
        } else if !pos0 && !pos1 && !pos2 {
            add_tri(&mut verts_b, &mut inds_b, v0, v1, v2);
        } else {
            split_tri(
                &v0, &v1, &v2, d0, d1, d2, 
                &mut verts_a, &mut inds_a, &mut cut_edges_a,
                &mut verts_b, &mut inds_b, &mut cut_edges_b
            );
        }
    }

    if fill_cut {
        fill_cap(&mut verts_a, &mut inds_a, &cut_edges_a, plane.normal);
        fill_cap(&mut verts_b, &mut inds_b, &cut_edges_b, -plane.normal);
    }
    
    if verts_a.is_empty() || verts_b.is_empty() {
        return None;
    }
    
    let mesh_a = create_mesh_data(mw, verts_a, inds_a, mesh.local_material.clone());
    let mesh_b = create_mesh_data(mw, verts_b, inds_b, mesh.local_material.clone());
    
    Some((mesh_a, mesh_b))
}

fn add_tri(verts: &mut Vec<Vertex3D>, inds: &mut Vec<u32>, v0: Vertex3D, v1: Vertex3D, v2: Vertex3D) {
    let start = verts.len() as u32;
    verts.push(v0);
    verts.push(v1);
    verts.push(v2);
    inds.push(start);
    inds.push(start + 1);
    inds.push(start + 2);
}

fn split_tri(
    v0: &Vertex3D, v1: &Vertex3D, v2: &Vertex3D,
    d0: f32, d1: f32, d2: f32,
    verts_a: &mut Vec<Vertex3D>, inds_a: &mut Vec<u32>, edges_a: &mut Vec<(Vec3, Vec3)>,
    verts_b: &mut Vec<Vertex3D>, inds_b: &mut Vec<u32>, edges_b: &mut Vec<(Vec3, Vec3)>,
) {
    let (v0, v1, v2, d0, d1, d2) = if (d0 >= 0.0) == (d1 >= 0.0) {
        (v2, v0, v1, d2, d0, d1)
    } else if (d1 >= 0.0) == (d2 >= 0.0) {
        (v0, v1, v2, d0, d1, d2)
    } else {
        (v1, v2, v0, d1, d2, d0)
    };

    let p0 = Vec3::from(v0.position);
    let p1 = Vec3::from(v1.position);
    let _p2 = Vec3::from(v2.position); // p2 не нужен для пересечения ребер v0 v1 и v0 v2

    let t0 = d0 / (d0 - d1);
    let t1 = d0 / (d0 - d2);
    
    let i0 = interpolate(*v0, *v1, t0);
    let i1 = interpolate(*v0, *v2, t1);
    
    let ip0 = Vec3::from(i0.position);
    let ip1 = Vec3::from(i1.position);

    if d0 >= 0.0 {
        add_tri(verts_a, inds_a, *v0, i0, i1);
        edges_a.push((ip0, ip1));
        
        add_tri(verts_b, inds_b, i0, *v1, *v2);
        add_tri(verts_b, inds_b, i0, *v2, i1);
        edges_b.push((ip1, p0));
        edges_b.push((p1, ip0)); 
    } else {
        add_tri(verts_b, inds_b, *v0, i0, i1);
        edges_b.push((ip0, ip1));
        
        add_tri(verts_a, inds_a, i0, *v1, *v2);
        add_tri(verts_a, inds_a, i0, *v2, i1);
        edges_a.push((ip1, p0)); 
        edges_a.push((p1, ip0)); 
    }
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

fn fill_cap(verts: &mut Vec<Vertex3D>, inds: &mut Vec<u32>, edges: &[(Vec3, Vec3)], normal: Vec3) {
    if edges.is_empty() { return; }
    
    let mut center = Vec3::ZERO;
    for (p1, p2) in edges {
        center += *p1;
        center += *p2;
    }
    center /= (edges.len() * 2) as f32;
    
    let c_vert = Vertex3D {
        position: center.to_array(),
        normal: normal.to_array(),
        uv: [0.5, 0.5], 
        tangent: [1.0, 0.0, 0.0],
    };
    
    for (p1, p2) in edges {
        let v1 = Vertex3D {
            position: p1.to_array(),
            normal: normal.to_array(),
            uv: [0.0, 0.0],
            tangent: [1.0, 0.0, 0.0],
        };

        let v2 = Vertex3D {
            position: p2.to_array(),
            normal: normal.to_array(),
            uv: [1.0, 0.0],
            tangent: [1.0, 0.0, 0.0],
        };
        
        let cross = (*p2 - *p1).cross(center - *p1);
        if cross.dot(normal) < 0.0 {
             add_tri(verts, inds, c_vert, v1, v2);
        } else {
             add_tri(verts, inds, c_vert, v2, v1);
        }
    }
}

fn create_mesh_data(mw: &MoonWalk, vertices: Vec<Vertex3D>, indices: Vec<u32>, mat: Option<Material>) -> MeshData {
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
