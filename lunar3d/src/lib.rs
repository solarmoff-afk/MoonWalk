// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

pub mod core;
pub mod resources;
pub mod tools;

mod internal;
mod factory;
pub mod scene;

pub use factory::LunarFactory;
pub use scene::LunarScene;
pub use core::types::{MeshId, ObjectId, LightId, Vertex3D, LightingModel, ShadowQuality, Light};
pub use resources::{MeshData, Material};
