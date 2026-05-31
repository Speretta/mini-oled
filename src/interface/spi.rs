//! # SPI Communication Interface
//!
//! This module provides the SPI implementation of [`CommunicationInterface`].
//!
//! ## 4-wire SPI
//!
//! The interface uses a Data/Command (DC) pin to distinguish command bytes
//! from pixel data, and relies on `embedded_hal::spi::SpiDevice` which
//! automatically manages the Chip Select (CS) pin.
//!
//! ## Example: Native `SpiDevice`
//!
//! ```rust,ignore
//! use mini_oled::interface::spi::SpiInterface;
//!
//! // let spi_device = ...;
//! // let dc_pin = ...;
//! let interface = SpiInterface::new(spi_device, dc_pin);
//! ```
//!
//! ## Example: Using with `SpiBus` (Embassy / Older HALs)
//!
//! If your HAL provides an `SpiBus` instead of an `SpiDevice`, you can wrap
//! it with the `embedded-hal-bus` crate.
//!
//! ```rust,ignore
//! use embedded_hal_bus::spi::ExclusiveDevice;
//! use mini_oled::interface::spi::SpiInterface;
//!
//! // let spi_bus = ...;
//! // let cs_pin = ...;
//! // let dc_pin = ...;
//! // let delay = ...;
//!
//! let spi_device = ExclusiveDevice::new(spi_bus, cs_pin, delay).unwrap();
//! let interface = SpiInterface::new(spi_device, dc_pin);
//! ```

use core::borrow::Borrow;

use embedded_hal::{
    digital::{Error as DigitalError, OutputPin},
    spi::{Error as SpiError, SpiDevice},
};

use crate::{
    command::CommandBuffer,
    error::{MiniOledError, SpiErrorType},
};

use super::CommunicationInterface;

/// SPI communication interface (4-wire SPI).
///
/// This struct implements [`CommunicationInterface`] for SPI displays. It
/// utilizes `embedded_hal::spi::SpiDevice`, which guarantees safe sharing of
/// the SPI bus by automatically managing the Chip Select (CS) pin.
///
/// A Data/Command (DC) pin is also required to distinguish between command
/// bytes and pixel data.
pub struct SpiInterface<SPI: SpiDevice, DC: OutputPin> {
    spi: SPI,
    dc: DC,
}

impl<SPI: SpiDevice, DC: OutputPin> SpiInterface<SPI, DC> {
    /// Creates a new SPI interface.
    ///
    /// # Arguments
    ///
    /// * `spi` - The SPI device (handles the bus and CS pin automatically).
    /// * `dc` - The Data/Command output pin.
    pub fn new(spi: SPI, dc: DC) -> Self {
        Self { spi, dc }
    }
}

impl<SPI: SpiDevice, DC: OutputPin> CommunicationInterface for SpiInterface<SPI, DC> {
    fn init(&mut self) -> Result<(), MiniOledError> {
        // Pins and SPI peripherals are assumed to be initialized by the user/HAL.
        Ok(())
    }

    fn write_data(&mut self, buf: &[u8]) -> Result<(), MiniOledError> {
        if buf.len() > 128 {
            return Err(MiniOledError::DataBufferSizeError);
        }

        // Set DC pin HIGH to indicate incoming pixel data.
        self.dc
            .set_high()
            .map_err(|e| MiniOledError::SpiError(SpiErrorType::Pin(e.kind())))?;

        // Write the data buffer.
        // Since we are using `SpiDevice`, the underlying HAL will automatically:
        // 1. Assert the CS pin (LOW)
        // 2. Transmit the data over the SPI bus
        // 3. De-assert the CS pin (HIGH)
        self.spi
            .write(buf)
            .map_err(|e| MiniOledError::SpiError(SpiErrorType::Comm(e.kind())))
    }

    fn write_command<const N: usize, B>(&mut self, buf: B) -> Result<(), MiniOledError>
    where
        B: Borrow<CommandBuffer<N>>,
    {
        let command_buf = buf.borrow();

        let mut send_buf = [0u8; 32];
        let command_bytes = command_buf.encode_to_slice(&mut send_buf)?;

        // Set DC pin LOW to indicate incoming command bytes.
        self.dc
            .set_low()
            .map_err(|e| MiniOledError::SpiError(SpiErrorType::Pin(e.kind())))?;

        // Write the command buffer. CS pin management is automatic.
        self.spi
            .write(command_bytes)
            .map_err(|e| MiniOledError::SpiError(SpiErrorType::Comm(e.kind())))
    }
}
