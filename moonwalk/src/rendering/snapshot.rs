// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use glam::Vec2;

/// Я не могу возвращать Result у снапшотов так как api уже сформирован и снапшоты
/// обычно считаются безопасной операцией. Вместо этого лучше написать клиппер
/// который обрежет снапшот, а если что-то не так то сделает позицию 0, 0, а размер
/// равный всей поверхности
pub struct ClippedSnapshot {
    pub position: Vec2,
    pub size: Vec2,
}

impl ClippedSnapshot {
    pub fn new(position: Vec2, size: Vec2) -> Self {
        Self {
            position,
            size,
        }
    }

    /// Этот метод нужен для того, чтобы обрезать снапшот если он не помещается
    /// в исходный surface чтобы предотвратить панику Validation error от wgpu
    /// В крайнем случае возвращает всю поверхность как снапшот
    pub fn clip_snapshot(&mut self, source_size: Vec2) {
        // Проверка что размер и позциия не бесконечные, а если бесконечные
        // то возвразается весь surface
        if self.position.x.is_infinite()
            || self.position.y.is_infinite()
            || self.size.x.is_infinite()
            || self.size.y.is_infinite()
        {
            self.position = Vec2::ZERO;
            self.size = source_size;
            return;
        }

        // Проверка размера на ноль
        if self.size.x == 0.0 || self.size.y == 0.0 {
            self.size = source_size;
        }

        if self.size.x < 0.0 || self.size.y < 0.0 {
            self.size = source_size;
        }

        // Обрезание
        self.size.x = Self::clip(self.size.x, source_size.x);
        self.size.y = Self::clip(self.size.y, source_size.y);

        // Для позиций ставим ноль если за пределами экрана. Клипить это нормально
        // невозможно, поэтому лучше просто захватить с левого верхнего угла
        if self.position.x > source_size.x {
            self.position.x = 0.0;
        }

        if self.position.y > source_size.y {
            self.position.y = 0.0;
        }

        // Если позиция находится за размерами снапшота то нужно обрезать по размеру
        // снапшота и минус один чтобы не было 0, 0
        self.position.x = Self::clip(self.position.x, self.size.x - 1.0);
        self.position.y = Self::clip(self.position.y, self.size.y - 1.0);

        // Проверка на отрицательное число
        if self.position.x < 0.0 || self.position.y < 0.0 {
            self.position = Vec2::ZERO;
        }
    }

    fn clip(value: f32, source: f32) -> f32 {
        // Если x или y position выходят за границы source_size то мы берём
        // x/y, отнимаем от них source_size, сохраняем в delta и записываем
        // в position.x/y оригинал - delta
        if value > source {
            return source;
        }

        return value;
    }
}

#[test]
fn snapshot_clip_test() {
    let mut snapshot_region = ClippedSnapshot::new(
        Vec2::new(0.0, 0.0),
        Vec2::new(100.0, 100.0),
    );
    snapshot_region.clip_snapshot(Vec2::new(200.0, 200.0));

    println!("x: {}, y: {}, w: {}, h: {}", snapshot_region.position.x, snapshot_region.position.y,
        snapshot_region.size.x, snapshot_region.size.y);
    
    assert_eq!(snapshot_region.position.x, 0.0);
    assert_eq!(snapshot_region.position.y, 0.0);
    assert_eq!(snapshot_region.size.x, 100.0);
    assert_eq!(snapshot_region.size.y, 100.0);

    snapshot_region.clip_snapshot(Vec2::new(50.0, 50.0));
    
    println!("x: {}, y: {}, w: {}, h: {}", snapshot_region.position.x, snapshot_region.position.y,
        snapshot_region.size.x, snapshot_region.size.y);
    
    assert_eq!(snapshot_region.position.x, 0.0);
    assert_eq!(snapshot_region.position.y, 0.0);
    assert_eq!(snapshot_region.size.x, 50.0);
    assert_eq!(snapshot_region.size.y, 50.0);

    snapshot_region = ClippedSnapshot::new(
        Vec2::new(160.0, 160.0),
        Vec2::new(100.0, 100.0),
    );
    snapshot_region.clip_snapshot(Vec2::new(100.0, 100.0));

    println!("x: {}, y: {}, w: {}, h: {}", snapshot_region.position.x, snapshot_region.position.y,
        snapshot_region.size.x, snapshot_region.size.y);
    
    assert_eq!(snapshot_region.position.x, 0.0);
    assert_eq!(snapshot_region.position.y, 0.0);
    assert_eq!(snapshot_region.size.x, 100.0);
    assert_eq!(snapshot_region.size.y, 100.0);

    snapshot_region = ClippedSnapshot::new(
        Vec2::new(200.0, 200.0),
        Vec2::new(100.0, 100.0),
    );
    snapshot_region.clip_snapshot(Vec2::new(500.0, 500.0));

    println!("x: {}, y: {}, w: {}, h: {}", snapshot_region.position.x, snapshot_region.position.y,
        snapshot_region.size.x, snapshot_region.size.y);
    
    assert_eq!(snapshot_region.position.x, 99.0);
    assert_eq!(snapshot_region.position.y, 99.0);
    assert_eq!(snapshot_region.size.x, 100.0);
    assert_eq!(snapshot_region.size.y, 100.0);
}