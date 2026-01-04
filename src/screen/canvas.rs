use crate::{error::MiniOledError, fast_mul};

use super::properties::{DisplayProperties, DisplayRotation};

pub struct Canvas<const N: usize, const W: u32, const H: u32, const O: u8> {
    buffer: [u8; N],
    dirty_area_min: (u32, u32),
    dirty_area_max: (u32, u32),
    display_properties: DisplayProperties<W, H, O>,
}

impl<const N: usize, const W: u32, const H: u32, const O: u8> Canvas<N, W, H, O> {
    pub(crate) fn new(display_properties: DisplayProperties<W, H, O>) -> Self {
        Canvas {
            buffer: [0; N],
            dirty_area_max: (0, 0),
            dirty_area_min: display_properties.get_display_size(),
            display_properties,
        }
    }

    pub(crate) fn get_column_offset(&self) -> u8{
        self.display_properties.get_column_offset()
    }

    pub(crate) const fn get_display_size(&self) -> (u32, u32) {
        self.display_properties.get_display_size()
    }

    pub(crate) fn get_rotation(&self) -> &DisplayRotation {
        self.display_properties.get_rotation()
    }
    
    pub(crate) fn set_rotation(&mut self, display_rotation: DisplayRotation) {
        self.display_properties.set_rotation(display_rotation);
    }

    pub(crate) fn get_buffer(&self) -> &[u8; N] {
        &self.buffer
    }

    pub(crate) fn get_dirty_area(&self) -> ((u32, u32), (u32, u32)) {
        (self.dirty_area_min, self.dirty_area_max)
    }

    pub(crate) fn force_full_dirty_area(&mut self){
        self.dirty_area_min = (0, 0);
        self.dirty_area_max = (W -1, H -1);
    }

    pub(crate) fn reset_dirty_area(&mut self) {
        self.dirty_area_min = self.display_properties.get_display_size();
        self.dirty_area_max = (0, 0);
    }

    #[inline]
    fn set_pixel(&mut self, x: u32, y: u32, pixel_status: bool) {
        let (physical_width, physical_height) = self.display_properties.get_display_size();
        let display_rotation = self.display_properties.get_rotation();

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

        let (idx, bit_mask) = match *display_rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => {
                let idx = fast_mul!((y>>3), W) + x; // y >> 3 is equal to y / 8
                let bit = 1 << (y & 7); // y & 7 is equal to y % 8
                (idx as usize, bit)
            }
            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => {
                let idx = fast_mul!((x>>3), W) + y; // y >> 3 is equal to y / 8
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
        if (idx as usize) < N {
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
impl<const N: usize, const W: u32, const H: u32, const O: u8> OriginDimensions for Canvas<N, W, H, O> {
    fn size(&self) -> Size {
        let (width, height) = self.display_properties.get_display_size();

        Size::new(width, height)
    }
}
