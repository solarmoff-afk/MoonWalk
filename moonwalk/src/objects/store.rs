// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use glam::{Vec2, Vec4};

use crate::objects;
use crate::objects::{ObjectId, ObjectType};
use crate::rendering::vertex::ObjectInstance;

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

    // Данные специфичные для текстов
    pub text_aligns: Vec<u8>,

    // Оптимизация: перерождение объектов. При удалении объекта его айди
    // добавляется в этот вектор и при следующем добавлении нового
    // объекта от может взять свой айди из этого вектора тем самым
    // не давая бесконечно расти и занимать ОЗУ
    pub free_slots: Vec<usize>,

    pub texture_ids: Vec<u32>,
    pub uvs: Vec<[f32; 4]>,

    // Параметры градиента для объекта. Формат данных:
    //  [x, y, z, w]
    // Использование:
    //  - Если z меньше чем 0.0 - градиента нет. Это значение по-умолчанию
    //  - если z и w равны 0.0 - это линейный градиент. (z и w это радиус, а радиус не
    //     может быть отрицательным)
    //  - если z больше или равен 0.0 и w больше чем z - это радиальный градиент. То есть
    //    круговой переход от центра к краю. X и y в его случае это центр, а z и h это
    //    внутренний и внешний радиус.
    // [!] Все индексы (x, y, z и w) указаываются от 0 до 1 
    pub gradient_data: Vec<[f32; 4]>,

    // Параметры эффектов, тут задаются данные обводки (1 индекс) и box-shadow
    // вторым индексом
    pub effect_data: Vec<[f32; 2]>,

    pub dirty: bool,

    // Существует проблема с лимитом в 86 (или на старых устройствах 64) байта 
    // на размер данных вершины. Эти ограничения устанавливаются судя по всему 
    // самим драйверов графического процессора, поэтому следовать им обязательно.
    // Так как все необходимые атрибуты занимают больше чем 86 байт необходимо
    // их сжимать через функции упаковки. Эти функции выливаются в потерю
    // производительности при частом вызове для большого количества объектов,
    // а вызываются они при перестройке батча. Поэтому необходимо создать
    // вектора для кэширования значения функции сжатия атрибута и вызывать
    // только при изменении параметра который отвечает за этот атрибут
    pub colors_cache: Vec<u32>,
    pub colors2_cache: Vec<u32>,
    pub rect_radii_cache: Vec<[u16; 4]>,
    pub uvs_cache: Vec<[u16; 4]>,
    pub gradient_data_cache: Vec<[i16; 4]>,
    pub effect_data_cache: Vec<[u16; 2]>,

    pub text_ids: Vec<ObjectId>,
    pub text_contents: Vec<String>,
    pub font_ids: Vec<crate::textware::FontId>,
    pub font_sizes: Vec<f32>,
    pub text_bounds: Vec<Vec2>,

    // Hit группы для коллизий
    pub hit_groups: Vec<u16>,
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
            text_aligns: Vec::with_capacity(128),
            free_slots: Vec::with_capacity(128),
            texture_ids: Vec::with_capacity(1024),
            uvs: Vec::with_capacity(1024),
            gradient_data: Vec::with_capacity(1024),
            effect_data: Vec::with_capacity(1024),

            // Для кэша сжатых значений
            colors_cache: Vec::with_capacity(1024),
            colors2_cache: Vec::with_capacity(1024),
            rect_radii_cache: Vec::with_capacity(1024),
            uvs_cache: Vec::with_capacity(1024),
            gradient_data_cache: Vec::with_capacity(1024),
            effect_data_cache: Vec::with_capacity(1024),

            // Текстов обычно меньше чем прямоугольников
            text_ids: Vec::with_capacity(128),
            text_contents: Vec::with_capacity(128),
            font_ids: Vec::with_capacity(128),
            font_sizes: Vec::with_capacity(128),
            text_bounds: Vec::with_capacity(128),

            // Объекты изначально не грязные потому-что их нет
            dirty: false,

            hit_groups: Vec::with_capacity(1024),
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
            self.text_aligns[idx] = 0;
            self.texture_ids[idx] = 0;
            self.uvs[idx] = [0.0, 0.0, 1.0, 1.0];

            // Нет градиента, так как радиус (z) отрицательный
            self.gradient_data[idx] = [0.0, 0.0, -1.0, 0.0];

            self.effect_data[idx] = [0.0, 0.0];
            
            self.dirty = true;

            // Для кэша сжатых значений
            self.colors_cache[idx] = ObjectInstance::pack_color(Vec4::ONE.to_array());
            self.colors2_cache[idx] = ObjectInstance::pack_color(Vec4::ONE.to_array());
            self.rect_radii_cache[idx] = ObjectInstance::pack_radii(Vec4::ZERO.to_array());
            self.uvs_cache[idx] = ObjectInstance::pack_uv([0.0, 0.0, 1.0, 1.0]);
            self.gradient_data_cache[idx] = ObjectInstance::pack_gradient([0.0, 0.0, -1.0, 0.0]);
            self.effect_data_cache[idx] = ObjectInstance::pack_effects(0.0, 0.0);

            self.text_contents[idx].clear();
            self.font_ids[idx] = crate::textware::FontId(0);
            self.font_sizes[idx] = 0.0;
            self.text_bounds[idx] = Vec2::new(9999.0, 9999.0);

            // Hit группа по умолчанию
            self.hit_groups[idx] = 0;

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
        self.text_aligns.push(0);
        self.uvs.push([0.0, 0.0, 1.0, 1.0]);
        self.gradient_data.push([0.0, 0.0, -1.0, 0.0]);
        self.effect_data.push([0.0, 0.0]);
        self.object_types.push(ObjectType::Unknown);
        self.texture_ids.push(0);

        // После создания объекта нам нужно пересобрать всё, поэтому
        // делаем хранилище грязным
        self.dirty = true;

        // Для кэша сжатых значений
        self.colors_cache.push(ObjectInstance::pack_color(Vec4::ONE.to_array()));
        self.colors2_cache.push(ObjectInstance::pack_color(Vec4::ONE.to_array()));
        self.rect_radii_cache.push(ObjectInstance::pack_radii(Vec4::ZERO.to_array()));
        self.uvs_cache.push(ObjectInstance::pack_uv([0.0, 0.0, 1.0, 1.0]));
        self.gradient_data_cache.push(ObjectInstance::pack_gradient([0.0, 0.0, -1.0, 0.0]));
        self.effect_data_cache.push(ObjectInstance::pack_effects(0.0, 0.0));

        self.text_contents.push(String::new());
        self.font_ids.push(crate::textware::FontId(0));
        self.font_sizes.push(0.0);
        self.text_bounds.push(Vec2::new(9999.0, 9999.0));

        self.hit_groups.push(0);

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

    pub fn new_text(&mut self, text: String, font_id: crate::textware::FontId, font_size: f32) -> ObjectId {
        let index = self.alloc_common();
        let id = objects::ObjectId::new(objects::ObjectType::Text, index);

        if self.object_types[index] != ObjectType::Text {
            self.text_ids.push(id);
            self.object_types[index] = ObjectType::Text;
        }

        self.text_contents[index] = text;
        self.font_ids[index] = font_id;
        self.font_sizes[index] = font_size;
        
        self.dirty = true;
        id
    }

    #[inline(always)]
    pub fn set_text(&mut self, id: ObjectId, text: String) {
        let idx = id.index();
        
        if self.text_contents[idx] != text {
            self.text_contents[idx] = text;
            self.dirty = true;
        }
    }

    #[inline(always)]
    pub fn set_font_size(&mut self, id: ObjectId, size: f32) {
        self.font_sizes[id.index()] = size;
        self.dirty = true;
    }

    #[inline(always)]
    pub fn set_text_bounds(&mut self, id: ObjectId, w: f32, h: f32) {
        self.text_bounds[id.index()] = Vec2::new(w, h);
        self.dirty = true;
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
        self.colors_cache[id.index()] = ObjectInstance::pack_color(color.to_array());
        self.dirty = true;
    }

    #[inline(always)]
    pub fn config_color2(&mut self, id: ObjectId, color2: Vec4) {
        self.colors2[id.index()] = color2;
        self.colors2_cache[id.index()] = ObjectInstance::pack_color(color2.to_array());
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
        self.uvs_cache[id.index()] = ObjectInstance::pack_uv(uv);
        self.dirty = true;
    }

    #[inline(always)]
    pub fn set_rounded(&mut self, id: ObjectId, radii: Vec4) {
        if id.index() < self.rect_radii.len() {
             self.rect_radii[id.index()] = radii;
             self.rect_radii_cache[id.index()] = ObjectInstance::pack_radii(radii.to_array());
             self.dirty = true;
        }
    }

    #[inline(always)]
    pub fn config_texture(&mut self, id: ObjectId, texture_id: u32) {
        self.texture_ids[id.index()] = texture_id;
        self.dirty = true;
    }

    #[inline(always)]
    pub fn config_gradient_data(&mut self, id: ObjectId, gradient_data: [f32; 4]) {
        self.gradient_data[id.index()] = gradient_data;
        self.gradient_data_cache[id.index()] = ObjectInstance::pack_gradient(
            gradient_data
        );

        self.dirty = true;
    }

    #[inline(always)]
    pub fn config_effect_data(&mut self, id: ObjectId, effect_data: [f32; 2]) {
        self.effect_data[id.index()] = effect_data;
        self.effect_data_cache[id.index()] = ObjectInstance::pack_effects(
            effect_data[0], effect_data[1]
        );

        self.dirty = true;
    }

    #[inline(always)]
    pub fn set_text_align(&mut self, id: ObjectId, align: u8) {
        let idx = id.index();
        if self.text_aligns[idx] != align {
            self.text_aligns[idx] = align;
            self.dirty = true;
        }
    }

    #[inline(always)]
    pub fn set_hit_group(&mut self, id: ObjectId, group: u16) {
        let idx = id.index();
        if self.hit_groups[idx] != group {
            self.hit_groups[idx] = group;
            self.dirty = true;
        }
    }

    #[inline(always)]
    pub fn resolve_hit(&self, position: Vec2, size: Vec2, target_group: u16) -> Option<ObjectId> {
        let half_size = size * 0.5;
        let test_min = position - half_size;
        let test_max = position + half_size;

        let mut best_candidate: Option<(usize, f32)> = None;

        for (idx, alive) in self.alive.iter().enumerate() {
            if !alive {
                continue;
            }

            if self.hit_groups[idx] != target_group {
                continue;
            }

            let obj_size = self.sizes[idx];
            let obj_pos = self.positions[idx];
            let obj_min = obj_pos;
            let obj_max = obj_pos + obj_size;

            if test_max.x > obj_min.x && 
            test_min.x < obj_max.x && 
            test_max.y > obj_min.y && 
            test_min.y < obj_max.y {
                let z_index = self.z_indices[idx];
                
                match best_candidate {
                    None => best_candidate = Some((idx, z_index)),
                    Some((_, best_z)) if z_index > best_z => {
                        best_candidate = Some((idx, z_index))
                    },
                    _ => {}
                }
            }
        }

        best_candidate.map(|(idx, _)| {
            let object_type = self.object_types[idx];
            objects::ObjectId::new(object_type, idx)
        })
    }

    // Геттеры
    
    #[inline(always)]
    pub fn get_position(&self, id: ObjectId) -> Vec2 {
        self.positions[id.index()]
    }

    #[inline(always)]
    pub fn get_size(&self, id: ObjectId) -> Vec2 {
        self.sizes[id.index()]
    }

    #[inline(always)]
    pub fn get_rotation(&self, id: ObjectId) -> f32 {
        self.rotations[id.index()]
    }

    #[inline(always)]
    pub fn get_color(&self, id: ObjectId) -> Vec4 {
        self.colors[id.index()]
    }

    #[inline(always)]
    pub fn get_color2(&self, id: ObjectId) -> Vec4 {
        self.colors2[id.index()]
    }

    #[inline(always)]
    pub fn get_z_index(&self, id: ObjectId) -> f32 {
        self.z_indices[id.index()]
    }

    #[inline(always)]
    pub fn get_hit_group(&self, id: ObjectId) -> u16 {
        self.hit_groups[id.index()]
    }

    #[inline(always)]
    pub fn get_rounded(&self, id: ObjectId) -> Vec4 {
        // Проверка на всякий случай если вектор еще не вырос
        if id.index() < self.rect_radii.len() {
            self.rect_radii[id.index()]
        } else {
            Vec4::ZERO
        }
    }

    #[inline(always)]
    pub fn get_text(&self, id: ObjectId) -> &str {
        if id.index() < self.text_contents.len() {
            &self.text_contents[id.index()]
        } else {
            ""
        }
    }

    #[inline(always)]
    pub fn get_font_size(&self, id: ObjectId) -> f32 {
        if id.index() < self.font_sizes.len() {
            self.font_sizes[id.index()]
        } else {
            0.0
        }
    }

    #[inline(always)]
    pub fn get_text_bounds(&self, id: ObjectId) -> Vec2 {
        if id.index() < self.text_bounds.len() {
            self.text_bounds[id.index()]
        } else {
            Vec2::ZERO
        }
    }

    #[inline(always)]
    pub fn get_text_align(&self, id: ObjectId) -> u8 {
        if id.index() < self.text_aligns.len() {
            self.text_aligns[id.index()]
        } else {
            0
        }
    }
    
    /// Метод проверяет жив ли сейчас объект по айди
    #[inline(always)]
    pub fn is_alive(&self, id: ObjectId) -> bool {
        let idx = id.index();
        if idx < self.alive.len() {
            self.alive[idx]
        } else {
            false
        }
    }
}
