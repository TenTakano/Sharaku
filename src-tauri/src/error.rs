use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("WebP encode failed")]
    WebpEncode,

    #[error("Not found")]
    NotFound,

    #[error("Invalid template: {0}")]
    InvalidTemplate(String),
}
