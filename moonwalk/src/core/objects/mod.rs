// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

pub mod store;

/// Айди объекта
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId(pub usize);

/// Айди текстуры
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureId(pub u32);

/// Айди шейдера
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ShaderId(pub u32);

/// Флаг грязности для объекта. Вся его суть в том что вместо прямого bool
/// где нужно следить за избавлением от параметра эта структура автоматически
/// переключает состояние при чтении через get, что не позволяет забыть
/// поставить dirty = false
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectDirty(bool);

impl ObjectDirty {
    pub fn new() -> Self {
        // При создании нового объекта его ещё нет в кэше батчинга, поэтому
        // нужно сразу же сделать его грязным чтобы батчинг заметил
        Self(true)
    }

    pub fn get(&mut self) -> bool {
        let data = self.0;
        
        // Когда флаг прочитан он больше не грязный
        self.0 = false;

        data
    }

    /// При изменении поля объекта нужно вызвать этот метод
    pub fn update(&mut self) {
        self.0 = true;
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectType {
    Unknown = 0,
    Rect = 1,
    Text = 2,
}

impl ObjectType {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            1 => Some(Self::Rect),
            _ => None,
        }
    }
}

impl TextureId {
    #[inline(always)]
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

impl ObjectId {
    // Хардкод
    const INDEX_MASK: usize = 0x00FF_FFFF;
    const TYPE_SHIFT: usize = 24;

    #[inline(always)]
    pub fn new(ty: ObjectType, index: usize) -> Self {
        let ty_val = (ty as usize) << Self::TYPE_SHIFT;
        let idx_val = index & Self::INDEX_MASK;
        Self(ty_val | idx_val)
    }

    #[inline(always)]
    pub fn get_type(&self) -> Option<ObjectType> {
        let ty_val = ((self.0 >> Self::TYPE_SHIFT) & 0xFF) as u8;
        ObjectType::from_u8(ty_val)
    }

    #[inline(always)]
    pub fn index(&self) -> usize {
        self.0 & Self::INDEX_MASK
    }
}
