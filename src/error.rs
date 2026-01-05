//! # Error
//!
//! This module defines the errors that can occur when using the library.
//!
//! ## Example
//!
//! Handling errors from the library.
//!
//! ```rust
//! use mini_oled::error::MiniOledError;
//!
//! fn check_error(result: Result<(), MiniOledError>) {
//!     match result {
//!         Ok(_) => {},
//!         Err(MiniOledError::CommandBufferSizeError) => {
//!             // Handle command buffer overflow
//!         },
//!         Err(MiniOledError::DataBufferSizeError) => {
//!             // Handle data buffer overflow
//!         },
//!         Err(MiniOledError::I2cError(_)) => {
//!             // Handle I2C communication error
//!         },
//!         Err(MiniOledError::SpiBusError(_)) => {
//!             // Handle SPI communication error
//!         },
//!     }
//! }
//! ```

use core::{
    error::Error,
    fmt::{self, Display},
};

use embedded_hal::{i2c, spi};

#[derive(Debug)]
pub enum MiniOledError {
    /// Error when the command buffer size is exceeded.
    CommandBufferSizeError,
    /// Error when the data buffer size is exceeded.
    DataBufferSizeError,
    /// Error wrapping an I2C communication error.
    I2cError(i2c::ErrorKind),
    /// Error wrapping an SPI communication error.
    SpiBusError(spi::ErrorKind),
}

impl Display for MiniOledError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MiniOledError::CommandBufferSizeError => {
                write!(f, "Mini Oled Library Error: Command Buffer Size Exceeded")
            }
            MiniOledError::DataBufferSizeError => {
                write!(f, "Mini Oled Library Error: Data Buffer Size Exceeded")
            }
            MiniOledError::I2cError(error_kind) => {
                write!(f, "Embedded Hal I2C Error: {}", error_kind)
            }
            MiniOledError::SpiBusError(error_kind) => {
                write!(f, "Embedded Hal Spi Bus Error: {}", error_kind)
            }
        }
    }
}

impl Error for MiniOledError {}
