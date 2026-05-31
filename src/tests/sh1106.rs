#[test]
fn create_sh1106() {
    use crate::screen::config::ScreenConfig;
    use crate::{interface::i2c::I2cInterface, screen, tests::i2c::I2c0};

    let i2c = I2c0;
    let i2c = I2cInterface::new(i2c, 0x78);
    let screen_config = ScreenConfig::default();
    let mut screen = screen::sh1106::Sh1106::new(i2c, screen_config);
    let _canvas = screen.canvas_mut();

    screen.init().unwrap();

    screen
        .set_rotation(screen::config::DisplayRotation::Rotate0)
        .unwrap();
}
