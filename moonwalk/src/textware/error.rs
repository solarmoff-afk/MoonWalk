// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use std::fmt;

#[derive(Debug)]
pub enum TextError {
    FontLoading(String),
    Io(std::io::Error),
}

impl fmt::Display for TextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextError::FontLoading(msg) => write!(f, "Font loading error: {}", msg),
            TextError::Io(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl std::error::Error for TextError {}

impl From<std::io::Error> for TextError {
    fn from(err: std::io::Error) -> Self {
        TextError::Io(err)
    }
}