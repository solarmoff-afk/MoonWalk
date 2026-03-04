// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use crate::error;
use crate::{MoonWalk, FontAsset};
use crate::core::objects::TextureId;

impl MoonWalk {
    /// Этот метод агружает текстуру из файла через его путь
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
    pub fn load_texture(&mut self, path: &str) -> Result<TextureId, error::MoonWalkError> {
        let texture = self.resources.load_texture(&mut self.renderer.context, path)?;
        let id = self.renderer.register_texture(texture);
        
        Ok(TextureId::new(id))
    }

    /// Этот метод читает utf8 файл и возвращает его содержимое в String 
    /// который обёрнут в Result. Работает на android
    ///
    /// [?] Android примеры:
    ///  "test.txt" - файл test.txt из assets приложения
    ///  "data/data/com.example.package/file/test.txt" - файл test.txt из файловой системы
    pub fn load_utf8(&self, path: &str) -> Result<String, error::MoonWalkError> {
        Ok(self.resources.read_text(path)?)
    }

    /// Эта функция очищает текстуру из памяти. Текстура после очищения просто
    /// перестанет отобразиться на объекте
    pub fn remove_texture(&mut self, texture_id: TextureId) {
        self.renderer.remove_texture(texture_id.0);
    }

    /// Эта функция загружает шрифт во время выполнения программы (Этот шрифт обязательно
    /// должен поставляться с программой) используя путь к шрифту. Возвращает структуру
    /// FontAsset (обёртка для u64) который нужен чтобы не использовать структуру FontId
    /// из TextWare
    pub fn load_font(&mut self, path: &str, _name: &str) -> Result<FontAsset, crate::error::MoonWalkError> {
        // _name нужен чтобы не ломать api так как раньше использовался чтобы задать
        // имя шрифта. Это было критически важно для cosmic-text, но с переходом
        // на moonpaint в этом нет необходимости поэтому textware не принимает
        // имя шрифта
        let bytes = self.resources.read_bytes(path)?;
        
        let internal_id = self.renderer.text_engine.load_font_bytes(bytes.to_vec())?;

        // Это безопасно, так как чтобы получить id необходимо загрузить шрифт
        // из байтов либо из файла, получить больше +- 100 шрифтов можно
        // только если в цикле идёт утечка загрузки шрифтов, но в таком случае
        // MoonFuse должен остановить работу приложения в дебаге вызван pamic!
        // указав что где-то в коде есть утечка шрифтов
        Ok(FontAsset(internal_id.0 as u64))
    }

    /// Эта функция загружает шрифт из набора байт который чаще всего известен уже на этапе
    /// компиляции. Создана для того, чтобы вшивать шрифт в бинарник/разделяемую библиотеку
    /// используя макрос для получения набора байтов из файла во время компиляции
    pub fn load_font_from_bytes(
        &mut self, 
        bytes: &[u8], 
        _name: &str
    ) -> Result<FontAsset, crate::error::MoonWalkError> {
        let id = self.renderer.text_engine.load_font_bytes(bytes.to_vec())?;

        // Это безопасно, пояснение выше
        Ok(FontAsset(id.0 as u64))
    }

    /// Возвращает размер текстуры в физических пикселях (ширина и высота) а если текстура
    /// не найдена то возвращает нули Vec2 [0.0, 0.0]
    pub fn get_texture_size(&self, texture_id: TextureId) -> glam::Vec2 {
        if let Some(tex) = self.renderer.state.textures.get(&texture_id.0) {
            glam::Vec2::new(tex.width as f32, tex.height as f32)
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
    pub fn get_texture_pixel(&mut self, texture_id: TextureId, x: u32, y: u32) -> Option<glam::Vec4> {
        let texture = self.renderer.state.textures.get(&texture_id.0)?;
        
        match texture.read_pixel(&mut self.renderer.context, x, y) {
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
    pub async fn load_texture_async(&mut self, path: &str) -> Result<TextureId, error::MoonWalkError> {
        let texture = self.resources.load_texture_async(&mut self.renderer.context, path).await?;

        let id = self.renderer.register_texture(texture);
        
        Ok(id.0)
    }

    /// Асинхронная загрузка шрифта. Читает файл без блокировки потока, парсит байты синхронно
    #[cfg(feature = "async")]
    pub async fn load_font_async(&mut self, path: &str, name: &str) -> Result<FontAsset, crate::error::MoonWalkError> {
        let bytes = self.resources.read_bytes_async(path).await?;
        
        let internal_id = self.renderer.text_engine.load_font_bytes(&bytes, name)?;

        Ok(FontAsset(internal_id.0))
    }
}
