use crate::{
    command::{Command, CommandBuffer, Page},
    error::MiniOledError,
    fast_mul,
    interface::CommunicationInterface,
};

use super::{
    canvas::Canvas,
    properties::{DisplayProperties, DisplayRotation},
};

const WIDTH: u32 = 128;
const HEIGHT: u32 = 64;
const OFFSET: u8 = 2;
const BUFFER_SIZE: usize = WIDTH as usize * HEIGHT as usize / 8;

pub struct Sh1106<CI: CommunicationInterface> {
    communication_interface: CI,
    canvas: Canvas<BUFFER_SIZE, WIDTH, HEIGHT, OFFSET>,
}

impl<CI: CommunicationInterface> Sh1106<CI> {
    pub fn new(communication_interface: CI) -> Sh1106<CI> {
        let display_properties: DisplayProperties<WIDTH, HEIGHT, 2> =
            DisplayProperties::new(DisplayRotation::Rotate0);
        Sh1106 {
            communication_interface,
            canvas: Canvas::new(display_properties),
        }
    }

    pub fn get_canvas(&self) -> &Canvas<BUFFER_SIZE, WIDTH, HEIGHT, OFFSET> {
        &self.canvas
    }

    pub fn get_mut_canvas(&mut self) -> &mut Canvas<BUFFER_SIZE, WIDTH, HEIGHT, OFFSET> {
        &mut self.canvas
    }

    pub fn flush_all(&mut self) -> Result<(), MiniOledError> {
        self.canvas.force_full_dirty_area();
        self.flush()
    }

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

    pub fn get_rotation(&self) -> &DisplayRotation {
        self.canvas.get_rotation()
    }

    pub fn test_screen(&mut self) -> Result<(), MiniOledError> {
        let command_buffer = &(CommandBuffer::from([Command::EnableTestScreen]));

        self.communication_interface.write_command(command_buffer)
    }

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
