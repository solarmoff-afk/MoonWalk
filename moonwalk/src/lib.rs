// MoonWalk это высокопроизводительный движок основанный на WGPU и предназначенный для
// рендеринга пользовательского интерфейса и игровых 2D сцен. MoonWalk распространяется
// свободно под лицензией EPL 2.0 (Eclipse public license). Подробнее про лицензию
// сказано в файле LICENSE (Корень репозитория). Copyright (с) 2025 MoonWalk
//
// Данный файл предоставляет публичный API рендер движка (В том числе и FFI) для
// использования в других проектах. В этом файле не должна содержаться какая-либо
// логика кроме подключения модулей и объявления публичных функций.
//
// Смотрите подробную документацию здесь: [ССЫЛКА]

// Этот модуль публичный так как используется в тестах
pub mod gpu;

pub mod public;
pub mod error;
pub mod rendering;
pub mod objects;
pub mod resource_manager;
pub mod path;

// abstract зарезервирован в расте поэтому нужно экранирование
pub mod r#abstract;

mod batching;
mod textware;
mod debug;
mod filters;
mod painting;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use resource_manager::ResourceManager;
use path::PathBuilder;

pub use crate::objects::ObjectId;
pub use crate::public::brush::BlendMode;
use crate::rendering::container::RenderContainer;
use crate::rendering::renderer::MoonRenderer;
use crate::error::MoonWalkError;

/// Основная структура движка которая содержит рендерер. Конструктор new
/// принимает окно (Которое можно получить через winit), ширину окна и
/// высоту окна. 
/// Пример (new возвращает result, необходимо обработать результат): 
/// let moonwalk = MoonWalk::new(static_window, 1280, 720).unwrap();
/// 
/// Совет: Вы можете получить статичное окно с помощью такого кода
/// let window = event_loop.create_window( ... ).unwrap();
/// let static_window: &'static Window = Box::leak(Box::new(window));
pub struct MoonWalk {
    renderer: MoonRenderer,
    pub resources: ResourceManager,
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
        let renderer = MoonRenderer::new(window, width, height)?;
        let resources = ResourceManager::new();

        Ok(Self {
            renderer,
            resources,
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
        let renderer = MoonRenderer::new(window, width, height)?;
        let resources = ResourceManager::new(asset_manager);

        Ok(Self {
            renderer,
            resources,
        })
    }

    pub fn get_graphics_info(&self) -> GraphicsInfo {
        let info = &self.renderer.context.adapter_info;
        
        GraphicsInfo {
            name: info.name.clone(),
            backend: format!("{:?}", info.backend),
            driver: info.driver.clone(),
        }
    }
}
