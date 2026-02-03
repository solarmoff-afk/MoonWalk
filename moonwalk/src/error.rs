// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use thiserror::Error;

#[cfg(not(feature = "modern"))]
use wgpu::{CreateSurfaceError, SurfaceError};

#[cfg(not(feature = "modern"))]
#[derive(Debug, Error)]
pub enum MoonWalkError {
    #[error("Failed to request a wgpu adapter")]
    AdapterRequestError,

    #[error("Failed to request a wgpu device")]
    DeviceRequestError(#[from] wgpu::RequestDeviceError),
    
    #[error("Failed to create wgpu surface")]
    CreateSurfaceError(#[from] CreateSurfaceError),

    #[error("Render surface error: {0}")]
    SurfaceError(#[from] SurfaceError),

    #[error("No suitable surface format found")]
    NoSuitableSurfaceFormat,

    #[error("Failed to compile shader: {0}")]
    ShaderCompilation(String),

    #[error("Failed to load font: {0}")]
    FontLoading(String),

    #[error("Failed to load texture: {0}")]
    TextureLoading(String),

    #[error("IO/Asset error: {0}")]
    IOError(String),

    #[error("Shader error: {0}")]
    ShaderError(String),

    #[error("Text error: {0}")]
    TextError(#[from] crate::textware::TextError),
}

#[cfg(feature = "modern")]
#[derive(Debug, Error)]
pub enum MoonWalkError {
    #[error("Failed to request a wgpu adapter")]
    AdapterRequestError,

    #[error("Failed to request a wgpu device")]
    DeviceRequestError,
    
    #[error("Failed to create wgpu surface")]
    CreateSurfaceError,

    #[error("Render surface lost")]
    SurfaceLostError,

    #[error("Out of memory error")]
    OutOfMemoryError,

    #[error("Timeout error")]
    TimeoutError,

    #[error("Undefined render surface error: {}")]
    SurfaceError(String),

    #[error("No suitable surface format found")]
    NoSuitableSurfaceFormat,

    #[error("Failed to compile shader: {0}")]
    ShaderCompilation(String),

    #[error("Failed to load font: {0}")]
    FontLoading(String),

    #[error("Failed to load texture: {0}")]
    TextureLoading(String),

    #[error("IO/Asset error: {0}")]
    IOError(String),

    #[error("Shader error: {0}")]
    ShaderError(String),

    #[error("Text error: {0}")]
    TextError(#[from] crate::textware::TextError),
}