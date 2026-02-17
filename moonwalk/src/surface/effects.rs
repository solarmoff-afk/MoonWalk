// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use crate::effects::material::elevation::{MoonMaterialLevel, draw_shadow};
use crate::surface::MoonSurface;
use crate::{MoonWalk, MoonWalkError, ObjectId};

impl MoonSurface {
    pub fn get_elevation(&self, level: u8) -> MoonMaterialLevel {
        match level {
            // Уровень ноль. В материал 3 он используется когда нужны фоновые
            // поверхности, которые не должны казаться поднятными. Например
            // базовый фон (цвет фона экрана), карточки без тени (такие бывают),
            // поля ввода текста также используют нулевой уровень, так как
            // находятся внутри страницы, а не над ней (По канонам)
            0 => MoonMaterialLevel { dp: 0.0,  tonal_alpha: 0.0,  opacity: 0.0  },

            // Первый уровень элевации. Минимальное отделение от фона, используется
            // для компонентов которые должны быть немного приподняты, чтобы
            // показать интерактивность. Используется для карточек, пунктов меню
            1 => MoonMaterialLevel { dp: 1.0,  tonal_alpha: 0.05, opacity: 0.25 },

            // Второй уровень. Элементы, которые находятся над первым уровнем.
            // Это FAB при нажатии, строка поиска, панель навигации снизу, а
            // также чипсы (chips если что) в нажатом состоянии
            2 => MoonMaterialLevel { dp: 3.0,  tonal_alpha: 0.08, opacity: 0.30 },

            // Третий уровень. Компоненты, которые временно перекрывают контент
            // или требуют внимания. Используется в боковом меню, всплывающем
            // уведомлени внизу экрана, плавающей панели (bottom sheet)
            3 => MoonMaterialLevel { dp: 6.0,  tonal_alpha: 0.11, opacity: 0.35 },

            // Четвёртый уровень. Диалоговые окна и модальные компоненты. Используется
            // в диалоговом окне и в других модальных компонентах типа modal
            // bottom sheet
            4 => MoonMaterialLevel { dp: 8.0,  tonal_alpha: 0.12, opacity: 0.38 },

            // Пятый уровень. Самые важные временные элементы которые должны
            // доминировать над всем интерфейсом. К нему относятся пикер даты,
            // меню, FAB при фокусе и так далее
            5 => MoonMaterialLevel { dp: 12.0, tonal_alpha: 0.14, opacity: 0.42 },

            // Ноль по умолчанию
            _ => MoonMaterialLevel { dp: 0.0,  tonal_alpha: 0.0,  opacity: 0.0  },
        }
    }

    pub fn new_shadow(&self, mw: &mut MoonWalk, base: ObjectId, level: MoonMaterialLevel) -> Result<ObjectId, MoonWalkError> {
        let position = self.store.positions[base.0];
        let size = self.store.sizes[base.0];
        let radii = self.store.rect_radii[base.0];

        let shadow = draw_shadow(mw, level, position, size, radii)?;
        
        Ok(shadow)
    }
}