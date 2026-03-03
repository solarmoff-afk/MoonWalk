// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use thiserror::Error;

#[derive(Debug)]
pub enum TypedBackendError {
    SurfaceLost,
    OutOfMemory,
    SurfaceTimeout,
    Internal(String),
    Validation(String),
    ContextNotFound,
    BindGroupNotFound,
    NoSuitableSurfaceFormat,
    Other(String),
}

impl From<&str> for TypedBackendError {
    /// Этот блок кода используется в moonwalk_bootstrap для преобразования
    /// строки в тип из удобного перечисления. Это нужно на случай смены
    /// бэкенда в moonwalk_backend и для удобства чтобы не плодить хардкод
    /// в moonwalk_bootstrap
    fn from(s: &str) -> Self {
        if s.contains("OutOfMemory") {
            TypedBackendError::OutOfMemory
        } else if s.contains("Lost") {
            TypedBackendError::SurfaceLost
        } else if s.contains("Timeout") {
            TypedBackendError::SurfaceTimeout
        } else if s.contains("Context not found") {
            TypedBackendError::ContextNotFound
        } else if s.contains("Bind group not found") {
            TypedBackendError::BindGroupNotFound
        } else if s.contains("No suitable surface format") {
            TypedBackendError::NoSuitableSurfaceFormat
        } else if s.contains("Internal") {
            TypedBackendError::Internal(s.to_string())
        } else if s.contains("Validation") {
            TypedBackendError::Validation(s.to_string())
        } else {
            TypedBackendError::Other(s.to_string())
        }
    }
}

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

    #[error("Undefined render surface error: {0}")]
    SurfaceError(String),

    #[error("Context not found error")]
    ContextNotFoundError,

    #[error("Bind group not found error")]
    BindGroupNotFoundError,

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

    #[error("MoonWalk gpu backend error: {0}")]
    BackendError(String),

    #[error("Text error: {0}")]
    TextError(#[from] crate::text::TextError),
}

impl From<moonwalk_backend::error::MoonBackendError> for MoonWalkError {
    fn from(err: moonwalk_backend::error::MoonBackendError) -> Self {
        MoonWalkError::BackendError(err.to_string())
    }
}
