// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use std::collections::HashMap;
use glam::{Mat3, Mat4};

use crate::scene::types::*;
use crate::scene::Scene3D;

pub(crate) fn prepare_batches(
    scene: &Scene3D
) -> (HashMap<BatchKey, Vec<InstanceRaw>>, HashMap<BatchKey, (usize, usize)>, Vec<InstanceRaw>) {
    let mut batches: HashMap<BatchKey, Vec<InstanceRaw>> = HashMap::new();
    
    let count = scene.inst_alive.len();
    let mut world_matrices = vec![Mat4::IDENTITY; count];
    let mut computed = vec![false; count];

    for i in 0..count {
        if scene.inst_alive[i] {
            compute_matrix_recursive(i, scene, &mut world_matrices, &mut computed);
        }
    }

    // Группировка объектов по материалам
    for (i, alive) in scene.inst_alive.iter().enumerate() {
        if !alive {
            continue;
        }
        
        let model = world_matrices[i];
        
        let normal_mat = Mat3::from_mat4(model.inverse().transpose());
        let c0 = normal_mat.col(0); let c1 = normal_mat.col(1); let c2 = normal_mat.col(2);

        let raw = InstanceRaw {
            model: model.to_cols_array_2d(),
            normal_mat_0: [c0.x, c0.y, c0.z, 0.0],
            normal_mat_1: [c1.x, c1.y, c1.z, 0.0],
            normal_mat_2: [c2.x, c2.y, c2.z, 0.0],
            color: scene.inst_mat_base_color[i].to_array(),
            metallic: scene.inst_mat_metallic[i],
            roughness: scene.inst_mat_roughness[i],
            unlit: if scene.inst_unlit[i] {
                1.0
            } else {
                0.0
            },
        };

        let mesh_id = scene.inst_mesh_ids[i];
        
        let albedo = scene.inst_mat_albedo_id[i].unwrap_or(scene.default_white);
        let normal = scene.inst_mat_normal_id[i].unwrap_or(scene.default_normal);
        let mr = scene.inst_mat_mr_id[i].unwrap_or(scene.default_white);

        let key = BatchKey {
            mesh_id,
            albedo_id: albedo,
            normal_id: normal,
            mr_id: mr,
        };

        batches.entry(key).or_default().push(raw);
    }

    let mut all_data = Vec::new();
    let mut offsets = HashMap::new();
    let mut current_offset = 0;

    for (key, list) in &batches {
        offsets.insert(*key, (current_offset, list.len()));
        all_data.extend_from_slice(list);
        current_offset += list.len();
    }
    
    (batches, offsets, all_data)
}

fn compute_matrix_recursive(
    idx: usize, 
    scene: &Scene3D, 
    matrices: &mut [Mat4], 
    computed: &mut [bool]
) -> Mat4 {
    if computed[idx] {
        return matrices[idx];
    }
    
    let pos = scene.inst_positions[idx];
    let rot = scene.inst_rotations[idx];
    let scale = scene.inst_scales[idx];
    
    let local = Mat4::from_translation(pos)
        * Mat4::from_rotation_z(rot.z)
        * Mat4::from_rotation_x(rot.x)
        * Mat4::from_rotation_y(rot.y)
        * Mat4::from_scale(scale);
        
    let world = if let Some(parent_idx) = scene.inst_parents[idx] {
        if parent_idx == idx {
            local
        } else {
            let parent_mat = compute_matrix_recursive(parent_idx, scene, matrices, computed);
            parent_mat * local
        }
    } else {
        local
    };
    
    matrices[idx] = world;
    computed[idx] = true;
    world
}