// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use glam::Vec3;
use moonwalk::MoonWalk;
use crate::resources::MeshData;
use crate::core::types::Vertex3D;
use crate::tools::knife;

struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_f32(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let x = (self.state >> 33) as u32;
        (x as f32) / (u32::MAX as f32)
    }

    fn range(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.next_f32()
    }

    fn dir(&mut self) -> Vec3 {
        let x = self.range(-1.0, 1.0);
        let y = self.range(-1.0, 1.0);
        let z = self.range(-1.0, 1.0);

        Vec3::new(x, y, z).normalize_or_zero()
    }
}

fn get_bounds(vertices: &[Vertex3D]) -> (Vec3, Vec3) {
    if vertices.is_empty() {
        return (Vec3::ZERO, Vec3::ZERO);
    }

    let mut min = Vec3::splat(f32::MAX);
    let mut max = Vec3::splat(f32::MIN);

    for v in vertices {
        let p = Vec3::from(v.position);
        min = min.min(p);
        max = max.max(p);
    }
    (min, max)
}

pub fn shatter_mesh(
    mw: &MoonWalk,
    source_mesh: &MeshData,
    iterations: usize,
    seed: u64,
    fill_cuts: bool
) -> Vec<MeshData> {
    let initial_vb = mw.create_vertex_buffer(bytemuck::cast_slice(&source_mesh.vertices));
    let initial_ib = mw.create_index_buffer_u32(bytemuck::cast_slice(&source_mesh.indices));
    
    let initial_shard = MeshData {
        vertex_buffer: initial_vb,
        index_buffer: initial_ib,
        index_count: source_mesh.indices.len() as u32,
        local_material: source_mesh.local_material.clone(),
        vertices: source_mesh.vertices.clone(),
        indices: source_mesh.indices.clone(),
    };

    let mut fragments = vec![initial_shard];
    let mut rng = Rng::new(seed);

    for _ in 0..iterations {
        let target_idx = (0..fragments.len())
            .max_by_key(|i| fragments[*i].vertices.len())
            .unwrap_or(0);

        let target = fragments.remove(target_idx);
        let (min, max) = get_bounds(&target.vertices);
        
        let point = Vec3::new(
            rng.range(min.x, max.x),
            rng.range(min.y, max.y),
            rng.range(min.z, max.z),
        );
        
        let normal = rng.dir();

        if let Some((part_a, part_b)) = knife::cut_mesh(mw, &target, point, normal, fill_cuts) {
            fragments.push(part_a);
            fragments.push(part_b);
        } else {
            fragments.push(target);
        }
    }

    fragments
}
