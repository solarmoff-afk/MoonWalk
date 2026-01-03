// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use crate::error;
use crate::{MoonWalk, FontAsset};

impl MoonWalk {
    /// Эта функция агружает текстуру из файла через его путь
    ///  [!] Данная функция очень медленная, не рекомендуется подгружать всё
    ///      при старте программы
    /// На windows, linux, macos, bsd и android указывается путь в файловой системе
    /// На android указывается либо путь к файловой системе
    ///   (Определяется по "/" как первый символ)
    /// либо как имя файла в assets.
    ///
    /// [?] Android примеры:
    ///  "test.png" - файл test.png из assets приложения
    ///  "data/data/com.example.package/file/test.png" - файл test.png из файловой системы
    pub fn load_texture(&mut self, path: &str) -> Result<u32, error::MoonWalkError> {
        let texture = self.resources.load_texture(&self.renderer.context, path)?;
        let id = self.renderer.register_texture(texture);
        
        Ok(id)
    }

    /// Эта функция загружает шрифт во время выполнения программы (Этот шрифт обязательно
    /// должен поставляться с программой) используя путь к шрифту. Возвращает структуру
    /// FontAsset (обёртка для u64) который нужен чтобы не использовать структуру FontId
    /// из TextWare
    pub fn load_font(&mut self, path: &str, name: &str) -> Result<FontAsset, crate::error::MoonWalkError> {
        let bytes = self.resources.read_bytes(path)?;
        
        let internal_id = self.renderer.text_engine.font_system.load_font_from_bytes(&bytes, name)?;

        Ok(FontAsset(internal_id.0))
    }

    /// Эта функция загружает шрифт из набора байт который чаще всего известен уже на этапе
    /// компиляции. Создана для того, чтобы вшивать шрифт в бинарник/разделяемую библиотеку
    /// используя макрос для получения набора байтов из файла во время компиляции
    pub fn load_font_from_bytes(
        &mut self, 
        bytes: &[u8], 
        name: &str
    ) -> Result<FontAsset, crate::error::MoonWalkError> {
        let id = self.renderer.text_engine.font_system.load_font_from_bytes(bytes, name)?;
        Ok(FontAsset(id.0))
    }
}
