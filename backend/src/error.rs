// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use thiserror::Error;

#[derive(Debug, Error)]
pub enum MoonBackendError {
    #[error("Context not found, please create context")]
    ContextNotFoundError,

    #[error("Command encoder submit failed")]
    EncoderSubmitError,

    #[error("Format is not supported for auto-upload")]
    TextureFormatNotSupportedError,

    #[error("IO error: {0}")]
    IOError(String),
}
