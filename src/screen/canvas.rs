//! # Canvas
//!
//! The `Canvas` module provides the drawing surface for the display.
//! It handles the pixel buffer, dirty-area tracking for efficient partial
//! updates, and integration with `embedded-graphics` when the corresponding
//! feature is enabled.
//!
//! ## Example
//!
//! ```rust,ignore
//! use mini_oled::screen::canvas::Canvas;
//! // Canvas is normally obtained from the display driver, not created directly.
//! // let canvas = display.canvas_mut();
//!
//! // set_pixel uses logical coordinates that are automatically rotated.
//! // canvas.set_pixel(10, 20, true);
//! ```

use crate::screen::fast_mul;

use crate::error::MiniOledError;

use crate::screen::config::DisplayRotation;

/// A drawing canvas that manages the pixel buffer and dirty-area tracking.
///
/// The canvas stores pixel data in page-addressed format (8 vertical pixels
/// per byte) suitable for the SH1106 controller. It also tracks a "dirty
/// rectangle" so that the driver can send only the changed region during
/// [`Sh1106::flush`](crate::screen::sh1106::Sh1106::flush).
///
/// # Coordinate system
///
/// All public drawing methods (e.g. [`set_pixel`](Self::set_pixel)) accept
/// **logical** coordinates. When a rotation other than
/// [`DisplayRotation::Rotate0`](crate::screen::config::DisplayRotation::Rotate0)
/// is active, the canvas maps the logical `(x, y)` to the correct physical
/// memory location automatically.
///
/// # Example
///
/// ```rust,ignore
/// // Assuming you have a canvas instance from the Sh1106 driver
/// // let mut canvas = screen.canvas_mut();
///
/// // Set a pixel in logical coordinates
/// canvas.set_pixel(10, 20, true);
///
/// // Access the raw byte buffer
/// let buffer = canvas.get_buffer();
/// ```
pub struct Canvas<const N: usize, const W: u32, const H: u32, const O: u8> {
    buffer: [u8; N],
    dirty_area_min: (u32, u32),
    dirty_area_max: (u32, u32),
    display_rotation: DisplayRotation,
}

impl<const N: usize, const W: u32, const H: u32, const O: u8> Canvas<N, W, H, O> {
    pub(crate) fn new(display_rotation: DisplayRotation) -> Self {
        Canvas {
            buffer: [0; N],
            dirty_area_max: (0, 0),
            dirty_area_min: (W, H),
            display_rotation,
        }
    }

    /// Returns the column offset added to page addresses when flushing.
    ///
    /// This compensates for controller memory mapping differences on some
    /// display modules.
    pub(crate) fn get_column_offset(&self) -> u8 {
        O
    }

    /// Returns the display dimensions in logical pixels.
    pub(crate) const fn get_display_size(&self) -> (u32, u32) {
        (W, H)
    }

    pub(crate) fn set_rotation(&mut self, display_rotation: DisplayRotation) {
        self.display_rotation = display_rotation;
    }

    /// Returns an immutable reference to the raw pixel buffer.
    ///
    /// The buffer is organised in pages: each byte represents 8 vertical
    /// pixels. If you modify the buffer through [`get_mut_buffer`](Self::get_mut_buffer),
    /// the dirty area is **not** updated automatically. You must call
    /// [`Sh1106::flush_all`](crate::screen::sh1106::Sh1106::flush_all) afterwards
    /// to ensure the display reflects your changes.
    pub fn get_buffer(&self) -> &[u8; N] {
        &self.buffer
    }

    /// Returns a mutable reference to the raw pixel buffer.
    ///
    /// **Warning:** Direct buffer mutations bypass dirty-area tracking. After
    /// modifying the buffer you should either mark the affected region dirty
    /// manually or call [`Sh1106::flush_all`](crate::screen::sh1106::Sh1106::flush_all)
    /// to refresh the whole screen.
    pub fn get_mut_buffer(&mut self) -> &mut [u8; N] {
        &mut self.buffer
    }

    pub(crate) fn get_dirty_area(&self) -> ((u32, u32), (u32, u32)) {
        (self.dirty_area_min, self.dirty_area_max)
    }

    /// Marks the entire canvas as dirty so the next flush sends everything.
    pub(crate) fn force_full_dirty_area(&mut self) {
        self.dirty_area_min = (0, 0);
        self.dirty_area_max = (W - 1, H - 1);
    }

    /// Resets the dirty area to an empty rectangle.
    pub(crate) fn reset_dirty_area(&mut self) {
        self.dirty_area_min = (W, H);
        self.dirty_area_max = (0, 0);
    }

    /// Sets the state of a single pixel using **logical** coordinates.
    ///
    /// The coordinates are automatically translated according to the current
    /// [`DisplayRotation`](crate::screen::config::DisplayRotation). Pixels that
    /// fall outside the logical bounds are silently ignored.
    ///
    /// This method also updates the internal dirty area so that
    /// [`Sh1106::flush`](crate::screen::sh1106::Sh1106::flush) can later send
    /// only the changed region.
    ///
    /// # Arguments
    ///
    /// * `x` - Logical X coordinate.
    /// * `y` - Logical Y coordinate.
    /// * `pixel_status` - `true` to turn the pixel on, `false` to turn it off.
    #[inline]
    pub fn set_pixel(&mut self, x: u32, y: u32, pixel_status: bool) {
        let (physical_width, physical_height) = (W, H);
        let display_rotation = self.display_rotation;

        let (calculated_width_for_rotation, calculated_height_for_rotation) = match display_rotation
        {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => {
                (physical_width, physical_height)
            }
            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => {
                (physical_height, physical_width)
            }
        };

        if x >= calculated_width_for_rotation || y >= calculated_height_for_rotation {
            return;
        }

        if x < self.dirty_area_min.0 {
            self.dirty_area_min.0 = x;
        }
        if y < self.dirty_area_min.1 {
            self.dirty_area_min.1 = y;
        }
        if x > self.dirty_area_max.0 {
            self.dirty_area_max.0 = x;
        }
        if y > self.dirty_area_max.1 {
            self.dirty_area_max.1 = y;
        }

        let (idx, bit_mask) = match display_rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => {
                let idx = fast_mul!((y >> 3), W) + x; // y >> 3 is equal to y / 8
                let bit = 1 << (y & 7); // y & 7 is equal to y % 8
                (idx as usize, bit)
            }
            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => {
                let idx = fast_mul!((x >> 3), W) + y; // y >> 3 is equal to y / 8
                let bit = 1 << (x & 7); // y & 7 is equal to y % 8
                (idx as usize, bit)
            }
        };
        /*
           match pixel_status {
               true => self.buffer[idx as usize] |= bit_mask,
               false => self.buffer[idx as usize] &= !bit_mask,
           }
           It's same to above code, it's better for branching but not reading
        */
        if idx < N {
            let pixel_status_mask = (-(pixel_status as i8)) as u8;
            self.buffer[idx] = (self.buffer[idx] & !bit_mask) | (pixel_status_mask & bit_mask);
        }
    }
}

#[cfg(feature = "embedded-graphics-core")]
use embedded_graphics_core::{
    Pixel,
    pixelcolor::BinaryColor,
    prelude::{Dimensions, DrawTarget, OriginDimensions, Size},
};

#[cfg(feature = "embedded-graphics-core")]
impl<const N: usize, const W: u32, const H: u32, const O: u8> DrawTarget for Canvas<N, W, H, O> {
    type Color = BinaryColor;

    type Error = MiniOledError;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics_core::Pixel<Self::Color>>,
    {
        let bb = self.bounding_box();

        pixels
            .into_iter()
            .filter(|Pixel(pos, _color)| bb.contains(*pos))
            .for_each(|Pixel(pos, color)| {
                self.set_pixel(pos.x as u32, pos.y as u32, color.is_on())
            });

        Ok(())
    }
}

#[cfg(feature = "embedded-graphics-core")]
impl<const N: usize, const W: u32, const H: u32, const O: u8> OriginDimensions
    for Canvas<N, W, H, O>
{
    fn size(&self) -> Size {
        let (width, height) = (W, H);

        Size::new(width, height)
    }
}
