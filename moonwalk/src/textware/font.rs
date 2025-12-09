// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use std::collections::HashMap;
use std::path::Path;
use std::ffi::CString;

use crate::textware::TextError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct FontId(pub u64);

pub struct FontSystem {
    pub(crate) sys: cosmic_text::FontSystem,
    next_id: u64,
    families: HashMap<FontId, String>,

    /// AssetManager только на android. На десктопе и iOS всё
    /// работает через прямые пути
    #[cfg(target_os = "android")]
    asset_manager: ndk::asset::AssetManager,
}

impl FontSystem {
    #[cfg(not(target_os = "android"))]
    pub fn new() -> Self {
        Self {
            sys: cosmic_text::FontSystem::new(),
            next_id: 1,
            families: HashMap::new(),
        }
    }

    #[cfg(target_os = "android")]
    pub fn new(asset_manager: ndk::asset::AssetManager) -> Self {
        Self {
            sys: cosmic_text::FontSystem::new(),
            next_id: 1,
            families: HashMap::new(),
            asset_manager,
        }
    }

    pub fn load_font(&mut self, path: &str) -> Result<FontId, TextError> {
        let font_data = {
            #[cfg(target_os = "android")]
            {
                // Создание C строки поскольку крейт ndk это обёртка над c кодом, а там
                // строки отличаются от растовских 
                let c_path = CString::new(path)
                    .map_err(|e| TextError::FontLoading(format!("Path contains null byte: {}", e)))?;

                let mut asset = self.asset_manager.open(&c_path)
                    .ok_or_else(|| TextError::FontLoading(format!("Asset not found: {}", path)))?;
                
                asset.buffer().map(|b| b.to_vec())
                    .map_err(|e| TextError::FontLoading(e.to_string()))
            }

            #[cfg(not(target_os = "android"))]
            {
                std::fs::read(path).map_err(|e| TextError::FontLoading(e.to_string()))
            }
        }?;

        self.sys.db_mut().load_font_data(font_data);

        let family_name = Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let id = FontId(self.next_id);
        self.next_id += 1;

        self.families.insert(id, family_name);

        Ok(id)
    }

    pub fn load_font_from_bytes(&mut self, data: &[u8], name: &str) -> Result<FontId, TextError> {
        self.sys.db_mut().load_font_data(data.to_vec());
        
        let id = FontId(self.next_id);
        self.next_id += 1;
        
        self.families.insert(id, name.to_string());
        
        Ok(id)
    }

    pub fn get_family_name(&self, id: FontId) -> Option<&String> {
        self.families.get(&id)
    }
}