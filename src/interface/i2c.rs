use embedded_hal::i2c::{Error, I2c};

use crate::{command::{Command, CommandBuffer}, error::MiniOledError};

use super::CommunicationInterface;

pub struct I2cInterface<IC: I2c> {
    i2c: IC,
    address: u8,
}

impl<IC: I2c> I2cInterface<IC>{
    pub fn new(i2c: IC, address: u8) -> Self{
        I2cInterface { i2c, address }
    }
}

impl<IC: I2c> CommunicationInterface for I2cInterface<IC> 
{
    fn init(&mut self) -> Result<(), MiniOledError> {
        Ok(())
    }

    fn write_data(&mut self, data_buf: &[u8]) -> Result<(), MiniOledError> {
        let mut send_buf = [0u8; 130];
        if data_buf.len() > 128 {
            return Err(MiniOledError::DataBufferSizeError);
        }
        send_buf[0] = 0x40;
        send_buf[1..data_buf.len() + 1].copy_from_slice(data_buf);
        self.i2c.write(self.address, &send_buf[..data_buf.len()+1]).map_err(|e| MiniOledError::I2cError(e.kind()))
    }
    
    fn write_command<const N: usize>(&mut self, command_buf: &CommandBuffer<N>) -> Result<(), MiniOledError> {
        let mut send_buf = [0u8; 30];
        let command_buf_bytes = command_buf.to_bytes(&mut send_buf[1..])?;
        let len = command_buf_bytes.len();

        self.i2c.write(self.address, &send_buf[..len+1]).map_err(|e| MiniOledError::I2cError(e.kind()))
    }
}


