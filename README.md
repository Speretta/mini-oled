# mini-oled

This is a fast and simple driver for the SH1106 OLED display. It is designed for bare-metal systems. The main goal is high speed. It uses `embedded-hal` for hardware communication and `embedded-graphics-core`(it's optional but recommended) for drawing. It works well for both simple text and complex animations.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
mini-oled = "0.1.1"
```

## Features

### Available Features

- [✓] **no-std Support**: Designed for bare-metal environments.
- [✓] **I2C Support**: Fully implemented using `embedded-hal`.
- [✓] **embedded-graphics**: Seamless integration for drawing shapes, text, and images.
- [✓] **Highly Optimized**: Algorithmically optimized with branchless programming and fast bitwise math for high performance.
- [✓] **Buffered Display**: Uses ~1KB RAM to create a local frame buffer. **Trade-off**: Higher RAM usage but significantly reduced bus traffic (only changed pixels are sent).
- [✓] **Partial Updates**: Smart "dirty area" tracking ensures efficient refresh rates.
- [✓] **Display Rotation**: Hardware-assisted rotation (0, 90, 180, 270 degrees).
- [✓] **Power Save Mode**: Supports turning the display logic on/off.
- [✓] **Contrast Control**: Programmable display contrast.

### Planned Features

- [ ] **SPI Support**: Currently not implemented.
- [ ] **Async Support**: Planned for future releases.

## Usage

### With `embedded-graphics`

Here is a complete example. It shows how to setup the display, draw shapes, write text, and make a simple animation.

```rust
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, PrimitiveStyle, Rectangle},
    text::{Alignment, Text},
};
use mini_oled::{
    interface::i2c::I2cInterface,
    screen::{properties::DisplayRotation, sh1106::Sh1106},
};
use core::fmt::Write;

// ... setup your hardware I2C driver here ...
// let i2c = ...;

// Create the I2C interface (address 0x3C is common for SH1106)
let i2c_interface = I2cInterface::new(i2c, 0x3C);

// Initialize the display driver
let mut screen = Sh1106::new(i2c_interface);

// Initialize the display
screen.init().unwrap();

// Set rotation
screen.set_rotation(DisplayRotation::Rotate180).unwrap();

// Draw a filled rectangle
let fill = PrimitiveStyle::with_fill(BinaryColor::Off);
Rectangle::new(Point::new(0, 0), Size::new(127, 60))
    .into_styled(fill)
    .draw(screen.get_mut_canvas())
    .unwrap();

// Prepare text style
let character_style = MonoTextStyleBuilder::new()
    .font(&FONT_6X10)
    .text_color(BinaryColor::On)
    .background_color(BinaryColor::Off)
    .build();

let mut i = 0;
let mut old_i = 0;

// Animation loop
loop {
    // 1. Clear previous circle
    Circle::new(Point::new(old_i, 22), 40)
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::Off, 1))
        .draw(screen.get_mut_canvas())
        .unwrap();

    // 2. Draw new circle
    i = (i + 1) % 128;
    Circle::new(Point::new(i, 22), 40)
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(screen.get_mut_canvas())
        .unwrap();

    old_i = i;

    // 3. Update FPS text (example logic)
    let mut fps_string: heapless::String<32> = heapless::String::new();
    write!(fps_string, "Fps: {}", 60).unwrap(); // Example FPS value

    Text::with_alignment(
        &fps_string,
        Point::new(64, 10),
        character_style,
        Alignment::Center,
    )
    .draw(screen.get_mut_canvas())
    .unwrap();

    // 4. Send changes to the display
    screen.flush().unwrap();
}
```

### Without `embedded-graphics`

You can also use the library without `embedded-graphics`. You can change pixels directly using `set_pixel` or by accessing the buffer.

Disable default features in `Cargo.toml`:

```toml
[dependencies]
mini-oled = { version = "0.1.1", default-features = false }
```

Usage:

```rust
use mini_oled::{
    interface::i2c::I2cInterface,
    screen::{properties::DisplayRotation, sh1106::Sh1106},
};

// ... setup your hardware I2C driver here ...
// let i2c = ...;

let i2c_interface = I2cInterface::new(i2c, 0x3C);
let mut screen = Sh1106::new(i2c_interface);
screen.init().unwrap();

// Manually set a pixel at (10, 10)
// This method automatically updates the "dirty area", so flush() is efficient.
screen.get_mut_canvas().set_pixel(10, 10, true);
screen.flush().unwrap();

// Or access the raw buffer directly
let buffer = screen.get_mut_canvas().get_mut_buffer();
// buffer[0] = 0xFF; // Set first 8 pixels on

// IMPORTANT: Changing the buffer directly does NOT update the "dirty area".
// The driver does not know which pixels changed.
// You must use `flush_all()` to send the entire buffer to the display.
screen.flush_all().unwrap();
```

## Credits

This project was **heavily inspired** by this projects:

-   https://github.com/rust-embedded-community/sh1106
-   https://github.com/techmccat/sh1106