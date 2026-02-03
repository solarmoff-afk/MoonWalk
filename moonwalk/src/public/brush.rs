// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use glam::{Vec2, Vec4};

use crate::MoonWalk;
use crate::painting::BrushVertex;

use crate::r#abstract::BlendMode as InternalBlendMode;
// use moonwalk_backend::pipeline::types::BlendMode as InternalBlendMode;

/// Простой генератор псевдослучайных чисел для джиттера
struct Lcg {
    state: u32,
}

impl Lcg {
    fn new(seed: u32) -> Self {
        Self { state: seed }
    }
    
    // Возвращает флоат от -1.0 до 1.0
    fn next_f32_signed(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
        let val = (self.state >> 9) | 0x3f800000;
        let f = f32::from_bits(val) - 1.0;
        f * 2.0 - 1.0
    }
}

/// Режимы наложения для кисти.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlendMode {
    #[default]
    Normal,
    Add,      // Сложение (свечение)
    Multiply, // Умножение (тени)
    Screen,   // Мягкое осветление
    Subtract, // Вычитание
    
    // Eraser здесь необязателен так как он управляется флагом, но я его оставил
    // для полноты апи
    Eraser,
}

impl BlendMode {
    fn to_internal(&self) -> InternalBlendMode {
        match self {
            BlendMode::Normal => InternalBlendMode::Alpha,
            BlendMode::Add => InternalBlendMode::Additive,
            BlendMode::Multiply => InternalBlendMode::Multiply,
            BlendMode::Screen => InternalBlendMode::Screen,
            BlendMode::Subtract => InternalBlendMode::Subtract,
            BlendMode::Eraser => InternalBlendMode::Eraser,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Brush {
    pub size: f32,
    pub color: Vec4,
    pub hardness: f32,
    pub opacity: f32,
    pub spacing: f32,
    pub texture_id: u32,
     
    /// Угол поворота кисти в радианах
    pub angle: f32,

    /// Если true то кисть поворачивается по направлению штриха
    pub follow_direction: bool,

    /// Отношение ширины к высоте (от 0.0 до 1.0) 1.0 это круг 0.1 это плоская линия
    pub roundness: f32,
    
    /// Разброс позиции
    pub jitter_position: f32,

    /// Разброс размера
    pub jitter_size: f32,

    /// Разброс угла
    pub jitter_angle: f32,

    /// Разброс прозрачности
    pub jitter_opacity: f32,

    /// Ластик ли это? (ластик стирает следы от кистей)
    pub is_eraser: bool,

    /// Режим смешиивания
    pub blend_mode: BlendMode,
}

impl Default for Brush {
    fn default() -> Self {
        Self {
            size: 20.0,
            color: Vec4::new(0.0, 0.0, 0.0, 1.0),
            hardness: 0.8,
            opacity: 1.0,
            spacing: 5.0,
            texture_id: 0, 
            angle: 0.0,
            follow_direction: false,
            roundness: 1.0,
            jitter_position: 0.0,
            jitter_size: 0.0,
            jitter_angle: 0.0,
            jitter_opacity: 0.0,
            is_eraser: false,
            blend_mode: BlendMode::Normal,
        }
    }
}

impl MoonWalk {
    /// Этот метод создаёт растровую кисть для рисования на текстуре
    pub fn new_brush(&self) -> Brush {
        Brush::default()
    }

    /// [WAIT DOC]
    pub fn draw_stroke(&mut self, target_id: u32, brush: &Brush, from: Vec2, to: Vec2) {
        let texture_res = self.renderer.state.textures.get(&target_id);
        if texture_res.is_none() {
            return;
        }
        
        let tip_texture = if brush.texture_id > 0 {
            self.renderer.state.textures.get(&brush.texture_id)
        } else {
            None
        };

        let dist = from.distance(to);
        let steps = (dist / brush.spacing).ceil() as usize;
        let steps = steps.max(1);

        let mut instances = Vec::with_capacity(steps + 1);
        
        // Вектор направления штриха для follow_direction
        let dir = (to - from).normalize_or_zero();
        let stroke_angle = if brush.follow_direction {
            dir.y.atan2(dir.x)
        } else {
            0.0
        };

        let seed = (from.x * 100.0 + from.y * 1000.0) as u32;
        let mut rng = Lcg::new(seed);

        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let mut pos = from.lerp(to, t);
            
            if brush.jitter_position > 0.0 {
                let jx = rng.next_f32_signed() * brush.jitter_position * brush.size;
                let jy = rng.next_f32_signed() * brush.jitter_position * brush.size;
                pos += Vec2::new(jx, jy);
            }

            let mut size = brush.size;
            if brush.jitter_size > 0.0 {
                // Вариация от 50 до 150 процентов размера при максимальном джиттере
                let factor = 1.0 + (rng.next_f32_signed() * brush.jitter_size * 0.5);
                size *= factor;
            }
            
            // Сплющивание
            let size_vec = Vec2::new(size, size * brush.roundness);

            // Вращение
            let mut angle = brush.angle + stroke_angle;
            if brush.jitter_angle > 0.0 {
                // Примерно 180 градусов при максимальном джиттере
                angle += rng.next_f32_signed() * std::f32::consts::PI * brush.jitter_angle;
            }

            // Прозрачность
            let mut alpha = brush.opacity;
            if brush.jitter_opacity > 0.0 {
                alpha *= 1.0 - (rng.next_f32_signed().abs() * brush.jitter_opacity);
            }

            instances.push(BrushVertex {
                position: [pos.x, pos.y],
                size: [size_vec.x, size_vec.y],
                rotation: angle,
                opacity: alpha,
            });
        }

        let internal_mode = if brush.is_eraser {
            InternalBlendMode::Eraser
        } else {
            brush.blend_mode.to_internal()
        };

        if let Some(target) = self.renderer.state.textures.get(&target_id) {
            self.renderer.painting_system.draw_strokes(
                &self.renderer.context,
                target,
                tip_texture,
                &instances,
                brush.color,
                brush.hardness,
                internal_mode,
            );
        }
    }
    
    pub fn draw_stamp(&mut self, target_id: u32, brush: &Brush, pos: Vec2) {
        self.draw_stroke(target_id, brush, pos, pos);
    }
}
