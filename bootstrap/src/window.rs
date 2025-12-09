use glam::Vec2;

#[derive(Debug, Clone)]
pub struct WindowSettings {
    pub title: String,
    pub min_size: Option<Vec2>,
    pub max_size: Option<Vec2>,
    pub initial_size: Vec2,
    pub resizable: bool,
    pub transparent: bool,
    pub decorated: bool,
}

impl WindowSettings {
    pub fn new(title: impl Into<String>, width: f32, height: f32) -> Self {
        Self {
            title: title.into(),
            initial_size: Vec2::new(width, height),
            min_size: None,
            max_size: None,
            resizable: true,
            transparent: false,
            decorated: true,
        }
    }

    pub fn with_min_size(mut self, width: f32, height: f32) -> Self {
        self.min_size = Some(Vec2::new(width, height));
        self
    }

    pub fn with_max_size(mut self, width: f32, height: f32) -> Self {
        self.max_size = Some(Vec2::new(width, height));
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    pub fn transparent(mut self, transparent: bool) -> Self {
        self.transparent = transparent;
        self
    }
    
    pub fn no_decoration(mut self) -> Self {
        self.decorated = false;
        self
    }
}