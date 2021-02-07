use thiserror::Error;

pub type Result<T> = std::result::Result<T, ApiError>;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error(transparent)]
    Request(#[from] reqwest::Error),
    #[error(transparent)]
    Image(#[from] image::ImageError),
}
