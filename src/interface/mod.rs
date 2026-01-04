use crate::{command::CommandBuffer, error::MiniOledError};

pub mod i2c;
pub mod spi;

pub trait CommunicationInterface {
    /// Initialize device.
    fn init(&mut self) -> Result<(), MiniOledError>;

    /// Send command to device.
    fn write_command<const N: usize>(
        &mut self,
        buf: &CommandBuffer<N>,
    ) -> Result<(), MiniOledError>;

    /// Send data to device.
    fn write_data(&mut self, buf: &[u8]) -> Result<(), MiniOledError>;
}
