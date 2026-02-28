// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use glam::Vec4;

use crate::TextureId;

/// Структура предохранитель которая работает в дебаг сборке и защищает пользователя
/// от утечек памяти, неправильного использования движка и падения фпс. Отключается
/// в релизной сборке или при наличии фичи no-fuse в Cargo.toml, при неправильном
/// использовании движка вызывает панику
pub struct MoonFuse {
    pub render_calls: u16,
}

/// Константа которая определяет максимальное количество объектов в одном
/// ObjectStore
const MAX_OBJECTS: usize = 50_000;

/// Константа которая определяет максимальное количество текстур в
/// RenderState движка (хэш мап)
const MAX_TEXTURES: usize = 200;

/// Константа которая определяет максимальное количество шрифтов в
/// текстовой подсистеме MoonWalk
const MAX_FONTS: usize = 100;

impl MoonFuse {
    pub fn new() -> Self {
        Self {
            // TODO: Добавить валидацию создания снапшотов
            render_calls: 0,
        }
    }

    /// Частая ошибка новичков заключается в том, что они пытаются загрузить
    /// RGBa формата как в фоторедакторах или палитрах, от 0 до 255. Чтобы
    /// у них не возникал вопрос почему объект белый при использовании
    /// значений выше 1.0 (формат RGBa в мунволке от 0.0 до 1.0, дробный)
    /// в дебаг сборке когда нет флага no_fuse будет идти паника с сообщением
    /// почему нельзя использовать значения RGBa от 0 до 255 и какие
    /// использовать нужно. Это помогает новичкам сразу понять какой формат
    /// цвета нужен для объектов 
    #[cfg(all(debug_assertions, not(feature = "no_fuse")))]
    pub fn validate_color(&self, color: Vec4) {
        let message = include_str!("fuse_errors/validate_color_error.txt");

        if color.x > 1.0 || color.y > 1.0 || color.z > 1.0 || color.w > 1.0 {
            panic!("{}", message);
        }

        // Дополнительная проверка на отрицательные числа
        if color.x < 0.0 || color.y < 0.0 || color.z < 0.0 || color.w < 0.0 {
            panic!("{}", message);
        }
    }

    /// Дополнительная проверка на id текстуры. Конкретно здесь проверяется
    /// ситуация когда пользователь может использовать как id для текстуры
    /// u32::MAX как заглушку (либо u32::MAX - 1), но эти id захардкожены
    /// в UberBatch для определения простого текста и эмодзи. Эти id нельзя
    /// использовать поэтому сразу же нужно выдать панику с сообщением
    /// которое объясняет ситуацию и просит использовать нормальный id
    /// для текстуры
    #[cfg(all(debug_assertions, not(feature = "no_fuse")))]
    pub fn validate_texture_id(&self, id: TextureId) {
        let message = include_str!("fuse_errors/validate_texture_error.txt");

        if id.0 == u32::MAX || id.0 == u32::MAX - 1 {
            panic!("{}", message);
        }
    }

    /// В многих других движках рисование работает по принципу "здесь и сейчас"
    /// когда вызывается draw функция и объект появлется на экране, после чего
    /// в следующем кадре исчезает. В MoonWalk существует ObjectStore, но
    /// об этом могут не знать некоторые пользователи и создавать объекты
    /// в бесконечном цикле, поэтому нужна валидация их количества в 
    /// ObjectStore (для указания на утечку)
    #[cfg(all(debug_assertions, not(feature = "no_fuse")))]
    pub fn validate_objects_count(&self, count: usize) {
        let message = include_str!("fuse_errors/validate_object_count.txt");

        if count > MAX_OBJECTS {
            panic!("{}", message);
        }
    }

    /// Эта проверка нужна в первую очередь для того чтобы предотвратить вызов
    /// .snapshot в каждом кадре. В сообщении есть отдельный пункт про то,
    /// что вместо бесконечного .snapshot стоит использовать .update_snapshot
    /// что даст больше fps, снизит нагрузку на VRAM и в целом это будет более
    /// лучший подход. Также тут проверяется утечка текстур/эмодзи при загрузке
    /// каждый кадр/в цикле
    #[cfg(all(debug_assertions, not(feature = "no_fuse")))]
    pub fn validate_resource_count(&self, textures_count: usize, fonts_count: usize) {
        let message = include_str!("fuse_errors/validate_resource_error.txt");

        if textures_count > MAX_TEXTURES || fonts_count > MAX_FONTS {
            panic!("{}", message);
        }
    }

    #[cfg(not(all(debug_assertions, not(feature = "no_fuse"))))]
    pub fn validate_color(&self, _color: Vec4) {
        // Пустой метод
    }

    #[cfg(not(all(debug_assertions, not(feature = "no_fuse"))))]
    pub fn validate_texture_id(&self, _id: TextureId) {
        // Пустой метод
    }

    #[cfg(not(all(debug_assertions, not(feature = "no_fuse"))))]
    pub fn validate_objects_count(&self, _count: usize) {
        // Пустой метод
    }

    #[cfg(not(all(debug_assertions, not(feature = "no_fuse"))))]
    pub fn validate_resource_count(&self, _textures_count: usize, _fonts_count: usize) {
        // Пустой метод
    }
}
