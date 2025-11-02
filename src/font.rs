use std::collections::HashMap;
use std::path::Path;
use crate::error::MoonWalkError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct FontId(pub u64);

impl FontId {
    pub fn to_u64(self) -> u64 { self.0 }
    pub fn from_u64(id: u64) -> Self { Self(id) }
}

struct FontInfo {
    family: String,
    size: f32,
}

pub struct FontSystem {
    cosmic_font_system: cosmic_text::FontSystem,
    next_id: u64,
    fonts: HashMap<FontId, FontInfo>,
}

impl FontSystem {
    pub fn new() -> Self {
        Self {
            cosmic_font_system: cosmic_text::FontSystem::new(),
            next_id: 1,
            fonts: HashMap::new(),
        }
    }
    
    pub fn cosmic_mut(&mut self) -> &mut cosmic_text::FontSystem {
        &mut self.cosmic_font_system
    }

    pub fn load_font(&mut self, path: &str, size: f32) -> Result<FontId, MoonWalkError> {
        let font_data = std::fs::read(path)
            .map_err(|e| MoonWalkError::FontLoading(e.to_string()))?;
        
        self.cosmic_font_system.db_mut().load_font_data(font_data);

        let family_name = Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let id = FontId(self.next_id);
        self.next_id += 1;

        let info = FontInfo {
            family: family_name,
            size,
        };

        self.fonts.insert(id, info);
        Ok(id)
    }

    pub fn clear_font(&mut self, id: FontId) {
        self.fonts.remove(&id);
    }

    pub fn get_font_info(&self, id: FontId) -> Option<(String, f32)> {
        self.fonts.get(&id).map(|info| (info.family.clone(), info.size))
    }
}

impl Default for FontSystem {
    fn default() -> Self {
        Self::new()
    }
}