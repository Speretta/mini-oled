//! # SH1106 Driver
//!
//! This module contains the main `Sh1106` driver struct.
//! It brings together the communication interface and the canvas to control the display.
//!
//! ## Example
//!
//! ```rust,ignore
//! use mini_oled::{
//!     interface::i2c::I2cInterface,
//!     screen::sh1106::Sh1106,
//! };
//!
//! // let i2c = ...; // I2C peripheral
//! let interface = I2cInterface::new(i2c, 0x3C);
//! let mut display = Sh1106::new(interface);
//!
//! display.init().unwrap();
//! display.test_screen().unwrap();
//! ```

use crate::{
    command::{Command, CommandBuffer, Page},
    error::MiniOledError,
    interface::CommunicationInterface,
    screen::fast_mul,
};

use crate::screen::{
    canvas::Canvas,
    properties::{DisplayProperties, DisplayRotation},
};

const WIDTH: u32 = 128;
const HEIGHT: u32 = 64;
const OFFSET: u8 = 2;
const BUFFER_SIZE: usize = WIDTH as usize * HEIGHT as usize / 8;

/// The main driver struct for the SH1106 OLED display.
///
/// This struct manages the communication interface and the drawing canvas.
///
/// # Example
///
/// ```rust,ignore
/// use mini_oled::{
///     interface::i2c::I2cInterface,
///     screen::sh1106::Sh1106,
/// };
///
/// // let i2c_interface = ...;
/// let mut screen = Sh1106::new(i2c_interface);
/// screen.init().unwrap();
/// screen.test_screen().unwrap();
/// ```
pub struct Sh1106<CI: CommunicationInterface> {
    communication_interface: CI,
    canvas: Canvas<BUFFER_SIZE, WIDTH, HEIGHT, OFFSET>,
}

impl<CI: CommunicationInterface> Sh1106<CI> {
    /// Creates a new `Sh1106` driver instance.
    ///
    /// # Arguments
    ///
    /// * `communication_interface` - The initialized communication interface (I2C or ~~SPI~~).
    pub fn new(communication_interface: CI) -> Sh1106<CI> {
        let display_properties: DisplayProperties<WIDTH, HEIGHT, 2> =
            DisplayProperties::new(DisplayRotation::Rotate0);
        Sh1106 {
            communication_interface,
            canvas: Canvas::new(display_properties),
        }
    }

    /// Returns a reference to the underlying canvas.
    pub fn get_canvas(&self) -> &Canvas<BUFFER_SIZE, WIDTH, HEIGHT, OFFSET> {
        &self.canvas
    }

    /// Returns a mutable reference to the underlying canvas.
    pub fn get_mut_canvas(&mut self) -> &mut Canvas<BUFFER_SIZE, WIDTH, HEIGHT, OFFSET> {
        &mut self.canvas
    }

    /// Flushes the entire display buffer to the screen, refreshing all pixels.
    pub fn flush_all(&mut self) -> Result<(), MiniOledError> {
        self.canvas.force_full_dirty_area();
        self.flush()
    }

    /// Flushes only the modified parts of the display buffer to the screen.
    ///
    /// This is more efficient than `flush_all` as it only sends changed data.
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

    /// Returns the current rotation of the display.
    pub fn get_rotation(&self) -> &DisplayRotation {
        self.canvas.get_rotation()
    }

    /// Enables the test screen mode (all pixels on).
    pub fn test_screen(&mut self) -> Result<(), MiniOledError> {
        let command_buffer = &(CommandBuffer::from([Command::EnableTestScreen]));

        self.communication_interface.write_command(command_buffer)
    }

    /// Sets the rotation of the display.
    ///
    /// # Arguments
    ///
    /// * `display_rotation` - The new rotation setting.
    pub fn set_rotation(&mut self, display_rotation: DisplayRotation) -> Result<(), MiniOledError> {
        self.canvas.set_rotation(display_rotation);

        let rotation_sequence: CommandBuffer<2> = match display_rotation {
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
        .into();

        self.communication_interface
            .write_command(&rotation_sequence)
    }

    /// Initializes the display with default settings.
    ///
    /// This sends a sequence of commands to set up the display driver.
    pub fn init(&mut self) -> Result<(), MiniOledError> {
        let init_sequence: CommandBuffer<15> = [
            Command::TurnDisplayOff,
            Command::DisplayClockDiv(0x8, 0x0),
            Command::Multiplex(self.canvas.get_display_size().1 as u8 - 1),
            Command::DisplayOffset(0),
            Command::StartLine(0),
            Command::EnableChargePump,
            Command::EnableSegmentRemap,
            Command::EnableReverseComDir,
            Command::AlternativeComPinConfig,
            Command::Contrast(0x80),
            Command::PreChargePeriod(0x1, 0xF),
            Command::VcomhDeselect(crate::command::VcomhLevel::Auto),
            Command::DisableTestScreen,
            Command::PositiveImageMode,
            Command::TurnDisplayOn,
        ]
        .into();

        self.communication_interface.write_command(&init_sequence)
    }
}
