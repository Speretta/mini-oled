# mini-oled

A fast and simple driver for the SH1106 OLED display, designed for bare-metal
`no-std` systems. It uses `embedded-hal` for hardware communication and
optionally integrates with `embedded-graphics-core` for advanced drawing.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
mini-oled = "0.1.3"
```

## Features

### Available Features

- [x] **no-std Support**: Designed for bare-metal environments.
- [x] **I2C Support**: Fully implemented using `embedded-hal`.
- [x] **embedded-graphics**: Seamless integration for drawing shapes, text, and images.
- [x] **Highly Optimized**: Algorithmically optimized with branchless programming and fast bitwise math for high performance.
- [x] **Buffered Display**: Uses ~1KB RAM for a local frame buffer. **Trade-off**: Higher RAM usage but significantly reduced bus traffic (only changed pixels are sent).
- [x] **Partial Updates**: Smart "dirty area" tracking ensures efficient refresh rates.
- [x] **Display Rotation**: Hardware-assisted rotation (0, 90, 180, 270 degrees).
- [x] **Power Save Mode**: Supports turning the display logic on/off.
- [x] **Contrast Control**: Programmable display contrast.
- [x] **Full Configuration**: All hardware settings (multiplex ratio, clock divider, pre-charge, Vcomh, COM pins, etc.) are configurable through a type-safe builder.

### Planned Features

- [ ] **Async Support**: Planned for future releases.

## Usage

### Screen Configuration

The driver is constructed with a [`ScreenConfig`] that holds every hardware-level
setting. Start with [`ScreenConfig::new()`] and chain `with_*` methods to
customize defaults.

```rust
use mini_oled::screen::config::{ScreenConfig, DisplayRotation};

let config = ScreenConfig::new()
    .with_rotation(DisplayRotation::Rotate180)
    .with_contrast(0x80);
```

| Setting | Default | Range |
|---|---|---|
| `contrast` | `0x80` | 0–255 |
| `display_on` | `true` | bool |
| `inverse_display` | `false` | bool |
| `rotation` | `Rotate0` | `Rotate0` / `90` / `180` / `270` |
| `display_offset` | `0` | 0–63 |
| `start_line` | `0` | 0–63 |
| `multiplex_ratio` | `63` | 15–63 |
| `charge_pump_enabled` | `true` | bool |
| `clock_div_osc_freq` | `8` | 0–15 |
| `clock_div_ratio` | `0` | 0–15 |
| `precharge_phase1` | `1` | 0–15 |
| `precharge_phase2` | `15` | 0–15 |
| `vcomh_level` | `Auto` | `V065` / `V077` / `V083` / `Auto` |
| `com_pin_config` | `Alternative` | `Alternative` / `Sequential` |

After the driver is created you can still change popular settings (contrast,
rotation, power, etc.) through the [`Sh1106`] API.

### With `embedded-graphics`

Here is a complete example showing how to set up the display, draw shapes, write
text, and make a simple animation.

```rust
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, PrimitiveStyle, Rectangle},
    text::{Alignment, Text},
};
use mini_oled::prelude::*;
use mini_oled::screen::config::{DisplayRotation, ScreenConfig};
use core::fmt::Write;

// ... setup your hardware I2C driver here ...
// let i2c = ...;

// Create the I2C interface (address 0x3C is common for SH1106)
let i2c_interface = I2cInterface::new(i2c, 0x3C);

// Build the hardware configuration
let config = ScreenConfig::new()
    .with_rotation(DisplayRotation::Rotate180)
    .with_contrast(0x80);

// Initialize the display driver
let mut screen = Sh1106::new(i2c_interface, config);

// Initialize the display hardware
screen.init().unwrap();

// Draw a filled rectangle
let fill = PrimitiveStyle::with_fill(BinaryColor::Off);
Rectangle::new(Point::new(0, 0), Size::new(127, 60))
    .into_styled(fill)
    .draw(screen.canvas_mut())
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
        .draw(screen.canvas_mut())
        .unwrap();

    // 2. Draw new circle
    i = (i + 1) % 128;
    Circle::new(Point::new(i, 22), 40)
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(screen.canvas_mut())
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
    .draw(screen.canvas_mut())
    .unwrap();

    // 4. Send only the changed region to the display
    screen.flush().unwrap();
}
```

### Without `embedded-graphics`

You can also use the library without `embedded-graphics`. Change pixels directly
using [`Canvas::set_pixel`] or by accessing the raw buffer.

Disable default features in `Cargo.toml`:

```toml
[dependencies]
mini-oled = { version = "0.1.3", default-features = false }
```

Usage:

```rust
use mini_oled::prelude::*;
use mini_oled::screen::config::ScreenConfig;

// ... setup your hardware I2C driver here ...
// let i2c = ...;

let i2c_interface = I2cInterface::new(i2c, 0x3C);
let config = ScreenConfig::new();
let mut screen = Sh1106::new(i2c_interface, config);
screen.init().unwrap();

// Manually set a pixel at (10, 10)
// This method automatically updates the "dirty area", so flush() is efficient.
screen.canvas_mut().set_pixel(10, 10, true);
screen.flush().unwrap();

// Or access the raw buffer directly
let buffer = screen.canvas_mut().get_mut_buffer();
// buffer[0] = 0xFF; // Set first 8 pixels on

// IMPORTANT: Changing the buffer directly does NOT update the "dirty area".
// The driver does not know which pixels changed.
// You must use flush_all() to send the entire buffer to the display.
screen.flush_all().unwrap();
```

## API Overview

| Method | Description |
|---|---|
| `Sh1106::new(interface, config)` | Create the driver. |
| `screen.init()` | Send the full setup sequence to the controller. |
| `screen.canvas_mut()` | Get the mutable canvas for drawing. |
| `screen.flush()` | Send only the dirty region to the display. |
| `screen.flush_all()` | Send the entire frame buffer. |
| `screen.set_contrast(v)` | Set contrast (0–255). |
| `screen.set_rotation(r)` | Set hardware rotation. |
| `screen.set_display_on(b)` | Turn the panel on/off (sleep mode). |
| `screen.apply_config(cfg)` | Apply a new `ScreenConfig` efficiently. |

## Credits

This project was **heavily inspired** by these projects:

-   https://github.com/rust-embedded-community/sh1106
-   https://github.com/techmccat/sh1106

## License

This project is licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in `mini-oled` by you shall be dual licensed as above, without any additional terms or conditions.
