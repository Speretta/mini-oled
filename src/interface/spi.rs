use embedded_hal::spi::SpiBus;

use crate::{command::CommandBuffer, error::MiniOledError};

use super::CommunicationInterface;

pub struct SPIInterface<SB: SpiBus> {
    _spi_bus: SB,
}

impl<SB: SpiBus> SPIInterface<SB> {
    #[allow(unused)]
    fn new(_spi_bus: SB) -> Self {
        Self { _spi_bus }
    }
}

impl<SB: SpiBus> CommunicationInterface for SPIInterface<SB> {
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
