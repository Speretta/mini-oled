use core::{
    error::Error,
    fmt::{self, Display},
};

use embedded_hal::{i2c, spi};

#[derive(Debug)]
pub enum MiniOledError {
    CommandBufferSizeError,
    DataBufferSizeError,
    I2cError(i2c::ErrorKind),
    SpiBusError(spi::ErrorKind),
}

impl Display for MiniOledError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MiniOledError::CommandBufferSizeError => {
                write!(f, "Mini Oled Library Error: Command Buffer Size Exceeded")
            }
            MiniOledError::DataBufferSizeError => {
                write!(f, "Mini Oled Library Error: Data Buffer Size Exceeded")
            }
            MiniOledError::I2cError(error_kind) => {
                write!(f, "Embedded Hal I2C Error: {}", error_kind)
            }
            MiniOledError::SpiBusError(error_kind) => {
                write!(f, "Embedded Hal Spi Bus Error: {}", error_kind)
            }
        }
    }
}

impl Error for MiniOledError {}
