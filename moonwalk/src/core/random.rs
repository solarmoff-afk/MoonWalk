// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

/// Простой генератор псевдослучайных чисел для джиттера
pub struct Lcg {
    state: u32,
}

impl Lcg {
    pub fn new(seed: u32) -> Self {
        Self {
            state: seed
        }
    }
    
    // Возвращает флоат от -1.0 до 1.0
    pub fn next_f32_signed(&mut self) -> f32 {
        self.state = self.state
            .wrapping_mul(1664525)
            .wrapping_add(1013904223);
        
        let val = (self.state >> 9) | 0x3f800000;
        let f = f32::from_bits(val) - 1.0;
        
        f * 2.0 - 1.0
    }
}
