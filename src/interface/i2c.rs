//! # I2C Communication Interface
//!
//! This module provides the I2C implementation of [`CommunicationInterface`].
//! It targets the `embedded-hal` `I2c` trait and is the primary way to talk to
//! SH1106 modules on a two-wire bus.
//!
//! ## Example
//!
//! ```rust,ignore
//! use mini_oled::interface::i2c::I2cInterface;
//!
//! // Verify that your I2C driver implements embedded_hal::i2c::I2c
//! // let i2c_driver = ...;
//! let interface = I2cInterface::new(i2c_driver, 0x3C);
//! ```

use core::borrow::Borrow;

use embedded_hal::i2c::{Error, I2c};

use crate::{command::CommandBuffer, error::MiniOledError};

use super::CommunicationInterface;

/// I2C communication interface.
///
/// # Example
///
/// ```rust,ignore
/// use mini_oled::interface::i2c::I2cInterface;
///
/// // let i2c_driver = ...;
/// let interface = I2cInterface::new(i2c_driver, 0x3C);
/// ```
pub struct I2cInterface<IC: I2c> {
    i2c: IC,
    address: u8,
}

impl<IC: I2c> I2cInterface<IC> {
    /// Creates a new I2C interface.
    ///
    /// # Arguments
    ///
    /// * `i2c` - The I2C peripheral.
    /// * `address` - The 7-bit I2C address of the display. `0x3C` and `0x3D`
    ///   are common for SH1106 modules.
    pub fn new(i2c: IC, address: u8) -> Self {
        I2cInterface { i2c, address }
    }
}

impl<IC: I2c> CommunicationInterface for I2cInterface<IC> {
    fn init(&mut self) -> Result<(), MiniOledError> {
        Ok(())
    }

    fn write_data(&mut self, data_buf: &[u8]) -> Result<(), MiniOledError> {
        let mut send_buf = [0u8; 130];
        if data_buf.len() > 128 {
            return Err(MiniOledError::DataBufferSizeError);
        }
        send_buf[0] = 0x40; // I2C data control byte
        send_buf[1..data_buf.len() + 1].copy_from_slice(data_buf);
        self.i2c
            .write(self.address, &send_buf[..data_buf.len() + 1])
            .map_err(|e| MiniOledError::I2cError(e.kind()))
    }

    fn write_command<const N: usize, B>(
        &mut self,
        command_buf: B,
    ) -> Result<(), MiniOledError>
    where
        B: Borrow<CommandBuffer<N>>,
    {
        let command_buf = command_buf.borrow();
        let mut send_buf = [0u8; 30];
        // The first byte (index 0) is reserved for the I2C control byte.
        let command_buf_bytes = command_buf.encode_to_slice(&mut send_buf[1..])?;
        let len = command_buf_bytes.len();

        self.i2c
            .write(self.address, &send_buf[..len + 1])
            .map_err(|e| MiniOledError::I2cError(e.kind()))
    }
}
