//! Prelude - Commonly used types
//!
//! This module re-exports the most commonly used types and traits for convenience.
//!
//! # Example
//!
//! ```rust,ignore
//! use mini_oled::prelude::*;
//! ```

pub use crate::error::MiniOledError;
pub use crate::interface::i2c::I2cInterface;
pub use crate::interface::spi::SpiInterface;
pub use crate::screen::properties::{DisplayProperties, DisplayRotation};
pub use crate::screen::sh1106::Sh1106;
