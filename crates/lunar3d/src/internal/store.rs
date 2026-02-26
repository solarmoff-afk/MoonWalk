// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use glam::{Vec3, Vec4};
use moonwalk::MoonWalk;
use moonwalk::rendering::custom::MoonBuffer;
use crate::core::types::{ObjectId, MeshId};
use crate::resources::InstanceRaw;

pub struct ObjectStore {
    // Модели
    pub mesh_ids: Vec<usize>,
    

    pub positions: Vec<Vec3>,
    pub rotations: Vec<Vec3>,
    pub scales: Vec<Vec3>,
    
    // Реинкарнация объектов (могут ли другие объекты занять место по этому id)
    pub alive: Vec<bool>,
    pub free_slots: Vec<usize>,
    

    // Слой позволяет применять свой MoonPipeline к конкретному объекту
    pub layer_names: Vec<String>,
    pub colors: Vec<Vec4>,

    // Материал
    pub metallic: Vec<f32>,
    pub roughness: Vec<f32>,
    pub albedo_ids: Vec<Option<u32>>,
    pub normal_ids: Vec<Option<u32>>,
    
    // Игнорирует ли объект свет (использовать для ламп)
    pub unlit: Vec<bool>,

    pub buffer: MoonBuffer,
    pub capacity: usize,
}

impl ObjectStore {
    pub fn new(mw: &MoonWalk) -> Self {
        // 100 объектов выделяются по умолчанию для оптимизации
        
        let capacity = 100;
        let buffer = mw.create_vertex_buffer(&vec![0u8; capacity * std::mem::size_of::<InstanceRaw>()]);
        
        Self {
            mesh_ids: Vec::new(),
            positions: Vec::new(),
            rotations: Vec::new(),
            scales: Vec::new(),
            alive: Vec::new(),
            free_slots: Vec::new(),
            layer_names: Vec::new(),
            colors: Vec::new(),
            metallic: Vec::new(),
            roughness: Vec::new(),
            albedo_ids: Vec::new(),
            normal_ids: Vec::new(),
            unlit: Vec::new(),
            buffer,
            capacity,
        }
    }

    pub fn new_object(&mut self, mesh_id: MeshId) -> ObjectId {
        if let Some(idx) = self.free_slots.pop() {
            self.alive[idx] = true;
            self.mesh_ids[idx] = mesh_id.0;
            self.reset_slot(idx);
            return ObjectId(idx);
        }

        let idx = self.positions.len();
        self.alive.push(true);
        self.mesh_ids.push(mesh_id.0);
        
        self.positions.push(Vec3::ZERO);
        self.rotations.push(Vec3::ZERO);
        self.scales.push(Vec3::ONE);
        self.layer_names.push("pbr".to_string());
        self.colors.push(Vec4::ONE);
        self.metallic.push(0.0);
        self.roughness.push(0.5);
        self.albedo_ids.push(None);
        self.normal_ids.push(None);
        self.unlit.push(false);
        
        ObjectId(idx)
    }

    pub fn remove(&mut self, id: ObjectId) {
        if id.0 < self.alive.len() {
            if self.alive[id.0] {
                self.alive[id.0] = false;
                self.free_slots.push(id.0);
            }
        }
    }

    fn reset_slot(&mut self, idx: usize) {
        self.positions[idx] = Vec3::ZERO;
        self.rotations[idx] = Vec3::ZERO;
        self.scales[idx] = Vec3::ONE;
        self.layer_names[idx] = "pbr".to_string();
        self.colors[idx] = Vec4::ONE;
        self.metallic[idx] = 0.0;
        self.roughness[idx] = 0.5;
        self.albedo_ids[idx] = None;
        self.normal_ids[idx] = None;
        self.unlit[idx] = false;
    }

    pub fn set_pos(&mut self, id: ObjectId, v: Vec3) { 
        if id.0 < self.positions.len() {
            self.positions[id.0] = v;
        } 
    }

    pub fn set_rot(&mut self, id: ObjectId, v: Vec3) { 
        if id.0 < self.rotations.len() {
            self.rotations[id.0] = v;
        } 
    }

    pub fn set_scale(&mut self, id: ObjectId, v: Vec3) { 
        if id.0 < self.scales.len() {
            self.scales[id.0] = v;
        } 
    }

    pub fn set_color(&mut self, id: ObjectId, v: Vec4) { 
        if id.0 < self.colors.len() {
            self.colors[id.0] = v;
        } 
    }

    pub fn set_layer(&mut self, id: ObjectId, v: &str) { 
        if id.0 < self.layer_names.len() {
            self.layer_names[id.0] = v.to_string();
        } 
    }

    pub fn set_albedo(&mut self, id: ObjectId, v: u32) { 
        if id.0 < self.albedo_ids.len() {
            self.albedo_ids[id.0] = Some(v);
        } 
    }


    pub fn set_normal(&mut self, id: ObjectId, v: u32) { 
        if id.0 < self.normal_ids.len() {
            self.normal_ids[id.0] = Some(v);
        } 
    }

    pub fn set_metallic(&mut self, id: ObjectId, v: f32) { 
        if id.0 < self.metallic.len() {
            self.metallic[id.0] = v;
        } 
    }

    pub fn set_roughness(&mut self, id: ObjectId, v: f32) { 
        if id.0 < self.roughness.len() {
            self.roughness[id.0] = v;
        } 
    }

    pub fn set_unlit(&mut self, id: ObjectId, v: bool) {
        if id.0 < self.unlit.len() {
            self.unlit[id.0] = v;
        }
    }

    pub fn clear(&mut self) {
        for i in 0..self.alive.len() {
            if self.alive[i] {
                self.alive[i] = false;
                self.free_slots.push(i);
            }
        }
    }

    pub fn set_all_layers(&mut self, v: &str) {
        for i in 0..self.layer_names.len() {
            if self.alive[i] {
                self.layer_names[i] = v.to_string();
            }
        }
    }

    pub fn get_pos(&self, id: ObjectId) -> Vec3 {
        if id.0 < self.positions.len() {
            return self.positions[id.0];
        } else {
            return Vec3::ZERO;
        }
    }

    pub fn get_rot(&self, id: ObjectId) -> Vec3 {
        if id.0 < self.rotations.len() {
            return self.rotations[id.0];
        } else {
            return Vec3::ZERO;
        }
    }

    pub fn get_scale(&self, id: ObjectId) -> Vec3 {
        if id.0 < self.scales.len() {
            return self.scales[id.0];
        } else {
            return Vec3::ONE;
        }
    }

    pub fn get_color(&self, id: ObjectId) -> Vec4 {
        if id.0 < self.colors.len() {
            return self.colors[id.0];
        } else {
            return Vec4::ONE;
        }
    }
}
