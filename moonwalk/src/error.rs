// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use thiserror::Error;
use wgpu::CreateSurfaceError;

#[derive(Debug, Error)]
pub enum MoonWalkError {
    #[error("Failed to request a wgpu adapter")]
    AdapterRequestError,

    #[error("Failed to request a wgpu device")]
    DeviceRequestError(#[from] wgpu::RequestDeviceError),
    
    #[error("Failed to create wgpu surface")]
    CreateSurfaceError(#[from] CreateSurfaceError),

    #[error("No suitable surface format found")]
    NoSuitableSurfaceFormat,

    #[error("Failed to compile shader: {0}")]
    ShaderCompilation(String),

    #[error("Failed to load font: {0}")]
    FontLoading(String),
}