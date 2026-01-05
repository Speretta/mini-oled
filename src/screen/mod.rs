//! # Screen
//!
//! This module contains the screen-related definitions, including the `Canvas` for drawing,
//! `DisplayProperties` for configuration, and the `Sh1106` driver implementation.
//!
//! ## Example
//!
//! Initialize the display and access the canvas.
//!
//! ```rust,ignore
//! use mini_oled::{
//!     interface::i2c::I2cInterface,
//!     screen::{properties::DisplayRotation, sh1106::Sh1106},
//! };
//!
//! // let i2c = ...; // Your I2C driver
//! let i2c_interface = I2cInterface::new(i2c, 0x3C);
//! let mut screen = Sh1106::new(i2c_interface);
//!
//! screen.init().unwrap();
//!
//! let canvas = screen.get_mut_canvas();
//! canvas.set_pixel(10, 10, true);
//! screen.flush().unwrap();
//! ```

pub mod canvas;
pub mod properties;
pub mod sh1106;

macro_rules! fast_mul {
    ($value:expr, $right:expr) => {{
        let value_u32 = ($value) as u32;
        if $right > 0 && ($right & ($right - 1)) == 0 {
            value_u32 << $right.trailing_zeros()
        } else {
            value_u32 * $right
        }
    }};
}

pub(crate) use fast_mul;
