use embedded_hal::spi::SpiBus;

use crate::{command::CommandBuffer, error::MiniOledError};

use super::CommunicationInterface;

/// SPI communication interface.
///
/// This struct implements the `CommunicationInterface` trait for SPI.
///
/// # Example
///
/// ```rust,ignore
/// use mini_oled::interface::spi::SpiInterface;
///
/// // Verify that your SPI driver implements embedded_hal::spi::SpiBus
/// // let spi_driver = ...;
/// let interface = SpiInterface::new(spi_driver);
/// ```
pub struct SpiInterface<SB: SpiBus> {
    _spi_bus: SB,
}

impl<SB: SpiBus> SpiInterface<SB> {
    /// Creates a new SPI interface.
    ///
    /// # Arguments
    ///
    /// * `_spi_bus` - The SPI bus.
    #[allow(unused)]
    pub fn new(_spi_bus: SB) -> Self {
        Self { _spi_bus }
    }
}

impl<SB: SpiBus> CommunicationInterface for SpiInterface<SB> {
    fn init(&mut self) -> Result<(), MiniOledError> {
        Ok(())
    }

    fn write_data(&mut self, _buf: &[u8]) -> Result<(), MiniOledError> {
        todo!()
    }

    fn write_command<const N: usize>(
        &mut self,
        _buf: &CommandBuffer<N>,
    ) -> Result<(), MiniOledError> {
        todo!()
    }
}
