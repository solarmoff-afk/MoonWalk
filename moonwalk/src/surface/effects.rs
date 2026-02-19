// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use glam::{Vec2, Vec4};

use crate::effects::EffectFactory;
use crate::effects::material::elevation::MoonMaterialLevel;
use crate::surface::MoonSurface;
use crate::{MoonWalk, MoonWalkError, ObjectId};

impl MoonSurface {
    pub fn new_effect_factory(&self) -> EffectFactory {
        EffectFactory::new()
    }

    pub fn new_shadow_object(
        &mut self,
        base: ObjectId,
        level: MoonMaterialLevel,
        umbra_tex: u32,
        penumbra_tex: u32,
        ambient_tex: u32,
        density: f32,
    ) -> Result<(ObjectId, ObjectId, ObjectId), MoonWalkError> {
        let position = self.store.get_position(base);
        let size = self.store.get_size(base);
        let base_z = self.store.get_z_index(base);

        let tex_array = [umbra_tex, penumbra_tex, ambient_tex];
        let mut obj_ids = [ObjectId(0); 3];

        for i in 0..3 {
            let layer = &level.layers[i];
            
            let sigma = layer.blur * 0.5 * density;
            let spread = layer.spread * density;
            let margin = (sigma * 3.5).ceil();
            
            let layer_size = size + Vec2::splat(spread * 2.0);
            let offset = layer.offset * density;

            let rect = self.new_rect();
            
            let final_pos = position - Vec2::splat(margin + spread) + offset;
            
            self.set_position(rect, final_pos);
            self.set_size(rect, layer_size + Vec2::splat(margin * 2.0));
            self.set_texture(rect, tex_array[i]);
            self.set_color(rect, Vec4::new(0.0, 0.0, 0.0, layer.alpha));
            
            self.set_z_index(rect, base_z - 0.1 - (i as f32 * 0.1));
            
            obj_ids[i] = rect;
        }

        Ok((obj_ids[0], obj_ids[1], obj_ids[2]))
    }
}