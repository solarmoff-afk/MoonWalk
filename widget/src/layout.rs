// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use taffy::prelude::*;
use taffy::style::Style;

#[derive(Clone, Debug)]
pub struct Layout {
    pub style: Style,
}

impl Layout {
    pub fn new() -> Self {
        Self {
            style: Style::DEFAULT,
        }
    }

    pub fn display(mut self, display: Display) -> Self {
        self.style.display = display;
        self
    }

    pub fn direction(mut self, dir: FlexDirection) -> Self {
        self.style.flex_direction = dir;
        self
    }

    pub fn column() -> Self {
        Self::new().display(Display::Flex).direction(FlexDirection::Column)
    }

    pub fn row() -> Self {
        Self::new().display(Display::Flex).direction(FlexDirection::Row)
    }

    pub fn stack() -> Self {
        Self::new().display(Display::Flex)
    }

    pub fn wrap(mut self) -> Self {
        self.style.flex_wrap = FlexWrap::Wrap;
        self
    }

    pub fn gap(mut self, width: f32, height: f32) -> Self {
        self.style.gap = Size {
            width: length(width),
            height: length(height),
        };
        self
    }

    pub fn justify(mut self, justify: JustifyContent) -> Self {
        self.style.justify_content = Some(justify);
        self
    }

    pub fn align(mut self, align: AlignItems) -> Self {
        self.style.align_items = Some(align);
        self
    }

    pub fn padding(mut self, left: f32, top: f32, right: f32, bottom: f32) -> Self {
        self.style.padding = Rect {
            left: length(left),
            right: length(right),
            top: length(top),
            bottom: length(bottom),
        };
        self
    }
    
    pub fn padding_all(self, val: f32) -> Self {
        self.padding(val, val, val, val)
    }

    pub fn margin(mut self, left: f32, top: f32, right: f32, bottom: f32) -> Self {
        self.style.margin = Rect {
            left: length(left).into(),
            right: length(right).into(),
            top: length(top).into(),
            bottom: length(bottom).into(),
        };
        self
    }

    pub fn margin_all(self, val: f32) -> Self {
        self.margin(val, val, val, val)
    }

    pub fn width(mut self, px: f32) -> Self {
        self.style.size.width = length(px).into();
        self
    }

    pub fn width_pct(mut self, pct: f32) -> Self {
        self.style.size.width = percent(pct).into();
        self
    }
    
    pub fn width_auto(mut self) -> Self {
        self.style.size.width = Dimension::Auto;
        self
    }

    pub fn height(mut self, px: f32) -> Self {
        self.style.size.height = length(px).into();
        self
    }

    pub fn height_pct(mut self, pct: f32) -> Self {
        self.style.size.height = percent(pct).into();
        self
    }
    
    pub fn height_auto(mut self) -> Self {
        self.style.size.height = Dimension::Auto;
        self
    }

    pub fn size(self, w: f32, h: f32) -> Self {
        self.width(w).height(h)
    }

    pub fn size_pct(self, w: f32, h: f32) -> Self {
        self.width_pct(w).height_pct(h)
    }

    pub fn grow(mut self, factor: f32) -> Self {
        self.style.flex_grow = factor;
        self
    }

    pub fn shrink(mut self, factor: f32) -> Self {
        self.style.flex_shrink = factor;
        self
    }
    
    pub fn absolute(mut self) -> Self {
        self.style.position = Position::Absolute;
        self
    }
    
    pub fn inset(mut self, left: f32, top: f32, right: f32, bottom: f32) -> Self {
        self.style.inset = Rect {
            left: length(left).into(),
            right: length(right).into(),
            top: length(top).into(),
            bottom: length(bottom).into(),
        };
        self
    }

    pub fn build(self) -> Style {
        self.style
    }
}

fn length(v: f32) -> LengthPercentage {
    LengthPercentage::Length(v)
}

fn percent(v: f32) -> LengthPercentage {
    LengthPercentage::Percent(v)
}
