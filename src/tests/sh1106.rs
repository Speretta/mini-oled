#[allow(unused)]
use crate::{interface::i2c::I2cInterface, screen, tests::i2c::I2c0};

#[test]
fn create_sh1106() {
    let i2c = I2c0;
    let i2c = I2cInterface::new(i2c, 0x78);
    let mut screen = screen::sh1106::Sh1106::new(i2c);
    let _canvas = screen.get_mut_canvas();

    screen.init().unwrap();

    screen
        .set_rotation(screen::properties::DisplayRotation::Rotate0)
        .unwrap();
}
