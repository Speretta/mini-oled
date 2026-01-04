use embedded_hal::i2c::I2c;

use crate::{interface::i2c::I2cInterface, screen::{self, properties::DisplayProperties}};

use super::i2c::I2c0;

#[test]
fn create_sh1106() {
    let i2c = I2c0;
    let i2c = I2cInterface::new(i2c, 0x78);
    let mut screen = screen::sh1106::Sh1106::new(i2c);
    let canvas = screen.get_mut_canvas();

    screen.init();

    screen.set_rotation(screen::properties::DisplayRotation::Rotate0);

}
