// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use moonwalk::rendering::vertex::ObjectInstance;
use glam::Vec2;
use moonwalk::gpu::MatrixStack;
use moonwalk::objects::store::ObjectStore;

#[test]
fn test_color_packing_u32() {
    // Красный r = 1.0 (255), g = 0,b = 0, a = 1.0 (255)
    // поэтому ожидаем 0xAABBGGRR в 0xFF0000FF
    let red_vec = [1.0, 0.0, 0.0, 1.0];
    let packed = ObjectInstance::pack_color(red_vec);
    
    assert_eq!(packed, 0xFF0000FF);
    
    // Полупрозрачный зеленый r = 0, g = 1 (255), b = 0, a = 0.5 (127 = 0x7F)
    // поэтому ожидаев 0x7F00FF00
    let green_vec = [0.0, 1.0, 0.0, 0.5];
    let packed_green = ObjectInstance::pack_color(green_vec);
    
    assert_eq!(packed_green, 0x7F00FF00);
}

#[test]
fn test_ortho_projection_logic() {
    let mut stack = MatrixStack::new();
    // Экран 800 на 600
    stack.set_ortho(800.0, 600.0);
    
    // Матрица не должна быть identity
    assert_ne!(stack.projection, glam::Mat4::IDENTITY);
}

#[test]
fn test_collision_detection_layers() {
    let mut store = ObjectStore::new();
    
    // Объект 1 фон
    let bg = store.new_rect();
    store.config_position(bg, Vec2::new(100.0, 100.0));
    store.config_size(bg, Vec2::new(200.0, 200.0)); // Центр 100 + 100=200, 100 + 100=200
    store.config_z_index(bg, 0.1);
    store.set_hit_group(bg, 1);

    // Объект 2 кнопка
    let btn = store.new_rect();
    store.config_position(btn, Vec2::new(150.0, 150.0)); // Внутри фона
    store.config_size(btn, Vec2::new(50.0, 50.0));
    store.config_z_index(btn, 0.9); // Выше
    store.set_hit_group(btn, 1);

    // Тест 1 клик в кнопку (должна вернуть btn, т.к. z больше)
    // Центр кнопки 150 + 25 = 175
    let hit = store.resolve_hit(Vec2::new(175.0, 175.0), Vec2::new(1.0, 1.0), 1);
    assert_eq!(hit, Some(btn));

    // Тест 2 клик в фон (мимо кнопки)
    // Точка 110, 110 (внутри фона, но левее кнопки)
    let hit_bg = store.resolve_hit(Vec2::new(110.0, 110.0), Vec2::new(1.0, 1.0), 1);
    assert_eq!(hit_bg, Some(bg));

    // Тест 3 другая группа
    let hit_wrong_group = store.resolve_hit(Vec2::new(175.0, 175.0), Vec2::new(1.0, 1.0), 2);
    assert_eq!(hit_wrong_group, None);
}
