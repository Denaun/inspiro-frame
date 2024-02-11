#[cfg(feature = "esp")]
use esp_idf_hal::sys::EspError;
#[cfg(feature = "rpi")]
use rppal::{gpio, spi};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, EpdError>;

#[derive(Error, Debug)]
pub enum EpdError {
    #[cfg(feature = "rpi")]
    #[error(transparent)]
    Gpio(#[from] gpio::Error),
    #[cfg(feature = "rpi")]
    #[error(transparent)]
    Spi(#[from] spi::Error),
    #[cfg(feature = "esp")]
    #[error(transparent)]
    Esp(#[from] EspError),
}
