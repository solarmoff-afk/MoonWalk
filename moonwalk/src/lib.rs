// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

#![allow(unused_must_use)]

pub mod error;
pub mod objects;
pub mod resource_manager;
pub mod surface;
pub mod path;
pub mod prelude;
pub mod rendering;
pub mod effects;

mod batching;
mod textware;
mod debug;
mod painting;
mod public;
mod gpu;
mod filters;

// Реэкспорт из glam для удобства
pub use glam::{Vec2, Vec3, Vec4, Mat4};

pub use public::brush::{Brush, BlendMode};
pub use public::custom::BindResource;

#[cfg(feature = "video")]
pub use public::video::{MoonVideo, VideoFormat, VideoPreset};

pub use crate::error::MoonWalkError;
pub use crate::objects::ObjectId;
pub use crate::surface::MoonSurface;
pub use crate::textware::FontId;
pub use crate::path::{PathBuilder, LineCap, LineJoin, FillRule};
pub use crate::rendering::container::RenderContainer;
pub use crate::rendering::custom::{
    CustomPaint, MoonRenderPass, MoonBuffer, MoonBindGroup, CustomPipeline,
};
pub use crate::effects::material::elevation::{MoonMaterialLevel, ShadowLayer};
pub use crate::effects::EffectFactory;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use resource_manager::ResourceManager;
use rendering::renderer::MoonRenderer;
use derive_more::derive::{Deref, DerefMut};

/// Основная структура движка которая содержит рендерер. Конструктор new
/// принимает окно (Которое можно получить через winit), ширину окна и
/// высоту окна. 
/// Пример (new возвращает result, необходимо обработать результат): 
/// let moonwalk = MoonWalk::new(static_window, 1280, 720).unwrap();
/// 
/// Совет: Вы можете получить статичное окно с помощью такого кода
/// let window = event_loop.create_window( ... ).unwrap();
/// let static_window: &'static Window = Box::leak(Box::new(window));

#[derive(Deref, DerefMut)]
pub struct MoonWalk {
    pub renderer: MoonRenderer,
    pub resources: ResourceManager,

    #[deref]
    #[deref_mut]
    surface: MoonSurface,
}

/// Обёртка над u64 для хранения айди шрифта (FontId из модуля textware)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FontAsset(pub u64);

/// Типы выранивания текста. Влево, вправо, по центру и по ширине (строки растягиваются так, чтобы
/// касаться и левого и правого края блока)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justified,
}

#[derive(Debug, Clone)]
pub struct GraphicsInfo {
    pub name: String,
    pub backend: String, // "Vulkan", "Metal", "Dx12"
    pub driver: String,
}

impl MoonWalk {
    #[cfg(not(target_os = "android"))]
    pub fn new(
        window: &(impl HasWindowHandle + HasDisplayHandle),
        width: u32,
        height: u32,
    ) -> Result<Self, error::MoonWalkError> {
        let mut renderer = MoonRenderer::new(window, width, height)?;
        let resources = ResourceManager::new();
        let surface = MoonSurface::new(&mut renderer.context, width, height)?;

        Ok(Self {
            renderer,
            resources,
            surface,
        })
    }

    /// Для android нужен отдельный new из-за AssetManager который необходим
    /// для загрузки шрифтов и текстур
    #[cfg(target_os = "android")]
    pub fn new(
        window: &'static (impl HasWindowHandle + HasDisplayHandle + Send + Sync),
        width: u32, height: u32,
        asset_manager: ndk::asset::AssetManager,
    ) -> Result<Self, error::MoonWalkError> {
        let mut renderer = MoonRenderer::new(window, width, height)?;
        let resources = ResourceManager::new(asset_manager);

        let surface = MoonSurface::new(&mut renderer.context, width, height)?;

        Ok(Self {
            renderer,
            resources,
            surface,
        })
    }

    pub fn get_graphics_info(&self) -> GraphicsInfo {
        // let info = &self.renderer.context.adapter_info;
        
        GraphicsInfo {
            name: "".to_string(),
            backend: "".to_string(),
            driver: "".to_string(),
        }
    }
}
