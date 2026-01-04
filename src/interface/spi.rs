use embedded_hal::spi::{Error, SpiBus};

use crate::{command::{Command, CommandBuffer}, error::MiniOledError};

use super::CommunicationInterface;

pub struct SPIInterface<SB: SpiBus> {
    spi_bus: SB,
}


impl<SB: SpiBus> SPIInterface<SB>{
    fn new(spi_bus: SB) -> Self {
        Self { spi_bus }
    }
}

impl<SB: SpiBus> CommunicationInterface for SPIInterface<SB> 
{
    fn init(&mut self) -> Result<(), MiniOledError> {
        Ok(())
    }

    fn write_data(&mut self, buf: &[u8]) -> Result<(), MiniOledError> {
        todo!()
    }
    
    fn write_command<const N: usize>(&mut self, buf: &CommandBuffer<N>) -> Result<(), MiniOledError>{
        todo!()
    }
}



