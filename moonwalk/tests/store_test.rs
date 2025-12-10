// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use glam::{Vec2, Vec4};
use std::time::Instant;
use moonwalk::objects::ObjectType;
use moonwalk::objects::store::ObjectStore;

#[test]
fn test_rect_creation_and_config() {
    // Инициализация
    let mut store = ObjectStore::new();
    let id = store.new_rect();

    // Проверка типа
    assert_eq!(id.get_type(), Some(ObjectType::Rect));

    // Проверка индекса и начального состояния
    let idx = id.index();
    assert_eq!(store.positions.len(), 1);
    
    assert_eq!(store.positions[idx], Vec2::ZERO);
    assert!(store.dirty);

    // Изменение данных
    let new_pos = Vec2::new(50.0, 100.0);
    store.config_position(id, new_pos);
    
    assert_eq!(store.positions[idx], new_pos);
    
    // Проверка цвета
    let new_color = Vec4::new(1.0, 0.0, 0.0, 1.0);
    store.config_color(id, new_color);
    assert_eq!(store.colors[idx], new_color);
}

#[test]
fn test_z_sorting_flag() {
    let mut store = ObjectStore::new();
    let id = store.new_rect();
    
    store.dirty = false;

    store.config_z_index(id, 10.0);
    
    assert!(store.dirty);
    assert_eq!(store.z_indices[id.index()], 10.0);
}

#[test]
fn test_soa_consistency_stress() {
    let mut store = ObjectStore::new();
    const COUNT: usize = 10_000;
    
    let mut ids = Vec::with_capacity(COUNT);

    for i in 0..COUNT {
        let id = store.new_rect();
        ids.push(id);
        
        let val = i as f32;
        store.config_position(id, Vec2::new(val, val * 2.0));
        store.config_z_index(id, val);
    }

    assert_eq!(store.positions.len(), COUNT);
    assert_eq!(store.z_indices.len(), COUNT);
    assert_eq!(store.colors.len(), COUNT);
    assert_eq!(store.sizes.len(), COUNT);
    assert_eq!(store.rotations.len(), COUNT);
    assert_eq!(store.rect_ids.len(), COUNT);

    for (i, &id) in ids.iter().enumerate() {
        let idx = id.index();
        let expected = i as f32;

        assert_eq!(store.positions[idx].x, expected, "Position X mismatch at index {}", i);
        assert_eq!(store.positions[idx].y, expected * 2.0, "Position Y mismatch at index {}", i);
        assert_eq!(store.z_indices[idx], expected, "Z-index mismatch at index {}", i);
    }
}

#[test]
fn test_performance_allocation() {
    let mut store = ObjectStore::new();
    const COUNT: usize = 100_000;
    
    let start = Instant::now();
    
    for _ in 0..COUNT {
        store.new_rect();
    }
    
    let duration = start.elapsed();
    println!("Allocation of {} objects took: {:?}", COUNT, duration);
    
    assert!(duration.as_millis() < 500); 
}

#[test]
fn test_reincarnation_memory_reuse() {
    let mut store = ObjectStore::new();
    const INITIAL_COUNT: usize = 100;
    
    let mut ids = Vec::new();

    // Создание 100 объектов
    for _ in 0..INITIAL_COUNT {
        ids.push(store.new_rect());
    }

    assert_eq!(store.positions.len(), INITIAL_COUNT);
    assert_eq!(store.free_slots.len(), 0);

    // Удаляем всё
    for id in ids {
        store.remove(id);
    }

    // Слотов всё еще 100 (память не освобождается, а помечается)
    assert_eq!(store.positions.len(), INITIAL_COUNT);

    // Но теперь у нас 100 свободных маест
    assert_eq!(store.free_slots.len(), INITIAL_COUNT);

    // Создаем 50 новых объектов (Реинкарнация)
    let mut new_ids = Vec::new();
    for i in 0..50 {
        let id = store.new_rect();
        new_ids.push(id);
        
        // Меняем данные чтобы проверить сброс
        store.config_position(id, Vec2::new(i as f32, 0.0));
    }

    // Длина векторов не должна увеличиться. Мы должны были использовать трупы
    assert_eq!(store.positions.len(), INITIAL_COUNT);
    assert_eq!(store.free_slots.len(), 50);
    
    // Проверяем что новые объекты живы и имеют правильные данные
    let reused_idx = new_ids[0].index();
    assert!(store.alive[reused_idx]);
    assert_eq!(store.positions[reused_idx], Vec2::new(0.0, 0.0));
}