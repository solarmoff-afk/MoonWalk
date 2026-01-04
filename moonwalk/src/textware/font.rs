// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use crate::textware::TextError;

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct FontId(pub u64);

pub struct FontSystem {
    pub(crate) sys: cosmic_text::FontSystem,
    next_id: u64,
    families: HashMap<FontId, String>,
}

impl FontSystem {
    pub fn new() -> Self {
        Self {
            sys: cosmic_text::FontSystem::new(),
            next_id: 1,
            families: HashMap::new(),
        }
    }

    pub fn load_font_from_bytes(&mut self, data: &[u8], _name: &str) -> Result<FontId, TextError> {
        // Имя игнорируется так как реальное имя берётся из метаданных. _name остаётся
        // из-за обратной совместимости
        let count_before = self.sys.db().faces().count();
        
        self.sys.db_mut().load_font_data(data.to_vec());
        
        let count_after = self.sys.db().faces().count();
        
        if count_after <= count_before {
            return Err(TextError::FontLoading("Failed to parse font data".to_string()));
        }

        // Получение последнего добавленного шрифта из базы faces возвращает итератор
        // по всем шрифтам
        let face = self.sys.db().faces().last()
            .ok_or_else(|| TextError::FontLoading("Database error after loading".to_string()))?;

        // Получение настоящего имени семейства из метаданных families возвращает список 
        // кортежей имя и язык поэтому используется первое имя
        let real_family_name = face.families.first()
            .map(|(name, _)| name.clone())
            .unwrap_or_else(|| "Unknown Family".to_string());

        let id = FontId(self.next_id);
        self.next_id += 1;

        self.families.insert(id, real_family_name);

        Ok(id)
    }

    pub fn get_family_name(&self, id: FontId) -> Option<&String> {
        self.families.get(&id)
    }
}
