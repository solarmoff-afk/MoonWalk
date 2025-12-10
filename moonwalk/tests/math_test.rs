// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use moonwalk::rendering::vertex::ObjectInstance;

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