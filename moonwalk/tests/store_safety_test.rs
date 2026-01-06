// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use moonwalk::objects::store::ObjectStore;
use moonwalk::objects::ObjectType;
use glam::Vec2;

#[test]
fn test_remove_twice() {
    let mut store = ObjectStore::new();
    let id = store.new_rect();
    
    store.remove(id);
    assert!(!store.alive[id.index()]);
    
    store.remove(id);
    assert!(!store.alive[id.index()]);
    
    assert_eq!(store.free_slots.len(), 1);
}

#[test]
fn test_getters_out_of_bounds() {
    // Эмуляция айди которого нет
    let mut store = ObjectStore::new();
    // Создаем айди с индексом 999
    let fake_id = moonwalk::objects::ObjectId::new(ObjectType::Rect, 999);
    
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        store.get_position(fake_id);
    })); 
}
