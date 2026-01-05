//! # Commands
//!
//! This module defines the commands that can be sent to the SH1106 display controller.
//! It includes the `Command` enum and the `CommandBuffer` struct for batching commands.
//!
//! ## Example
//!
//! Sending a manual command sequence (pseudo-code as `CommunicationInterface` is needed).
//!
//! ```rust
//! use mini_oled::command::{Command, CommandBuffer};
//!
//! let commands: CommandBuffer<2> = [
//!     Command::TurnDisplayOn,
//!     Command::Contrast(0xFF),
//! ].into();
//!
//! // Write commands using the interface...
//! // interface.write_command(&commands).unwrap();
//! ```

use crate::error::MiniOledError;

/// A buffer for storing commands to be sent to the display.
///
/// This struct holds an array of `Command`s.
#[derive(Debug, Clone, Copy)]
pub struct CommandBuffer<const N: usize> {
    buffer: [Command; N],
}

impl From<Command> for CommandBuffer<1> {
    fn from(value: Command) -> Self {
        CommandBuffer { buffer: [value] }
    }
}

impl<const N: usize> From<[Command; N]> for CommandBuffer<N> {
    fn from(value: [Command; N]) -> Self {
        CommandBuffer { buffer: value }
    }
}

impl<const N: usize> CommandBuffer<N> {
    /// Serializes the command buffer into a byte slice.
    ///
    /// # Arguments
    ///
    /// * `buffer` - A mutable byte slice to write the serialized commands into.
    ///
    /// # Returns
    ///
    /// A slice containing the written bytes on success, or `MiniOledError` if the buffer is too small.
    pub fn to_bytes<'a>(&self, buffer: &'a mut [u8]) -> Result<&'a [u8], MiniOledError> {
        let mut output_length = 1usize;
        for command in &self.buffer {
            let (command_bytes, bytes_length) = command.to_bytes();
            if output_length + bytes_length > buffer.len() {
                return Err(MiniOledError::CommandBufferSizeError);
            }
            buffer[output_length..output_length + bytes_length]
                .copy_from_slice(&command_bytes[0..bytes_length]);
            output_length += bytes_length;
        }
        Ok(&buffer[..output_length])
    }
}

/// Enum representing commands that can be sent to the SH1106 controller.
#[derive(Debug, Clone, Copy)]
pub enum Command {
    /// Set contrast. Higher number is higher contrast.
    /// Default is `0x7F`.
    Contrast(u8),
    /// Forces the entire display to be on regardless of the contents of the display RAM.
    /// It does not overwrite the RAM. Often used for testing pixels or creating a flash effect.
    /// Sending `DisableTestSceen` resumes displaying the RAM content.
    EnableTestScreen,
    /// Disables test screen mode.
    DisableTestScreen,
    /// Inverts the display data.
    /// Normally, a 1 in memory means a lit pixel. (`PositiveImageMode`)
    /// When inverted, 0 means lit and 1 means off. (`NegativeImageMode`)
    /// Default is `PositiveImageMode`.
    PositiveImageMode,
    /// Enable negative image mode.
    NegativeImageMode,
    /// Turns the display on.
    TurnDisplayOn,
    /// Puts the display into sleep mode.
    /// In sleep mode (0xAE), the internal circuit is active but the driving circuit is off,
    /// reducing power consumption drastically (< 20µA). RAM content is preserved.
    TurnDisplayOff,
    /// Set column address lower 4 bits.
    ColumnAddressLow(u8),
    /// Set column address higher 4 bits.
    ColumnAddressHigh(u8),
    /// Set page address.
    PageAddress(Page),
    /// Set display start line from 0-63.
    StartLine(u8),
    /// Reverse columns from 127-0, mirrors the display horizontally (X-axis).
    /// Default is `DisableSegmentRemap`.
    EnableSegmentRemap,
    /// Disable segment remap (normal column order).
    DisableSegmentRemap,
    /// Set multipex ratio from 15-63 (MUX-1).
    Multiplex(u8),
    /// Scan from COM[n-1] to COM0 (where N is mux ratio).
    /// Used together with `EnableSegmentRemap` to rotate the display 180 degrees.
    /// Default is `DisableReverseComDir`.
    EnableReverseComDir,
    /// Disable reverse COM direction (normal scan).
    DisableReverseComDir,
    /// Set vertical display offset.
    DisplayOffset(u8),
    /// Setup COM hardware configuration.
    /// Value indicates sequential (`SequentialComPinConfig`) or alternative (`AlternativeComPinConfig`)
    /// pin configuration.
    /// Default is `AlternativeComPinConfig`.
    AlternativeComPinConfig,
    /// Sequential COM pin configuration.
    SequentialComPinConfig,
    /// Set up display clock.
    /// First value is oscillator frequency, increasing with higher value.
    /// Second value is divide ratio - 1.
    DisplayClockDiv(u8, u8),
    /// Set up phase 1 and 2 of precharge period. Each value is from 0-63.
    PreChargePeriod(u8, u8),
    /// Set Vcomh Deselect level.
    VcomhDeselect(VcomhLevel),
    /// No Operation.
    Noop,
    /// Enable charge pump.
    /// Display must be off when performing this command.
    /// Default is `EnableChargePump`.
    EnableChargePump,
    /// Disable charge pump.
    DisableChargePump,
}

impl Command {
    pub fn to_bytes(&self) -> ([u8; 2], usize) {
        match self {
            Command::Contrast(val) => ([0x81, *val], self.get_byte_size()),
            Command::EnableTestScreen => ([0xA5, 0], self.get_byte_size()),
            Command::DisableTestScreen => ([0xA4, 0], self.get_byte_size()),
            Command::PositiveImageMode => ([0xA6, 0], self.get_byte_size()),
            Command::NegativeImageMode => ([0xA7, 0], self.get_byte_size()),
            Command::TurnDisplayOn => ([0xAF, 0], self.get_byte_size()),
            Command::TurnDisplayOff => ([0xAE, 0], self.get_byte_size()),
            Command::ColumnAddressLow(addr) => ([0xF & addr, 0], self.get_byte_size()),
            Command::ColumnAddressHigh(addr) => ([0x10 | (0xF & addr), 0], self.get_byte_size()),
            Command::PageAddress(page) => ([0xB0 | (*page as u8), 0], self.get_byte_size()),
            Command::StartLine(line) => ([0x40 | (0x3F & line), 0], self.get_byte_size()),
            Command::EnableSegmentRemap => ([0xA1, 0], self.get_byte_size()),
            Command::DisableSegmentRemap => ([0xA0, 0], self.get_byte_size()),
            Command::Multiplex(ratio) => ([0xA8, *ratio], self.get_byte_size()),
            Command::EnableReverseComDir => ([0xC8, 0], self.get_byte_size()),
            Command::DisableReverseComDir => ([0xC0, 0], self.get_byte_size()),
            Command::DisplayOffset(offset) => ([0xD3, *offset], self.get_byte_size()),
            Command::AlternativeComPinConfig => ([0xDA, 0x12], self.get_byte_size()),
            Command::SequentialComPinConfig => ([0xDA, 0x02], self.get_byte_size()),
            Command::DisplayClockDiv(fosc, div) => (
                [0xD5, ((0xF & fosc) << 4) | (0xF & div)],
                self.get_byte_size(),
            ),
            Command::PreChargePeriod(phase1, phase2) => (
                [0xD9, ((0xF & phase2) << 4) | (0xF & phase1)],
                self.get_byte_size(),
            ),
            Command::VcomhDeselect(level) => ([0xDB, (*level as u8) << 4], self.get_byte_size()),
            Command::Noop => ([0xE3, 0], self.get_byte_size()),
            Command::EnableChargePump => ([0xAD, 0x8B], self.get_byte_size()),
            Command::DisableChargePump => ([0xAD, 0x8A], self.get_byte_size()),
        }
    }

    /// Returns the size in bytes of the command when serialized.
    pub const fn get_byte_size(&self) -> usize {
        match self {
            Command::Contrast(_) => 2,
            Command::EnableTestScreen => 1,
            Command::DisableTestScreen => 1,
            Command::PositiveImageMode => 1,
            Command::NegativeImageMode => 1,
            Command::TurnDisplayOn => 1,
            Command::TurnDisplayOff => 1,
            Command::ColumnAddressLow(_) => 1,
            Command::ColumnAddressHigh(_) => 1,
            Command::PageAddress(_) => 1,
            Command::StartLine(_) => 1,
            Command::EnableSegmentRemap => 1,
            Command::DisableSegmentRemap => 1,
            Command::Multiplex(_) => 2,
            Command::EnableReverseComDir => 1,
            Command::DisableReverseComDir => 1,
            Command::DisplayOffset(_) => 2,
            Command::AlternativeComPinConfig => 2,
            Command::SequentialComPinConfig => 2,
            Command::DisplayClockDiv(_, _) => 2,
            Command::PreChargePeriod(_, _) => 2,
            Command::VcomhDeselect(_) => 2,
            Command::Noop => 1,
            Command::EnableChargePump => 2,
            Command::DisableChargePump => 2,
        }
    }
}

/// Display page address (0-7).
///
/// The display memory is divided into 8 pages, each 8 pixels high.
///
/// # Example
///
/// ```rust
/// use mini_oled::command::Page;
///
/// let page = Page::Page0;
/// assert_eq!(page as u8, 0);
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Page {
    /// Page 0
    Page0 = 0,
    /// Page 1
    Page1 = 1,
    /// Page 2
    Page2 = 2,
    /// Page 3
    Page3 = 3,
    /// Page 4
    Page4 = 4,
    /// Page 5
    Page5 = 5,
    /// Page 6
    Page6 = 6,
    /// Page 7
    Page7 = 7,
}

impl Page {
    /// Returns an iterator over a range of pages.
    pub fn range(start: Page, end: Page) -> impl Iterator<Item = Page> {
        (start as u8..=end as u8).map(Page::from)
    }

    /// Returns an iterator over all 8 pages (0-7).
    pub fn all() -> impl Iterator<Item = Page> {
        (0..8).map(Page::from)
    }
}

impl From<u8> for Page {
    fn from(val: u8) -> Page {
        // Faster way the casting u8 to Page
        // ```rust
        // 0x00 => Page::Page0,
        // 0x08 => Page::Page1,
        // 0x09 => Page::Page2,
        // 0x0A => Page::Page3,
        // ```
        let new_val = val & 0b111;
        unsafe { core::mem::transmute(new_val) }
    }
}

/// Frame interval configuration for the display clock.
///
/// This determines how often the display refreshes.
///
/// # Example
///
/// ```rust
/// use mini_oled::command::NFrames;
///
/// let frames = NFrames::F5;
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy)]

pub enum NFrames {
    /// 2 Frames
    F2 = 0b111,
    /// 3 Frames
    F3 = 0b100,
    /// 4 Frames
    F4 = 0b101,
    /// 5 Frames
    F5 = 0b000,
    /// 25 Frames
    F25 = 0b110,
    /// 64 Frames
    F64 = 0b001,
    /// 128 Frames
    F128 = 0b010,
    /// 256 Frames
    F256 = 0b011,
}

/// Vcomh Deselect level.
///
/// This adjusts the Vcomh regulator output.
///
/// # Example
///
/// ```rust
/// use mini_oled::command::VcomhLevel;
///
/// let level = VcomhLevel::V077;
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy)]

pub enum VcomhLevel {
    /// 0.65 * Vcc
    V065 = 0b001,
    /// 0.77 * Vcc
    V077 = 0b010,
    /// 0.83 * Vcc
    V083 = 0b011,
    /// Auto
    Auto = 0b100,
}
