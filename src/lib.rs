#![no_std]
//! # Mini OLED
//!
//! `mini-oled` is an I2C/SPI driver for the SH1106 OLED display controller,
//! designed for embedded `no-std` environments. It supports basic drawing
//! operations and integrates with `embedded-graphics` for advanced graphics.
//!
//! ## Usage
//!
//! ### With `embedded-graphics`
//!
//! The driver is constructed from a communication interface and a
//! [`ScreenConfig`]. After calling [`Sh1106::init`], you can draw shapes,
//! text, and images through `embedded-graphics`.
//!
//! ```rust,ignore
//! use embedded_graphics::{
//!     mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
//!     pixelcolor::BinaryColor,
//!     prelude::*,
//!     primitives::{Circle, PrimitiveStyle, Rectangle},
//!     text::{Alignment, Text},
//! };
//! use mini_oled::prelude::*;
//! use mini_oled::screen::config::ScreenConfig;
//! use core::fmt::Write;
//!
//! // ... setup your hardware I2C driver here ...
//! let i2c = /* ... */;
//!
//! // Create the I2C interface (address 0x3C is common for SH1106)
//! let i2c_interface = I2cInterface::new(i2c, 0x3C);
//!
//! // Build the screen configuration with your preferred defaults
//! let config = ScreenConfig::new()
//!     .with_rotation(DisplayRotation::Rotate180)
//!     .with_contrast(0x80);
//!
//! // Initialize the display driver
//! let mut screen = Sh1106::new(i2c_interface, config);
//!
//! // Initialize the display hardware
//! screen.init().unwrap();
//!
//! // Draw a filled rectangle
//! let fill = PrimitiveStyle::with_fill(BinaryColor::Off);
//! Rectangle::new(Point::new(0, 0), Size::new(127, 60))
//!     .into_styled(fill)
//!     .draw(screen.canvas_mut())
//!     .unwrap();
//!
//! // Prepare text style
//! let character_style = MonoTextStyleBuilder::new()
//!     .font(&FONT_6X10)
//!     .text_color(BinaryColor::On)
//!     .background_color(BinaryColor::Off)
//!     .build();
//!
//! let mut i = 0;
//! let mut old_i = 0;
//!
//! // Animation loop
//! loop {
//!     // 1. Clear previous circle
//!     Circle::new(Point::new(old_i, 22), 40)
//!         .into_styled(PrimitiveStyle::with_stroke(BinaryColor::Off, 1))
//!         .draw(screen.canvas_mut())
//!         .unwrap();
//!
//!     // 2. Draw new circle
//!     i = (i + 1) % 128;
//!     Circle::new(Point::new(i, 22), 40)
//!         .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
//!         .draw(screen.canvas_mut())
//!         .unwrap();
//!
//!     old_i = i;
//!
//!     // 3. Update FPS text (example logic)
//!     let mut fps_string: heapless::String<32> = heapless::String::new();
//!     write!(fps_string, "Fps: {}", 60).unwrap(); // Example FPS value
//!
//!     Text::with_alignment(
//!         &fps_string,
//!         Point::new(64, 10),
//!         character_style,
//!         Alignment::Center,
//!     )
//!     .draw(screen.canvas_mut())
//!     .unwrap();
//!
//!     // 4. Send changes to the display
//!     screen.flush().unwrap();
//! }
//! ```
//!
//! ### Without `embedded-graphics`
//!
//! You can also use the library without `embedded-graphics`. You can change
//! pixels directly using [`Canvas::set_pixel`] or by accessing the raw buffer.
//!
//! Disable default features in `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! mini-oled = { version = "0.1.3", default-features = false }
//! ```
//!
//! ```rust,ignore
//! use mini_oled::prelude::*;
//! use mini_oled::screen::config::ScreenConfig;
//!
//! // ... setup your hardware I2C driver here ...
//! let i2c = /* ... */;
//!
//! let i2c_interface = I2cInterface::new(i2c, 0x3C);
//! let config = ScreenConfig::new();
//! let mut screen = Sh1106::new(i2c_interface, config);
//! screen.init().unwrap();
//!
//! // Manually set a pixel at (10, 10)
//! // This method automatically updates the "dirty area", so flush() is efficient.
//! screen.canvas_mut().set_pixel(10, 10, true);
//! screen.flush().unwrap();
//!
//! // Or access the raw buffer directly
//! let buffer = screen.canvas_mut().get_mut_buffer();
//! // buffer[0] = 0xFF; // Set first 8 pixels on
//!
//! // IMPORTANT: Changing the buffer directly does NOT update the "dirty area".
//! // The driver does not know which pixels changed.
//! // You must use `flush_all()` to send the entire buffer to the display.
//! screen.flush_all().unwrap();
//! ```

pub mod command;
pub mod error;
pub mod interface;
pub mod prelude;
pub mod screen;

mod tests;
