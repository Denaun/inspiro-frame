#[cfg(feature = "epd_12in48b")]
pub mod epd_12in48b;
#[cfg(feature = "epd_2in7b")]
pub mod epd_2in7b;
mod error;

pub use error::{EpdError as Error, Result};
