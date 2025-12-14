// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use glam::{Vec2, Vec4};

use crate::objects;
use crate::objects::{ObjectId, ObjectType};

/// Хранилище для объектов
pub struct ObjectStore {
    // Основные данные (SoA)
    pub positions: Vec<Vec2>,
    pub sizes: Vec<Vec2>,
    pub colors: Vec<Vec4>,

    // Второй цвет для использования в градиентах
    pub colors2: Vec<Vec4>,
    
    pub rotations: Vec<f32>,
    pub z_indices: Vec<f32>,

    // alive хранит жив ли объект. True - жив и отрисовыватся, false - мёртв
    pub alive: Vec<bool>,

    pub object_types: Vec<ObjectType>,
    
    // Айди объектов
    pub rect_ids: Vec<ObjectId>,
    
    // Данные специфичные для прямоугольника
    pub rect_radii: Vec<Vec4>,

    // Оптимизация: перерождение объектов. При удалении объекта его айди
    // добавляется в этот вектор и при следующем добавлении нового
    // объекта от может взять свой айди из этого вектора тем самым
    // не давая бесконечно расти и занимать ОЗУ
    pub free_slots: Vec<usize>,

    pub texture_ids: Vec<u32>,
    pub uvs: Vec<[f32; 4]>,

    pub dirty: bool,
}

impl ObjectStore {
    pub fn new() -> Self {
        Self {
            // Оптимизация: Сразу же даём капасити
            positions: Vec::with_capacity(1024),
            sizes: Vec::with_capacity(1024),
            colors: Vec::with_capacity(1024),
            colors2: Vec::with_capacity(1024),
            rotations: Vec::with_capacity(1024),
            z_indices: Vec::with_capacity(1024),
            alive: Vec::with_capacity(1024),
            object_types: Vec::with_capacity(1024),
            rect_ids: Vec::with_capacity(1024),
            rect_radii: Vec::with_capacity(1024),
            free_slots: Vec::with_capacity(128),
            texture_ids: Vec::with_capacity(1024),
            uvs: Vec::with_capacity(1024),

            // Объекты изначально не грязные потому-что их нет
            dirty: false,
        }
    }

    fn alloc_common(&mut self) -> usize {
        if let Some(idx) = self.free_slots.pop() {
            // Ставим дефольные данные. Тут небольшой дубляж кода, пока-что я оставляю
            // так, но позже желательно исправить и перейти на константы
            self.positions[idx] = Vec2::ZERO;
            self.sizes[idx] = Vec2::new(100.0, 100.0);
            self.colors[idx] = Vec4::ONE;
            self.colors2[idx] = Vec4::ONE;
            self.rotations[idx] = 0.0;
            self.z_indices[idx] = 0.0;
            self.alive[idx] = true;
            self.rect_radii[idx] = Vec4::ZERO;
            self.texture_ids[idx] = 0;
            self.uvs[idx] = [0.0, 0.0, 1.0, 1.0];
            self.dirty = true;
            
            return idx;
        }
        
        let index = self.positions.len();

        self.positions.push(Vec2::ZERO); // Нулевая позиция (Левый верхний угол)
        self.sizes.push(Vec2::new(100.0, 100.0)); // Позиция 100 на 100
        self.colors.push(Vec4::ONE); // Цвет белый (1, 1, 1, 1)
        self.colors2.push(Vec4::ONE); // Вторлой цвет тоже белый (1, 1, 1, 1)
        self.rotations.push(0.0); // Вращение: 0.0 радиан
        self.z_indices.push(0.0); // Нулевой z индекс
        self.alive.push(true);
        self.rect_radii.push(Vec4::ZERO);
        self.uvs.push([0.0, 0.0, 1.0, 1.0]);
        self.object_types.push(ObjectType::Unknown);
        self.texture_ids.push(0);

        // После создания объекта нам нужно пересобрать всё, поэтому
        // делаем хранилище грязным
        self.dirty = true;

        index
    }

    pub fn new_rect(&mut self) -> ObjectId {
        // Делаем аллокацию
        let index = self.alloc_common();
        let id = objects::ObjectId::new(objects::ObjectType::Rect, index);

        // Если это не rect - добавляем труп в rect_ids и даём ему тип rect 
        if self.object_types[index] != ObjectType::Rect {
            self.rect_ids.push(id);
            self.object_types[index] = ObjectType::Rect;
        }

        id
    }

    pub fn remove(&mut self, id: ObjectId) {
        let idx = id.index();
        
        if idx < self.alive.len() {
            // Если объект был жив, и мы его убиваем - ставим дирти,
            // чтобы перерисовать кадр без него
            if self.alive[idx] {
                self.alive[idx] = false;
                self.dirty = true;

                // После смерти добавляем объект га кладбище откуда труп
                // будут перерождёе для другого объекта не давая векторам
                // бесконечно расти забивая оперативку
                self.free_slots.push(idx);
            }
        }
    }

    /// Каждая функция конфигурации должна делать хранилище объектов
    /// грязным чтобы пересобрать всё

    #[inline(always)]
    pub fn config_position(&mut self, id: ObjectId, pos: Vec2) {
        self.positions[id.index()] = pos;
        self.dirty = true;
    }

    #[inline(always)]
    pub fn config_size(&mut self, id: ObjectId, size: Vec2) {
        self.sizes[id.index()] = size;
        self.dirty = true;
    }

    #[inline(always)]
    pub fn config_color(&mut self, id: ObjectId, color: Vec4) {
        self.colors[id.index()] = color;
        self.dirty = true;
    }
    
    #[inline(always)]
    pub fn config_rotation(&mut self, id: ObjectId, rad: f32) {
        self.rotations[id.index()] = rad;
        self.dirty = true;
    }

    #[inline(always)]
    pub fn config_z_index(&mut self, id: ObjectId, z: f32) {
        self.z_indices[id.index()] = z;
        self.dirty = true;
    }

    #[inline(always)]
    pub fn config_uv(&mut self, id: ObjectId, uv: [f32; 4]) {
        self.uvs[id.index()] = uv; 
        self.dirty = true;
    }

    #[inline(always)]
    pub fn set_rounded(&mut self, id: ObjectId, radii: Vec4) {
        if id.index() < self.rect_radii.len() {
             self.rect_radii[id.index()] = radii;
             self.dirty = true;
        }
    }

    #[inline(always)]
    pub fn config_texture(&mut self, id: ObjectId, texture_id: u32) {
        self.texture_ids[id.index()] = texture_id;
        self.dirty = true;
    }
}