use thiserror::Error;
use textware::TextError;

#[derive(Debug, Error)]
pub enum MoonWalkError {
    #[error("Failed to request a wgpu adapter")]
    AdapterRequestError,

    #[error("Failed to create easygpu context")]
    ContextCreationError,

    #[error("Failed to compile shader: {0}")]
    ShaderCompilation(String),

    #[error("Text error: {0}")]
    TextError(#[from] TextError),

    #[error("Surface error: {0}")]
    SurfaceError(#[from] wgpu::SurfaceError),

    #[error("Initialization error: {0}")]
    InitializationError(String),
}