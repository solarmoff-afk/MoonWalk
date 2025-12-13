// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use std::path::Path;

#[cfg(target_os = "android")]
use std::ffi::CString;

use crate::easy_gpu::Context;
use crate::rendering::texture::Texture;
use crate::error::MoonWalkError;

pub struct ResourceManager {
    // Для windows, linux, macos, bsd и ios - он пустой, так как на этих ОС получение файлов
    // (В том числе из ассэтов) делается через путь в реальной файловой системе, но для
    // android нужен AssetManager чтобы получить доступ к ассэтам, папки с ними нет в
    // файловой системе.
    #[cfg(target_os = "android")]
    asset_manager: ndk::asset::AssetManager,
}

impl ResourceManager {
    #[cfg(not(target_os = "android"))]
    pub fn new() -> Self {
        Self {}
    }

    #[cfg(target_os = "android")]
    pub fn new(asset_manager: ndk::asset::AssetManager) -> Self {
        Self {
            asset_manager
        }
    }

    /// Главный загрузчик. Работает очень хитро, на desktop и IOS он просто берёт путь
    /// и читает файл по этому пути через fs, но на android есть два способа чтения.
    /// Если первый символ пути это "/" то файл также загружается из файловой системы
    /// (Пример: data/data/com.example.package/files/test.txt), а если символ любой
    /// другой кроме "/" - файл получается и читается через AssetManager
    pub fn read_bytes(&self, path: &str) -> Result<Vec<u8>, MoonWalkError> {
        #[cfg(target_os = "android")]
        {
            if path.starts_with('/') {
                std::fs::read(path).map_err(|e| MoonWalkError::IOError(e.to_string()))
            } else {
                let c_path = CString::new(path)
                    .map_err(|e| MoonWalkError::IOError(format!("Invalid path string: {}", e)))?;
                
                let mut asset = self.asset_manager.open(&c_path)
                    .ok_or_else(|| MoonWalkError::IOError(format!("Asset not found: {}", path)))?;
                
                asset.buffer().map(|b| b.to_vec())
                    .map_err(|e| MoonWalkError::IOError(e.to_string()))
            }
        }

        #[cfg(not(target_os = "android"))]
        {
            std::fs::read(path).map_err(|e| MoonWalkError::IOError(e.to_string()))
        }
    }

    /// Загружает текстуру из файла через easy_gpu контекст и путь к нему
    pub fn load_texture(&self, ctx: &Context, path: &str) -> Result<Texture, MoonWalkError> {
        let bytes = self.read_bytes(path)?;
        
        let label = Path::new(path).file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown Texture");
            
        Texture::from_bytes(ctx, &bytes, label)
    }
}