use embedded_hal::i2c::{self, I2c, Operation, SevenBitAddress, TenBitAddress};

/// I2C0 hardware peripheral which supports both 7-bit and 10-bit addressing.
pub struct I2c0;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error {
    // ...
}

impl i2c::Error for Error {
    fn kind(&self) -> i2c::ErrorKind {
        match *self {
            // ...
        }
    }
}

impl i2c::ErrorType for I2c0 {
    type Error = Error;
}

impl I2c<SevenBitAddress> for I2c0 {
    fn transaction(&mut self, address: u8, operations: &mut [Operation<'_>]) -> Result<(), Self::Error> {
        // ...
        Ok(())
    }
}

impl I2c<TenBitAddress> for I2c0 {
    fn transaction(&mut self, address: u16, operations: &mut [Operation<'_>]) -> Result<(), Self::Error> {
        // ...
        Ok(())
    }
}