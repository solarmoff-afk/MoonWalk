// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use glam::{Vec4, Vec2};

use crate::{MoonWalk, MoonWalkError, ObjectId};

pub struct MoonMaterialLevel {
    pub dp: f32,
    pub tonal_alpha: f32,
    pub opacity: f32,
}

pub fn draw_shadow(
    mw: &mut MoonWalk,
    level: MoonMaterialLevel,
    position: Vec2,
    size: Vec2,
    radii: Vec4,
) -> Result<ObjectId, MoonWalkError> {
    let density = mw.get_scale_factor();

    if level.dp > 0.0 {
        let sigma = (level.dp * 0.8 * density).max(1.0);
        let margin = (sigma * 3.5).ceil();

        let mut surface = mw.new_surface(
            (size.x + margin * 2.0) as u32, 
            (size.y + margin * 2.0) as u32
        )?;

        let m_id = surface.new_rect();
        surface.set_position(m_id, Vec2::new(margin, margin));
        surface.set_size(m_id, size);
        surface.set_rounded(m_id, radii);
        surface.set_color(m_id, Vec4::new(0.0, 0.0, 0.0, 1.0));
        surface.render(mw, Some(Vec4::ZERO))?;

        let shadow_tex = surface.snapshot(
            mw,
            Vec2::ZERO,
            Vec2::new(size.x + margin * 2.0, size.y + margin * 2.0)
        )?;
        
        mw.blur_texture(shadow_tex, sigma, true);
        mw.blur_texture(shadow_tex, sigma, false);

        let shadow_rect = mw.new_rect();
        let offset_y = level.dp * 0.9 * density;
        
        mw.set_position(shadow_rect, position - Vec2::splat(margin) + Vec2::new(0.0, offset_y));
        mw.set_size(shadow_rect, size + Vec2::splat(margin * 2.0));
        mw.set_texture(shadow_rect, shadow_tex);
        mw.set_color(shadow_rect, Vec4::new(0.0, 0.0, 0.0, level.opacity));
    
        return Ok(shadow_rect);
    }

    let shadow_rect = mw.new_rect();
    mw.set_color(shadow_rect, Vec4::ZERO);
    mw.set_position(shadow_rect, position);
    mw.set_size(shadow_rect, size);
    
    Ok(shadow_rect)
}