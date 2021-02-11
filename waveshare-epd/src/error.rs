use rppal::{gpio, spi};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, EpdError>;

#[derive(Error, Debug)]
pub enum EpdError {
    #[error(transparent)]
    Gpio(#[from] gpio::Error),
    #[error(transparent)]
    Spi(#[from] spi::Error),
}
