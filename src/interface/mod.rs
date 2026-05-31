//! # Communication Interface
//!
//! This module defines the [`CommunicationInterface`] trait and provides
//! implementations for I2C and SPI. It abstracts the underlying hardware
//! communication details so that the [`Sh1106`](crate::screen::sh1106::Sh1106)
//! driver can work with either bus type.
//!
//! ## Example
//!
//! Creating an I2C interface.
//!
//! ```rust,ignore
//! use mini_oled::interface::i2c::I2cInterface;
//!
//! // let i2c = ...; // Your embedded-hal I2C driver
//! let interface = I2cInterface::new(i2c, 0x3C);
//! ```

use core::borrow::Borrow;

use crate::{command::CommandBuffer, error::MiniOledError};

pub mod i2c;
pub mod spi;

/// Trait representing the communication interface with the display.
///
/// This trait is implemented by [`I2cInterface`](i2c::I2cInterface) and
/// [`SpiInterface`](spi::SpiInterface). The driver uses it to send command
/// sequences and pixel data without knowing whether the underlying bus is I2C
/// or SPI.
pub trait CommunicationInterface {
    /// Initialize the communication device.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or a [`MiniOledError`] on failure.
    fn init(&mut self) -> Result<(), MiniOledError>;

    /// Send a command buffer to the device.
    ///
    /// Accepts either an owned [`CommandBuffer`] or a reference to one thanks
    /// to the [`Borrow`] bound.
    ///
    /// # Arguments
    ///
    /// * `buf` - The command buffer to send.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or a [`MiniOledError`] on failure.
    fn write_command<const N: usize, B>(&mut self, buf: B) -> Result<(), MiniOledError>
    where
        B: Borrow<CommandBuffer<N>>;

    /// Send a data (pixel) buffer to the device.
    ///
    /// # Arguments
    ///
    /// * `buf` - The pixel data to send. Length must not exceed 128 bytes.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or a [`MiniOledError`] on failure.
    fn write_data(&mut self, buf: &[u8]) -> Result<(), MiniOledError>;
}
