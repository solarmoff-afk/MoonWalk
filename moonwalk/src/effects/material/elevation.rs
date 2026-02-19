// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use glam::{Vec4, Vec2};

use crate::{MoonSurface, MoonWalk, MoonWalkError, ObjectId};
use crate::effects::EffectFactory;

#[derive(Clone, Copy, Debug)]
pub struct ShadowLayer {
    pub offset: Vec2,
    pub blur: f32,
    pub spread: f32,
    pub alpha: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct MoonMaterialLevel {
    pub tonal_alpha: f32,
    pub layers: [ShadowLayer; 3],
}

impl EffectFactory {
    pub fn get_elevation(&self, level: u8) -> MoonMaterialLevel {
        match level {
            // Первый уровень элевации. Минимальное отделение от фона, используется
            // для компонентов которые должны быть немного приподняты, чтобы
            // показать интерактивность. Используется для карточек, пунктов меню
            1 => MoonMaterialLevel { 
                tonal_alpha: 0.05,
                layers: [
                    ShadowLayer {
                        offset: Vec2::new(0.0, 2.0),
                        blur: 1.0,
                        spread: -1.0,
                        alpha: 0.20,
                    },

                    ShadowLayer {
                        offset: Vec2::new(0.0, 1.0),
                        blur: 1.0,
                        spread: 0.0,
                        alpha: 0.14,
                    },

                    ShadowLayer {
                        offset: Vec2::new(0.0, 1.0),
                        blur: 3.0,
                        spread: 0.0,
                        alpha: 0.12,
                    }
                ]
            },

            // Второй уровень. Элементы, которые находятся над первым уровнем.
            // Это FAB при нажатии, строка поиска, панель навигации снизу, а
            // также чипсы (chips если что) в нажатом состоянии
            2 => MoonMaterialLevel { 
                tonal_alpha: 0.08,
                layers: [
                    ShadowLayer {
                        offset: Vec2::new(0.0, 3.0),
                        blur: 1.0,
                        spread: -2.0,
                        alpha: 0.20,
                    },

                    ShadowLayer {
                        offset: Vec2::new(0.0,2.0),
                        blur: 2.0,
                        spread: 0.0,
                        alpha: 0.14,
                    },

                    ShadowLayer {
                        offset: Vec2::new(0.0, 1.0),
                        blur: 5.0,
                        spread: 0.0,
                        alpha: 0.12,
                    }
                ]
            },

            // Третий уровень. Компоненты, которые временно перекрывают контент
            // или требуют внимания. Используется в боковом меню, всплывающем
            // уведомлени внизу экрана, плавающей панели (bottom sheet)
            3 => MoonMaterialLevel { 
                tonal_alpha: 0.11,
                layers: [
                    ShadowLayer {
                        offset: Vec2::new(0.0, 3.0),
                        blur: 3.0,
                        spread: -2.0,
                        alpha: 0.20,
                    },

                    ShadowLayer {
                        offset: Vec2::new(0.0,3.0),
                        blur: 4.0,
                        spread: 0.0,
                        alpha: 0.14,
                    },

                    ShadowLayer {
                        offset: Vec2::new(0.0, 1.0),
                        blur: 8.0,
                        spread: 0.0,
                        alpha: 0.12,
                    }
                ]
            },

            // Четвёртый уровень. Диалоговые окна и модальные компоненты. Используется
            // в диалоговом окне и в других модальных компонентах типа modal
            // bottom sheet
            4 => MoonMaterialLevel { 
                tonal_alpha: 0.12,
                layers: [
                    ShadowLayer {
                        offset: Vec2::new(0.0, 3.0),
                        blur: 5.0,
                        spread: -1.0,
                        alpha: 0.20,
                    },

                    ShadowLayer {
                        offset: Vec2::new(0.0,6.0),
                        blur: 10.0,
                        spread: 0.0,
                        alpha: 0.14,
                    },

                    ShadowLayer {
                        offset: Vec2::new(0.0, 1.0),
                        blur: 18.0,
                        spread: 0.0,
                        alpha: 0.12,
                    }
                ]
            },

            // Пятый уровень. Самые важные временные элементы которые должны
            // доминировать над всем интерфейсом. К нему относятся пикер даты,
            // меню, FAB при фокусе и так далее
            5 => MoonMaterialLevel { 
                tonal_alpha: 0.14,
                layers: [
                    ShadowLayer {
                        offset: Vec2::new(0.0, 7.0),
                        blur: 8.0,
                        spread: -4.0,
                        alpha: 0.20,
                    },

                    ShadowLayer {
                        offset: Vec2::new(0.0,12.0),
                        blur: 17.0,
                        spread: 2.0,
                        alpha: 0.14,
                    },

                    ShadowLayer {
                        offset: Vec2::new(0.0, 5.0),
                        blur: 22.0,
                        spread: 4.0,
                        alpha: 0.12,
                    }
                ]
            },

            // Ноль по умолчанию
            // Уровень ноль. В материал 3 он используется когда нужны фоновые
            // поверхности, которые не должны казаться поднятными. Например
            // базовый фон (цвет фона экрана), карточки без тени (такие бывают),
            // поля ввода текста также используют нулевой уровень, так как
            // находятся внутри страницы, а не над ней (По канонам)
            _ => MoonMaterialLevel {
                tonal_alpha: 0.0,
                layers: [ShadowLayer {offset: Vec2::ZERO, blur: 0.0, spread: 0.0, alpha: 0.0}; 3],
            }
        }
    }

    pub fn new_shadow(
        &self,
        moonwalk: &mut MoonWalk,
        target: Option<&mut MoonSurface>,
        base: ObjectId,
        level: MoonMaterialLevel,
    ) -> Result<(u32, u32, u32), MoonWalkError> {
         let (position, size, radii, base_z) = match target {
            Some(surface) => (
                surface.get_position(base),
                surface.get_size(base),
                surface.get_rounded(base),
                surface.get_z_index(base),
            ),

            None => (
                moonwalk.get_position(base),
                moonwalk.get_size(base),
                moonwalk.get_rounded(base),
                moonwalk.get_z_index(base),
            )
        };

        draw_shadow(moonwalk, level, position, size, radii, base_z)
    }
}

fn draw_shadow(
    moonwalk: &mut MoonWalk,
    level: MoonMaterialLevel,
    _position: Vec2,
    size: Vec2,
    radii: Vec4,
    _base_z_index: f32,
) -> Result<(u32, u32, u32), MoonWalkError> {
    let density = moonwalk.get_scale_factor();
    let mut tex_ids = [0u32; 3];

    for i in 0..3 {
        let layer = &level.layers[i];
        let sigma = (layer.blur * 0.5 * density).max(0.1);
        let spread = layer.spread * density;
        let margin = (sigma * 3.5).ceil();
        
        let layer_size = size + Vec2::splat(spread * 2.0);
        let surf_size = layer_size + Vec2::splat(margin * 2.0);

        let mut surface = moonwalk.new_surface(
            surf_size.x.max(1.0) as u32, 
            surf_size.y.max(1.0) as u32
        )?;
        
        let mask = surface.new_rect();
        surface.set_position(mask, Vec2::splat(margin));
        surface.set_size(mask, layer_size);
        surface.set_rounded(mask, radii); 
        surface.set_color(mask, Vec4::new(0.0, 0.0, 0.0, 1.0));
        surface.render(moonwalk, Some(Vec4::ZERO))?;

        let tex = surface.snapshot(moonwalk, Vec2::ZERO, surf_size)?;

        if sigma > 0.1 {
            moonwalk.blur_texture(tex, sigma, true);
            moonwalk.blur_texture(tex, sigma, false);
        }

        tex_ids[i] = tex;
    }

    Ok((tex_ids[0], tex_ids[1], tex_ids[2]))
}
