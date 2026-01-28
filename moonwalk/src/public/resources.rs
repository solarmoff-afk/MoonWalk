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

    /// Эта функция очищает текстуру из памяти. Текстура после очищения просто
    /// перестанет отобразиться на объекте
    pub fn remove_texture(&mut self, texture_id: u32) {
        self.renderer.remove_texture(texture_id);
    }

    /// Эта функция загружает шрифт во время выполнения программы (Этот шрифт обязательно
    /// должен поставляться с программой) используя путь к шрифту. Возвращает структуру
    /// FontAsset (обёртка для u64) который нужен чтобы не использовать структуру FontId
    /// из TextWare
    pub fn load_font(&mut self, path: &str, name: &str) -> Result<FontAsset, crate::error::MoonWalkError> {
        let bytes = self.resources.read_bytes(path)?;
        
        let internal_id = self.renderer.text_engine.load_font_bytes(&bytes, name)?;

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
        let id = self.renderer.text_engine.load_font_bytes(bytes, name)?;
        Ok(FontAsset(id.0))
    }

    /// Возвращает размер текстуры в физических пикселях (ширина и высота) а если текстура
    /// не найдена то возвращает нули Vec2 [0.0, 0.0]
    pub fn get_texture_size(&self, texture_id: u32) -> glam::Vec2 {
        if let Some(tex) = self.renderer.state.textures.get(&texture_id) {
            glam::Vec2::new(tex.texture.width() as f32, tex.texture.height() as f32)
        } else {
            glam::Vec2::ZERO
        }
    }

    /// Этот метод нужен чтобы получить цвет конкретного пикселя текстуры
    /// по координатам. Принимает x и y. Не Vec2 так как не является настройкой
    /// какого либо объекта. Возвращает option для цвета пикселя текстуры
    /// по физическим координатам в Vec4 из glam и в диапазоне от 0.0 до 1.0
    /// (для единобразия апи). Если координаты выходят за размер текстуры то
    /// возвращает None. Паники в таком случае не будет
    /// - [!] Эта операция медленная, не рекомендуется использовать каждый кадр
    pub fn get_texture_pixel(&self, texture_id: u32, x: u32, y: u32) -> Option<glam::Vec4> {
        let texture = self.renderer.state.textures.get(&texture_id)?;
        
        match texture.read_pixel(&self.renderer.context, x, y) {
            Ok(bytes) => {
                Some(glam::Vec4::new(
                    bytes[0] as f32 / 255.0,
                    bytes[1] as f32 / 255.0,
                    bytes[2] as f32 / 255.0,
                    bytes[3] as f32 / 255.0,
                ))
            },

            Err(e) => {
                eprintln!("MoonWalk Error reading pixel: {}", e);
                None
            }
        }
    }

    /// Асинхронная загрузка текстуру. Полезен для подгрузки контента в рантайме
    /// Сначала асинхронно читает файл, потом регистрирует текстуру в рендерере
    #[cfg(feature = "async")]
    pub async fn load_texture_async(&mut self, path: &str) -> Result<u32, error::MoonWalkError> {
        let texture = self.resources.load_texture_async(&self.renderer.context, path).await?;

        let id = self.renderer.register_texture(texture);
        
        Ok(id)
    }

    /// Асинхронная загрузка шрифта. Читает файл без блокировки потока, парсит байты синхронно
    #[cfg(feature = "async")]
    pub async fn load_font_async(&mut self, path: &str, name: &str) -> Result<FontAsset, crate::error::MoonWalkError> {
        let bytes = self.resources.read_bytes_async(path).await?;
        
        let internal_id = self.renderer.text_engine.load_font_bytes(&bytes, name)?;

        Ok(FontAsset(internal_id.0))
    }
}
