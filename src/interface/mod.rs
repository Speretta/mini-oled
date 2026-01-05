//! # Communication Interface
//!
//! This module defines the `CommunicationInterface` trait and provides implementations for I2C and ~~SPI~~ (planned).
//! It abstracts the underlying hardware communication details.
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

use crate::{command::CommandBuffer, error::MiniOledError};

pub mod i2c;
pub mod spi;

/// Trait representing the communication interface with the display.
///
/// This trait is implemented by `I2cInterface` and `SPIInterface`.
pub trait CommunicationInterface {
    /// Initialize the communication device.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or a `MiniOledError` on failure.
    fn init(&mut self) -> Result<(), MiniOledError>;

    /// Send a command buffer to the device.
    ///
    /// # Arguments
    ///
    /// * `buf` - The command buffer to send.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or a `MiniOledError` on failure.
    fn write_command<const N: usize>(
        &mut self,
        buf: &CommandBuffer<N>,
    ) -> Result<(), MiniOledError>;

    /// Send data to the device.
    ///
    /// # Arguments
    ///
    /// * `buf` - The data buffer to send.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or a `MiniOledError` on failure.
    fn write_data(&mut self, buf: &[u8]) -> Result<(), MiniOledError>;
}
