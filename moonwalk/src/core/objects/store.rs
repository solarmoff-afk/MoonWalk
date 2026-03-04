// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

#![allow(unused_imports)]

use glam::{Vec2, Vec3, Vec4, Mat4, Quat};

use crate::core::objects;
use crate::core::objects::{ObjectId, ObjectType, ObjectDirty};
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

    // Определяет грязность стора, то есть изменились ли объекты
    pub dirty: bool,

    // Для оптимизации батчинга будем перегенерировать команды только для
    // объектов которые изменились, для этого собираем вектор где false 
    // означает что изменений нет, а true значит что было изменение. Батчинг
    // обязуется лично снять этот флаг после того как перезаписал команду
    // для этого объекта
    pub dirty_objects: Vec<ObjectDirty>,

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
    pub font_ids: Vec<crate::draw::text::FontId>,
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

            dirty_objects: Vec::with_capacity(1024),

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
            self.dirty_objects[idx] = ObjectDirty::new();

            // Для кэша сжатых значений
            self.colors_cache[idx] = ObjectInstance::pack_color(Vec4::ONE.to_array());
            self.colors2_cache[idx] = ObjectInstance::pack_color(Vec4::ONE.to_array());
            self.rect_radii_cache[idx] = ObjectInstance::pack_radii(Vec4::ZERO.to_array());
            self.uvs_cache[idx] = ObjectInstance::pack_uv([0.0, 0.0, 1.0, 1.0]);
            self.gradient_data_cache[idx] = ObjectInstance::pack_gradient([0.0, 0.0, -1.0, 0.0]);
            self.effect_data_cache[idx] = ObjectInstance::pack_effects(0.0, 0.0);

            self.text_contents[idx].clear();
            self.font_ids[idx] = crate::draw::text::FontId(0);
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
        self.dirty_objects.push(ObjectDirty::new());

        // Для кэша сжатых значений
        self.colors_cache.push(ObjectInstance::pack_color(Vec4::ONE.to_array()));
        self.colors2_cache.push(ObjectInstance::pack_color(Vec4::ONE.to_array()));
        self.rect_radii_cache.push(ObjectInstance::pack_radii(Vec4::ZERO.to_array()));
        self.uvs_cache.push(ObjectInstance::pack_uv([0.0, 0.0, 1.0, 1.0]));
        self.gradient_data_cache.push(ObjectInstance::pack_gradient([0.0, 0.0, -1.0, 0.0]));
        self.effect_data_cache.push(ObjectInstance::pack_effects(0.0, 0.0));

        self.text_contents.push(String::new());
        self.font_ids.push(crate::draw::text::FontId(0));
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

    pub fn new_text(&mut self, text: String, font_id: crate::draw::text::FontId, font_size: f32) -> ObjectId {
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
            self.dirty_objects[idx].update();
        }
    }

    #[inline(always)]
    pub fn set_font_size(&mut self, id: ObjectId, size: f32) {
        let idx = id.index();

        self.font_sizes[idx] = size;
        self.dirty = true;
        self.dirty_objects[idx].update();
    }

    #[inline(always)]
    pub fn set_text_bounds(&mut self, id: ObjectId, w: f32, h: f32) {
        let idx = id.index();

        self.text_bounds[idx] = Vec2::new(w, h);
        self.dirty = true;
        self.dirty_objects[idx].update();
    }

    pub fn remove(&mut self, id: ObjectId) {
        let idx = id.index();
        
        if idx < self.alive.len() {
            // Если объект был жив, и мы его убиваем - ставим дирти,
            // чтобы перерисовать кадр без него
            if self.alive[idx] {
                self.alive[idx] = false;
                self.dirty = true;
                self.dirty_objects[idx].update();

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
        let idx = id.index();

        self.positions[idx] = pos;
        self.dirty = true;
        self.dirty_objects[idx].update();
    }

    #[inline(always)]
    pub fn config_size(&mut self, id: ObjectId, size: Vec2) {
        let idx = id.index();

        self.sizes[idx] = size;
        self.dirty = true;
        self.dirty_objects[idx].update();
    }

    #[inline(always)]
    pub fn config_color(&mut self, id: ObjectId, color: Vec4) {
        let idx = id.index();

        self.colors[idx] = color;
        self.colors_cache[id.index()] = ObjectInstance::pack_color(color.to_array());
        self.dirty = true;
        self.dirty_objects[idx].update();
    }

    #[inline(always)]
    pub fn config_color2(&mut self, id: ObjectId, color2: Vec4) {
        let idx = id.index();

        self.colors2[idx] = color2;
        self.colors2_cache[id.index()] = ObjectInstance::pack_color(color2.to_array());
        self.dirty = true;
        self.dirty_objects[idx].update();
    }
    
    #[inline(always)]
    pub fn config_rotation(&mut self, id: ObjectId, rad: f32) {
        let idx = id.index();

        self.rotations[idx] = rad;
        self.dirty = true;
        self.dirty_objects[idx].update();
    }

    #[inline(always)]
    pub fn config_z_index(&mut self, id: ObjectId, z: f32) {
        let idx = id.index();

        self.z_indices[idx] = z;
        self.dirty = true;
        self.dirty_objects[idx].update();
    }

    #[inline(always)]
    pub fn config_uv(&mut self, id: ObjectId, uv: [f32; 4]) {
        let idx = id.index();

        self.uvs[id.index()] = uv; 
        self.uvs_cache[idx] = ObjectInstance::pack_uv(uv);
        self.dirty = true;
        self.dirty_objects[idx].update();
    }

    #[inline(always)]
    pub fn set_rounded(&mut self, id: ObjectId, radii: Vec4) {
        let idx = id.index();

        if id.index() < self.rect_radii.len() {
             self.rect_radii[idx] = radii;
             self.rect_radii_cache[idx] = ObjectInstance::pack_radii(radii.to_array());
             self.dirty = true;
             self.dirty_objects[idx].update();
        }
    }

    #[inline(always)]
    pub fn config_texture(&mut self, id: ObjectId, texture_id: u32) {
        let idx = id.index();

        self.texture_ids[idx] = texture_id;
        self.dirty = true;
        self.dirty_objects[idx].update();
    }

    #[inline(always)]
    pub fn config_gradient_data(&mut self, id: ObjectId, gradient_data: [f32; 4]) {
        let idx = id.index();

        self.gradient_data[idx] = gradient_data;
        self.gradient_data_cache[idx] = ObjectInstance::pack_gradient(
            gradient_data
        );
        self.dirty_objects[idx].update();

        self.dirty = true;
    }

    #[inline(always)]
    pub fn config_effect_data(&mut self, id: ObjectId, effect_data: [f32; 2]) {
        let idx = id.index();

        self.effect_data[idx] = effect_data;
        self.effect_data_cache[idx] = ObjectInstance::pack_effects(
            effect_data[0], effect_data[1]
        );
        self.dirty_objects[idx].update();

        self.dirty = true;
    }

    #[inline(always)]
    pub fn set_text_align(&mut self, id: ObjectId, align: u8) {
        let idx = id.index();

        if self.text_aligns[idx] != align {
            self.text_aligns[idx] = align;
            self.dirty = true;
            self.dirty_objects[idx].update();
        }
    }

    #[inline(always)]
    pub fn set_hit_group(&mut self, id: ObjectId, group: u16) {
        let idx = id.index();

        if self.hit_groups[idx] != group {
            self.hit_groups[idx] = group;
            self.dirty = true;
            self.dirty_objects[idx].update();
        }
    }

    #[inline(always)]
    pub fn resolve_hit(&self, position: Vec2, size: Vec2, target_group: u16) -> Option<ObjectId> {
        let test_min = position;
        let test_max = position + size;

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

    // Обёртка
    #[inline(always)]
    pub fn decompose_matrix(&self, matrix: Mat4) -> (Vec2, f32, Vec2) {
        decompose_matrix(matrix)
    }
}

/// Эта функция позволяет преобразовать mat4 в полноценную позицию, вращение и размер
/// это используется чтобы не передавать матрицу напрямую в шейдер
pub fn decompose_matrix(matrix: Mat4) -> (Vec2, f32, Vec2) {
    // Позиция это просто последний столбец
    let position = Vec2::new(matrix.w_axis.x, matrix.w_axis.y);
    
    // Базисные векторы
    let x_basis = Vec2::new(matrix.x_axis.x, matrix.x_axis.y);
    let y_basis = Vec2::new(matrix.y_axis.x, matrix.y_axis.y);
    
    let rotation = f32::atan2(x_basis.y, x_basis.x);
    
    let size = Vec2::new(x_basis.length(), y_basis.length());
    
    (position, rotation, size)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Этот тест тестирует создание и удаление объектов в ObjectSotre
    #[test]
    fn object_pool() {
        let mut store = ObjectStore::new();

        // Создадим 3 объекта
        let rect = store.new_rect();
        store.new_rect();
        store.new_rect();

        // Длина должна быть 3, так как тут три объекта
        assert_eq!(store.rect_ids.len(), 3);

        // Далее удаляем один объект
        store.remove(rect);

        // Должен быть один свободный слот
        assert_eq!(store.free_slots.len(), 1);

        // Создаём новый объект
        let rect2 = store.new_rect();

        // Rect2 должен занять айди старого rect
        assert_eq!(rect, rect2);
    }

    /// Этот тест тестирует хит тесты для объектов
    #[test]
    fn object_hit_test() {
        let mut store = ObjectStore::new();

        // Объект для теста
        let rect = store.new_rect();

        store.config_position(rect, Vec2::new(150.0, 150.0));
        store.config_size(rect, Vec2::new(50.0, 50.0));

        // Первая хит группа для теста
        store.set_hit_group(rect, 1);

        // Первая проверка, объекты сталкиваются
        assert_eq!(store.resolve_hit(Vec2::new(125.0, 125.0), Vec2::new(50.0, 50.0), 1), Some(rect));

        // Тут объекты не сталкиваются
        assert_eq!(store.resolve_hit(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0), 1), None);

        // Тут сталкиваются
        assert_eq!(store.resolve_hit(Vec2::new(0.0, 0.0), Vec2::new(5000.0, 5000.0), 1), Some(rect));
    }

    /// Этот тест тестирует хит тесты для объектов с учётом z индекса
    #[test]
    fn object_hit_test_z_index() {
        let mut store = ObjectStore::new();

        // Объект пустышка
        let rect = store.new_rect();

        // Объект для теста
        let rect2 = store.new_rect();

        store.config_position(rect, Vec2::new(150.0, 150.0));
        store.config_size(rect, Vec2::new(50.0, 50.0));
        store.config_z_index(rect, 0.1);

        store.config_position(rect2, Vec2::new(150.0, 150.0));
        store.config_size(rect2, Vec2::new(50.0, 50.0));
        store.config_z_index(rect2, 0.5);

        // Первая хит группа для теста для обоих объектов
        store.set_hit_group(rect, 1);
        store.set_hit_group(rect2, 1);

        // Тесты должны пройти для второго объекта, так как он выше по z индексу

        // Первая проверка, объекты сталкиваются
        assert_eq!(store.resolve_hit(Vec2::new(125.0, 125.0), Vec2::new(50.0, 50.0), 1), Some(rect2));

        // Тут объекты не сталкиваются
        assert_eq!(store.resolve_hit(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0), 1), None);

        // Тут сталкиваются
        assert_eq!(store.resolve_hit(Vec2::new(0.0, 0.0), Vec2::new(5000.0, 5000.0), 1), Some(rect2));
    }

    /// Этот тест тестирует геттеры
    #[test]
    fn object_getter_test() {
        let mut store = ObjectStore::new();

        // Объект для теста
        let rect = store.new_rect();

        // Тут задаются тестовые поля 
        store.config_position(rect, Vec2::new(150.0, 150.0));
        store.config_size(rect, Vec2::new(150.0, 150.0));

        // Объект должен быть живым
        assert!(store.is_alive(rect));

        // Геттеры должны вернуть эти поля
        assert_eq!(store.get_position(rect), Vec2::new(150.0, 150.0));
        assert_eq!(store.get_size(rect), Vec2::new(150.0, 150.0));

        // Удаление
        store.remove(rect);

        // Объект должен умереть
        assert_eq!(store.is_alive(rect), false);
    }

    // Тесты для расчленения матрицы

    #[test]
    fn test_identity_matrix() {
        let m = Mat4::IDENTITY;
        let (pos, rot, size) = decompose_matrix(m);
        
        assert_eq!(pos, Vec2::ZERO);
        assert_eq!(rot, 0.0);
        assert_eq!(size, Vec2::ONE);
    }

    /// Этот тест тестирует то, что если разобрать и собрать матрицу то должна выйти
    /// точно такая же матрица. Это проверяет то что логика разборки на части не нарушена
    /// и выдаёт валидный результат
    #[test]
    fn test_round_trip() {
        let original_pos = Vec2::new(123.0, 456.0);
        let original_rot = 67.0_f32.to_radians();
        let original_size = Vec2::new(3.0, 7.0);
        
        // Создаем оригинальную матрицу
        let original = Mat4::from_scale_rotation_translation(
            Vec3::new(original_size.x, original_size.y, 1.0),
            Quat::from_rotation_z(original_rot),
            Vec3::new(original_pos.x, original_pos.y, 0.0)
        );
        
        let (pos, rot, size) = decompose_matrix(original);
        
        assert!((pos.x - original_pos.x).abs() < 1e-6);
        assert!((pos.y - original_pos.y).abs() < 1e-6);
        assert!((rot - original_rot).abs() < 1e-6);
        assert!((size.x - original_size.x).abs() < 1e-6);
        assert!((size.y - original_size.y).abs() < 1e-6);
    }
}
