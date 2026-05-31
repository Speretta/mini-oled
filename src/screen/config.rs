//! # Screen Configuration
//!
//! This module contains [`ScreenConfig`], a single struct that holds every
//! hardware-level setting for the SH1106 display.
//!
//! All fields are private. You create a config by starting with
//! [`ScreenConfig::new()`] and chaining [`with_*`](ScreenConfig) builder
//! methods. Once built, the driver owns it and you change settings through
//! the [`Sh1106`](crate::screen::sh1106::Sh1106) API.
//!
//! ## Example
//!
//! ```rust,ignore
//! use mini_oled::screen::config::{ScreenConfig, ComPinConfig, DisplayRotation};
//! use mini_oled::command::VcomhLevel;
//!
//! let cfg = ScreenConfig::new()
//!     .with_contrast(0xFF)
//!     .with_rotation(DisplayRotation::Rotate180)
//!     .with_inverse_display(true);
//!
//! let mut screen = Sh1106::new(i2c, cfg);
//! ```

use crate::command::{Command, CommandBuffer, VcomhLevel};

/// How the COM (common) pins are wired to the screen glass.
///
/// This is a factory/hardware design choice. You usually do not change it
/// after you receive the display module, because it depends on how the
/// manufacturer wired the panel.
///
/// If you set the wrong one, the image may look vertically split or garbled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComPinConfig {
    /// **Recommended default.**
    ///
    /// The driver alternates between COM pins. This matches most 128×64
    /// OLED modules you buy online.
    Alternative,
    /// Sequential scanning.
    ///
    /// COM0, COM1, COM2 ... are used in strict order. Rarely needed for
    /// standard modules, but available if your custom hardware requires it.
    Sequential,
}

impl From<ComPinConfig> for Command {
    fn from(value: ComPinConfig) -> Self {
        match value {
            ComPinConfig::Alternative => Command::AlternativeComPinConfig,
            ComPinConfig::Sequential => Command::SequentialComPinConfig,
        }
    }
}

/// Display rotation configuration.
///
/// # Example
///
/// ```rust
/// use mini_oled::screen::config::DisplayRotation;
///
/// let rotation = DisplayRotation::Rotate90;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayRotation {
    /// No rotation, normal display.
    Rotate0,
    /// Rotate by 90 degrees clockwise.
    Rotate90,
    /// Rotate by 180 degrees clockwise.
    Rotate180,
    /// Rotate by 270 degrees clockwise.
    Rotate270,
}

impl From<DisplayRotation> for CommandBuffer<2> {
    fn from(value: DisplayRotation) -> Self {
        match value {
            DisplayRotation::Rotate0 => [Command::EnableSegmentRemap, Command::EnableReverseComDir],
            DisplayRotation::Rotate90 => {
                [Command::DisableSegmentRemap, Command::EnableReverseComDir]
            }
            DisplayRotation::Rotate180 => {
                [Command::DisableSegmentRemap, Command::DisableReverseComDir]
            }
            DisplayRotation::Rotate270 => {
                [Command::EnableSegmentRemap, Command::DisableReverseComDir]
            }
        }
        .into()
    }
}

/// All hardware settings that define how the SH1106 drives the screen.
///
/// Fields are private. Use the consuming builder methods (`with_*`) to
/// construct a config, then pass ownership to the driver.
///
/// # Example
///
/// ```rust,ignore
/// use mini_oled::screen::config::{ScreenConfig, ComPinConfig, DisplayRotation};
///
/// let cfg = ScreenConfig::new()
///     .with_contrast(0xFF)
///     .with_rotation(DisplayRotation::Rotate180);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenConfig {
    // ================================================================
    //  VISUAL SETTINGS
    // ================================================================
    contrast: u8,
    display_on: bool,
    inverse_display: bool,
    rotation: DisplayRotation,
    // ================================================================
    //  SCANNING & POSITION
    // ================================================================
    display_offset: u8,
    start_line: u8,
    pub(crate) multiplex_ratio: u8,
    // ================================================================
    //  POWER & TIMING
    // ================================================================
    charge_pump_enabled: bool,
    clock_div_osc_freq: u8,
    clock_div_ratio: u8,
    precharge_phase1: u8,
    precharge_phase2: u8,
    vcomh_level: VcomhLevel,
    com_pin_config: ComPinConfig,
}

impl ScreenConfig {
    /// Creates a new config with factory-safe defaults.
    ///
    /// | Setting | Default |
    /// |---|---|
    /// | contrast | `0x80` |
    /// | display_on | `true` |
    /// | inverse_display | `false` |
    /// | rotation | [`DisplayRotation::Rotate0`] |
    /// | display_offset | `0` |
    /// | start_line | `0` |
    /// | multiplex_ratio | `63` |
    /// | charge_pump_enabled | `true` |
    /// | clock_div_osc_freq | `8` |
    /// | clock_div_ratio | `0` |
    /// | precharge_phase1 | `1` |
    /// | precharge_phase2 | `15` |
    /// | vcomh_level | [`VcomhLevel::Auto`] |
    /// | com_pin_config | [`ComPinConfig::Alternative`] |
    ///
    /// Equivalent to [`ScreenConfig::default()`].
    pub const fn new() -> Self {
        Self {
            contrast: 0x80,
            display_on: true,
            inverse_display: false,
            rotation: DisplayRotation::Rotate0,
            display_offset: 0,
            start_line: 0,
            multiplex_ratio: 63,
            charge_pump_enabled: true,
            clock_div_osc_freq: 8,
            clock_div_ratio: 0,
            precharge_phase1: 1,
            precharge_phase2: 15,
            vcomh_level: VcomhLevel::Auto,
            com_pin_config: ComPinConfig::Alternative,
        }
    }

    // ----------------------------------------------------------------
    // Getters
    // ----------------------------------------------------------------

    /// Display contrast, 0–255. Higher is higher contrast.
    pub const fn contrast(&self) -> u8 {
        self.contrast
    }

    /// `true` when the display panel is on.
    pub const fn display_on(&self) -> bool {
        self.display_on
    }

    /// `true` when inverse (negative) display mode is active.
    pub const fn inverse_display(&self) -> bool {
        self.inverse_display
    }

    /// Current display rotation.
    pub const fn rotation(&self) -> DisplayRotation {
        self.rotation
    }

    /// Vertical display offset, 0–63.
    pub const fn display_offset(&self) -> u8 {
        self.display_offset
    }

    /// Display start line, 0–63.
    pub const fn start_line(&self) -> u8 {
        self.start_line
    }

    /// Multiplex ratio, 15–63. Value is `MUX - 1`.
    pub const fn multiplex_ratio(&self) -> u8 {
        self.multiplex_ratio
    }

    /// `true` when the built-in charge pump is enabled.
    pub const fn charge_pump_enabled(&self) -> bool {
        self.charge_pump_enabled
    }

    /// Oscillator frequency component of the clock divider, 0–15.
    pub const fn clock_div_osc_freq(&self) -> u8 {
        self.clock_div_osc_freq
    }

    /// Divide-ratio component of the clock divider, 0–15. Value is `ratio - 1`.
    pub const fn clock_div_ratio(&self) -> u8 {
        self.clock_div_ratio
    }

    /// Pre-charge phase 1 period, 0–15.
    pub const fn precharge_phase1(&self) -> u8 {
        self.precharge_phase1
    }

    /// Pre-charge phase 2 period, 0–15.
    pub const fn precharge_phase2(&self) -> u8 {
        self.precharge_phase2
    }

    /// Vcomh deselect level.
    pub const fn vcomh_level(&self) -> VcomhLevel {
        self.vcomh_level
    }

    /// COM pin configuration.
    pub const fn com_pin_config(&self) -> ComPinConfig {
        self.com_pin_config
    }

    // ----------------------------------------------------------------
    // Setters (pub(crate) – mutated by the driver, not by user code)
    // ----------------------------------------------------------------

    pub(crate) const fn set_contrast(&mut self, contrast: u8) {
        self.contrast = contrast;
    }

    pub(crate) const fn set_display_on(&mut self, display_on: bool) {
        self.display_on = display_on;
    }

    pub(crate) const fn set_inverse_display(&mut self, inverse_display: bool) {
        self.inverse_display = inverse_display;
    }

    pub(crate) const fn set_rotation(&mut self, display_rotation: DisplayRotation) {
        self.rotation = display_rotation;
    }

    pub(crate) const fn set_display_offset(&mut self, display_offset: u8) {
        self.display_offset = display_offset;
    }

    pub(crate) const fn set_start_line(&mut self, start_line: u8) {
        self.start_line = start_line;
    }

    pub(crate) const fn set_multiplex_ratio(&mut self, multiplex_ratio: u8) {
        self.multiplex_ratio = multiplex_ratio;
    }

    pub(crate) const fn set_charge_pump_enabled(&mut self, charge_pump_enabled: bool) {
        self.charge_pump_enabled = charge_pump_enabled;
    }

    pub(crate) const fn set_clock_div_osc_freq(&mut self, clock_div_osc_freq: u8) {
        self.clock_div_osc_freq = clock_div_osc_freq;
    }

    pub(crate) const fn set_clock_div_ratio(&mut self, clock_div_ratio: u8) {
        self.clock_div_ratio = clock_div_ratio;
    }

    pub(crate) const fn set_precharge_phase1(&mut self, precharge_phase1: u8) {
        self.precharge_phase1 = precharge_phase1;
    }

    pub(crate) const fn set_precharge_phase2(&mut self, precharge_phase2: u8) {
        self.precharge_phase2 = precharge_phase2;
    }

    pub(crate) const fn set_vcomh_level(&mut self, vcomh_level: VcomhLevel) {
        self.vcomh_level = vcomh_level;
    }

    pub(crate) const fn set_com_pin_config(&mut self, com_pin_config: ComPinConfig) {
        self.com_pin_config = com_pin_config;
    }

    // ----------------------------------------------------------------
    // Consuming builders
    // ----------------------------------------------------------------

    /// Sets the contrast (0–255). Default: `0x80`.
    pub const fn with_contrast(mut self, value: u8) -> Self {
        self.contrast = value;
        self
    }

    /// Turns the display on (`true`) or off (`false`). Default: `true`.
    pub const fn with_display_on(mut self, value: bool) -> Self {
        self.display_on = value;
        self
    }

    /// Enables (`true`) or disables (`false`) inverse display mode. Default: `false`.
    pub const fn with_inverse_display(mut self, value: bool) -> Self {
        self.inverse_display = value;
        self
    }

    /// Sets the rotation. Default: [`DisplayRotation::Rotate0`].
    pub const fn with_rotation(mut self, value: DisplayRotation) -> Self {
        self.rotation = value;
        self
    }

    /// Sets the vertical display offset (0–63). Default: `0`.
    pub const fn with_display_offset(mut self, value: u8) -> Self {
        self.display_offset = value;
        self
    }

    /// Sets the display start line (0–63). Default: `0`.
    pub const fn with_start_line(mut self, value: u8) -> Self {
        self.start_line = value;
        self
    }

    /// Sets the multiplex ratio (15–63). Default: `63`.
    pub const fn with_multiplex_ratio(mut self, value: u8) -> Self {
        self.multiplex_ratio = value;
        self
    }

    /// Enables (`true`) or disables (`false`) the charge pump. Default: `true`.
    pub const fn with_charge_pump_enabled(mut self, value: bool) -> Self {
        self.charge_pump_enabled = value;
        self
    }

    /// Sets the oscillator frequency component (0–15). Default: `8`.
    pub const fn with_clock_div_osc_freq(mut self, value: u8) -> Self {
        self.clock_div_osc_freq = value;
        self
    }

    /// Sets the divide-ratio component (0–15). Default: `0`.
    pub const fn with_clock_div_ratio(mut self, value: u8) -> Self {
        self.clock_div_ratio = value;
        self
    }

    /// Sets pre-charge phase 1 (0–15). Default: `1`.
    pub const fn with_precharge_phase1(mut self, value: u8) -> Self {
        self.precharge_phase1 = value;
        self
    }

    /// Sets pre-charge phase 2 (0–15). Default: `15`.
    pub const fn with_precharge_phase2(mut self, value: u8) -> Self {
        self.precharge_phase2 = value;
        self
    }

    /// Sets the Vcomh deselect level. Default: [`VcomhLevel::Auto`].
    pub const fn with_vcomh_level(mut self, value: VcomhLevel) -> Self {
        self.vcomh_level = value;
        self
    }

    /// Sets the COM pin configuration. Default: [`ComPinConfig::Alternative`].
    pub const fn with_com_pin_config(mut self, value: ComPinConfig) -> Self {
        self.com_pin_config = value;
        self
    }
}

impl Default for ScreenConfig {
    fn default() -> Self {
        Self::new()
    }
}
