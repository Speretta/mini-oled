//! # SH1106 Driver
//!
//! This module contains the main [`Sh1106`] driver struct.
//! It brings together the communication interface and the canvas to control the
//! display.
//!
//! ## Example
//!
//! ```rust,ignore
//! use mini_oled::{
//!     interface::i2c::I2cInterface,
//!     screen::config::{ScreenConfig, DisplayRotation},
//!     screen::sh1106::Sh1106,
//! };
//!
//! // let i2c = ...; // I2C peripheral
//! let interface = I2cInterface::new(i2c, 0x3C);
//! let config = ScreenConfig::new().with_rotation(DisplayRotation::Rotate0);
//! let mut display = Sh1106::new(interface, config);
//!
//! display.init().unwrap();
//! display.enable_test_screen().unwrap();
//! ```

use crate::{
    command::{Command, CommandBuffer, Page},
    error::MiniOledError,
    interface::CommunicationInterface,
    screen::{config::ScreenConfig, fast_mul},
};

use crate::screen::{
    canvas::Canvas,
    config::DisplayRotation,
};

const WIDTH: u32 = 128;
const HEIGHT: u32 = 64;
const OFFSET: u8 = 2;
const BUFFER_SIZE: usize = WIDTH as usize * HEIGHT as usize / 8;

/// The main driver struct for the SH1106 OLED display.
///
/// `Sh1106` owns the communication interface, the drawing [`Canvas`], and the
/// hardware [`ScreenConfig`]. After construction you must call [`Sh1106::init`]
/// to send the initial command sequence to the controller.
///
/// # Example
///
/// ```rust,ignore
/// use mini_oled::{
///     interface::i2c::I2cInterface,
///     screen::config::ScreenConfig,
///     screen::sh1106::Sh1106,
/// };
///
/// // let i2c_interface = ...;
/// let config = ScreenConfig::new();
/// let mut screen = Sh1106::new(i2c_interface, config);
/// screen.init().unwrap();
/// screen.enable_test_screen().unwrap();
/// ```
pub struct Sh1106<CI: CommunicationInterface> {
    communication_interface: CI,
    canvas: Canvas<BUFFER_SIZE, WIDTH, HEIGHT, OFFSET>,
    screen_config: ScreenConfig,
}

impl<CI: CommunicationInterface> Sh1106<CI> {
    /// Creates a new `Sh1106` driver instance.
    ///
    /// # Arguments
    ///
    /// * `communication_interface` - The initialized communication interface
    ///   (e.g. [`I2cInterface`](crate::interface::i2c::I2cInterface) or
    ///   [`SpiInterface`](crate::interface::spi::SpiInterface)).
    /// * `screen_config` - The hardware configuration. Use [`ScreenConfig::new`]
    ///   for sensible defaults and chain `with_*` methods to customize.
    pub fn new(communication_interface: CI, screen_config: ScreenConfig) -> Sh1106<CI> {
        let display_rotation = screen_config.rotation();
        Sh1106 {
            communication_interface,
            canvas: Canvas::new(display_rotation),
            screen_config,
        }
    }

    /// Returns an immutable reference to the drawing canvas.
    pub fn canvas(&self) -> &Canvas<BUFFER_SIZE, WIDTH, HEIGHT, OFFSET> {
        &self.canvas
    }

    /// Returns a mutable reference to the drawing canvas.
    ///
    /// Use this to draw pixels or hand the canvas to
    /// `embedded-graphics` draw targets.
    pub fn canvas_mut(&mut self) -> &mut Canvas<BUFFER_SIZE, WIDTH, HEIGHT, OFFSET> {
        &mut self.canvas
    }

    /// Returns a snapshot of the current hardware configuration.
    pub fn config(&self) -> &ScreenConfig {
        &self.screen_config
    }

    /// Flushes the entire display buffer to the screen, refreshing all pixels.
    ///
    /// This is equivalent to marking the whole canvas dirty and then calling
    /// [`flush`](Self::flush). It is useful after you have modified the raw
    /// pixel buffer directly (e.g. via [`Canvas::get_mut_buffer`]).
    pub fn flush_all(&mut self) -> Result<(), MiniOledError> {
        self.canvas.force_full_dirty_area();
        self.flush()
    }

    /// Flushes only the modified ("dirty") region of the display buffer.
    ///
    /// This is much more efficient than [`flush_all`](Self::flush_all) because
    /// it only sends the pages and columns that actually changed since the last
    /// flush. The dirty area is tracked automatically when you use
    /// [`Canvas::set_pixel`] or draw through `embedded-graphics`.
    pub fn flush(&mut self) -> Result<(), MiniOledError> {
        let ((dirty_min_x, dirty_min_y), (dirty_max_x, dirty_max_y)) = self.canvas.get_dirty_area();

        if dirty_min_x > dirty_max_x || dirty_min_y > dirty_max_y {
            return Ok(());
        }

        let start_page = Page::from((dirty_min_y >> 3) as u8);
        let end_page = Page::from((dirty_max_y >> 3) as u8);

        let pixel_buffer = self.canvas.get_buffer();

        for page in Page::range(start_page, end_page) {
            let page_start_idx = fast_mul!(page, WIDTH) + dirty_min_x;
            let page_end_idx = fast_mul!(page, WIDTH) + dirty_max_x;

            if page_end_idx as usize >= pixel_buffer.len() {
                break;
            }

            let dirty_pixel_buffer = &pixel_buffer[page_start_idx as usize..=page_end_idx as usize];
            let current_column = dirty_min_x + self.canvas.get_column_offset() as u32;
            let commands: CommandBuffer<3> = [
                Command::PageAddress(page),
                Command::ColumnAddressLow(current_column as u8),
                Command::ColumnAddressHigh((current_column >> 4) as u8),
            ]
            .into();

            self.communication_interface.write_command(&commands)?;
            self.communication_interface
                .write_data(dirty_pixel_buffer)?;
        }

        self.canvas.reset_dirty_area();
        Ok(())
    }

    // ------------------------------------------------------------------
    // Popular getters
    // ------------------------------------------------------------------

    /// Returns the current contrast setting (0–255).
    ///
    /// Higher values produce higher contrast. Default is `0x80`.
    pub fn contrast(&self) -> u8 {
        self.screen_config.contrast()
    }

    /// Returns `true` if the display panel is on.
    ///
    /// When `false` the display is in sleep mode (≈ 20 µA) but RAM contents
    /// are preserved.
    pub fn display_on(&self) -> bool {
        self.screen_config.display_on()
    }

    /// Returns `true` if inverse (negative) display mode is active.
    pub fn inverse_display(&self) -> bool {
        self.screen_config.inverse_display()
    }

    /// Returns the current display rotation.
    pub fn rotation(&self) -> DisplayRotation {
        self.screen_config.rotation()
    }

    // ------------------------------------------------------------------
    // High-level helpers
    // ------------------------------------------------------------------

    /// Enables the test-screen mode (all pixels forced on).
    ///
    /// This does **not** overwrite display RAM. Call
    /// [`disable_test_screen`](Self::disable_test_screen) to resume normal
    /// operation.
    pub fn enable_test_screen(&mut self) -> Result<(), MiniOledError> {
        self.communication_interface
            .write_command(&Command::EnableTestScreen.into())
    }

    /// Disables the test-screen mode and resumes displaying RAM contents.
    pub fn disable_test_screen(&mut self) -> Result<(), MiniOledError> {
        self.communication_interface
            .write_command(&Command::DisableTestScreen.into())
    }

    // ------------------------------------------------------------------
    // Visual setters
    // ------------------------------------------------------------------

    /// Sets the display contrast (0–255).
    pub fn set_contrast(&mut self, contrast: u8) -> Result<(), MiniOledError> {
        self.screen_config.set_contrast(contrast);
        self.communication_interface
            .write_command(&Command::Contrast(contrast).into())
    }

    /// Turns the display panel on (`true`) or puts it into sleep mode (`false`).
    pub fn set_display_on(&mut self, on: bool) -> Result<(), MiniOledError> {
        self.screen_config.set_display_on(on);
        self.communication_interface.write_command(
            &if on {
                Command::TurnDisplayOn
            } else {
                Command::TurnDisplayOff
            }
            .into(),
        )
    }

    /// Enables (`true`) or disables (`false`) inverse display mode.
    pub fn set_inverse_display(&mut self, inverse: bool) -> Result<(), MiniOledError> {
        self.screen_config.set_inverse_display(inverse);
        self.communication_interface.write_command(
            &if inverse {
                Command::NegativeImageMode
            } else {
                Command::PositiveImageMode
            }
            .into(),
        )
    }

    /// Sets the display rotation.
    ///
    /// This updates both the hardware configuration and the internal canvas so
    /// that subsequent drawing operations use the new coordinate system.
    pub fn set_rotation(&mut self, display_rotation: DisplayRotation) -> Result<(), MiniOledError> {
        self.canvas.set_rotation(display_rotation);
        self.screen_config.set_rotation(display_rotation);

        let rotation_sequence: CommandBuffer<2> = display_rotation.into();
        self.communication_interface
            .write_command(&rotation_sequence)
    }

    // ------------------------------------------------------------------
    // Position / scanning setters
    // ------------------------------------------------------------------

    /// Sets the vertical display offset (0–63).
    ///
    /// This shifts the image up or down without modifying RAM contents.
    pub fn set_display_offset(&mut self, offset: u8) -> Result<(), MiniOledError> {
        self.screen_config.set_display_offset(offset);
        self.communication_interface
            .write_command(&Command::DisplayOffset(offset).into())
    }

    /// Sets the display start line (0–63).
    ///
    /// This determines which row of the RAM is mapped to the top of the
    /// physical display.
    pub fn set_start_line(&mut self, line: u8) -> Result<(), MiniOledError> {
        self.screen_config.set_start_line(line);
        self.communication_interface
            .write_command(&Command::StartLine(line).into())
    }

    /// Sets the multiplex ratio (15–63).
    ///
    /// The value is `MUX - 1`. For a 64-pixel-tall display the correct value
    /// is `63`.
    pub fn set_multiplex_ratio(&mut self, ratio: u8) -> Result<(), MiniOledError> {
        self.screen_config.set_multiplex_ratio(ratio);
        self.communication_interface
            .write_command(&Command::Multiplex(ratio).into())
    }

    // ------------------------------------------------------------------
    // Power / timing setters
    // ------------------------------------------------------------------

    /// Enables (`true`) or disables (`false`) the built-in charge pump.
    ///
    /// **Note:** The SH1106 datasheet recommends sending this command only
    /// while the display is off.
    pub fn set_charge_pump_enabled(&mut self, enabled: bool) -> Result<(), MiniOledError> {
        self.screen_config.set_charge_pump_enabled(enabled);
        self.communication_interface.write_command(
            &if enabled {
                Command::EnableChargePump
            } else {
                Command::DisableChargePump
            }
            .into(),
        )
    }

    /// Sets the display clock divider.
    ///
    /// # Arguments
    ///
    /// * `osc_freq` - Oscillator frequency, 0–15. Higher values increase
    ///   refresh rate (and power consumption).
    /// * `div_ratio` - Divide ratio minus one, 0–15.
    pub fn set_clock_div(
        &mut self,
        osc_freq: u8,
        div_ratio: u8,
    ) -> Result<(), MiniOledError> {
        self.screen_config.set_clock_div_osc_freq(osc_freq);
        self.screen_config.set_clock_div_ratio(div_ratio);
        self.communication_interface
            .write_command(&Command::DisplayClockDiv(osc_freq, div_ratio).into())
    }

    /// Sets the pre-charge period.
    ///
    /// # Arguments
    ///
    /// * `phase1` - Phase 1 period, 0–15.
    /// * `phase2` - Phase 2 period, 0–15.
    pub fn set_precharge_period(
        &mut self,
        phase1: u8,
        phase2: u8,
    ) -> Result<(), MiniOledError> {
        self.screen_config.set_precharge_phase1(phase1);
        self.screen_config.set_precharge_phase2(phase2);
        self.communication_interface
            .write_command(&Command::PreChargePeriod(phase1, phase2).into())
    }

    /// Sets the Vcomh deselect level.
    ///
    /// This adjusts the COM deselect voltage and can help reduce ghosting.
    pub fn set_vcomh_level(
        &mut self,
        level: crate::command::VcomhLevel,
    ) -> Result<(), MiniOledError> {
        self.screen_config.set_vcomh_level(level);
        self.communication_interface
            .write_command(&Command::VcomhDeselect(level).into())
    }

    /// Sets the COM pin configuration.
    ///
    /// This is a hardware design choice. Most 128×64 modules require
    /// [`ComPinConfig::Alternative`](crate::screen::config::ComPinConfig::Alternative).
    pub fn set_com_pin_config(
        &mut self,
        pin_config: crate::screen::config::ComPinConfig,
    ) -> Result<(), MiniOledError> {
        self.screen_config.set_com_pin_config(pin_config);
        self.communication_interface
            .write_command(&Command::from(pin_config).into())
    }

    /// Applies a new [`ScreenConfig`] by sending only the commands that differ.
    ///
    /// This is useful when you want to update several settings at once without
    /// calling each individual setter. Only changed fields generate bus traffic.
    pub fn apply_config(&mut self, config: ScreenConfig) -> Result<(), MiniOledError> {
        if self.screen_config.contrast() != config.contrast() {
            self.set_contrast(config.contrast())?;
        }
        if self.screen_config.display_on() != config.display_on() {
            self.set_display_on(config.display_on())?;
        }
        if self.screen_config.inverse_display() != config.inverse_display() {
            self.set_inverse_display(config.inverse_display())?;
        }
        if self.screen_config.rotation() != config.rotation() {
            self.set_rotation(config.rotation())?;
        }
        if self.screen_config.display_offset() != config.display_offset() {
            self.set_display_offset(config.display_offset())?;
        }
        if self.screen_config.start_line() != config.start_line() {
            self.set_start_line(config.start_line())?;
        }
        if self.screen_config.multiplex_ratio() != config.multiplex_ratio() {
            self.set_multiplex_ratio(config.multiplex_ratio())?;
        }
        if self.screen_config.charge_pump_enabled() != config.charge_pump_enabled() {
            self.set_charge_pump_enabled(config.charge_pump_enabled())?;
        }
        if self.screen_config.clock_div_osc_freq() != config.clock_div_osc_freq()
            || self.screen_config.clock_div_ratio() != config.clock_div_ratio()
        {
            self.set_clock_div(config.clock_div_osc_freq(), config.clock_div_ratio())?;
        }
        if self.screen_config.precharge_phase1() != config.precharge_phase1()
            || self.screen_config.precharge_phase2() != config.precharge_phase2()
        {
            self.set_precharge_period(config.precharge_phase1(), config.precharge_phase2())?;
        }
        if self.screen_config.vcomh_level() != config.vcomh_level() {
            self.set_vcomh_level(config.vcomh_level())?;
        }
        if self.screen_config.com_pin_config() != config.com_pin_config() {
            self.set_com_pin_config(config.com_pin_config())?;
        }

        Ok(())
    }

    /// Initializes the display with the current [`ScreenConfig`] settings.
    ///
    /// This sends the full setup command sequence (contrast, clock divider,
    /// multiplex ratio, charge pump, rotation, COM pins, etc.) and must be
    /// called once after [`Sh1106::new`] before drawing or flushing.
    pub fn init(&mut self) -> Result<(), MiniOledError> {
        let screen_config = self.screen_config;
        let rotation_command_buffer: CommandBuffer<2> = screen_config.rotation().into();
        let init_sequence: CommandBuffer<15> = [
            Command::TurnDisplayOff,
            Command::DisplayClockDiv(screen_config.clock_div_osc_freq(), screen_config.clock_div_ratio()),
            Command::Multiplex(screen_config.multiplex_ratio()),
            Command::DisplayOffset(screen_config.display_offset()),
            Command::StartLine(screen_config.start_line()),
            if screen_config.charge_pump_enabled() { Command::EnableChargePump } else { Command::DisableChargePump },
            rotation_command_buffer[0],
            rotation_command_buffer[1],
            screen_config.com_pin_config().into(),
            Command::Contrast(screen_config.contrast()),
            Command::PreChargePeriod(screen_config.precharge_phase1(), screen_config.precharge_phase2()),
            Command::VcomhDeselect(screen_config.vcomh_level()),
            Command::DisableTestScreen,
            if screen_config.inverse_display() { Command::NegativeImageMode } else { Command::PositiveImageMode},
            if screen_config.display_on() { Command::TurnDisplayOn } else { Command::TurnDisplayOff },
        ]
        .into();

        self.communication_interface.write_command(&init_sequence)
    }
}
