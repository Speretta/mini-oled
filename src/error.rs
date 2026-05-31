//! # Error Types
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
//!         Err(MiniOledError::SpiError(_)) => {
//!             // Handle SPI communication error
//!         },
//!     }
//! }
//! ```

use core::{
    error::Error,
    fmt::{self, Display},
};

use embedded_hal::{digital, i2c, spi};

/// Errors that can be returned by the driver or its communication interfaces.
#[derive(Debug)]
pub enum MiniOledError {
    /// The provided byte slice was too small to hold the serialized command
    /// sequence.
    CommandBufferSizeError,
    /// The pixel data buffer exceeded the maximum length allowed for a single
    /// bus transaction (128 bytes).
    DataBufferSizeError,
    /// An I2C communication error occurred.
    I2cError(i2c::ErrorKind),
    /// An SPI communication or pin-control error occurred.
    SpiError(SpiErrorType),
}

/// SPI-specific error kinds.
#[derive(Debug)]
pub enum SpiErrorType {
    /// An SPI bus communication error.
    Comm(spi::ErrorKind),
    /// A digital pin (e.g. DC) manipulation error.
    Pin(digital::ErrorKind),
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
            MiniOledError::SpiError(spi_error_type) => match spi_error_type {
                SpiErrorType::Comm(error_kind) => {
                    write!(f, "Embedded Hal Spi Bus Error: {}", error_kind)
                }

                SpiErrorType::Pin(error_kind) => {
                    write!(f, "Embedded Hal Spi Pin Error: {}", error_kind)
                }
            },
        }
    }
}

impl Error for MiniOledError {}
