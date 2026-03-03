// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use moonwalk_backend::error::MoonBackendError;
use moonpaint::Error as PaintError;

#[derive(Debug, thiserror::Error)]
pub enum TextError {
    #[error("Backend error: {0}")]
    Backend(#[from] MoonBackendError),

    #[error("Moonpaint layout/raster error: {0}")]
    Paint(#[from] PaintError),

    #[error("Font with ID {0} not found")]
    FontNotFound(usize),

    #[error("Atlas allocation failed (Texture full)")]
    AtlasFull,

    #[error("Failed to create atlas texture")]
    AtlasCreationFailed,

    #[error("Atlas bind group is missing")]
    BindGroupMissing,

    #[error("Atlas texture not found in RenderState")]
    TextureNotFound,
    
    #[error("Internal error: {0}")]
    Internal(String),
}
