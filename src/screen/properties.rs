//! Properties of the display, such as dimensions and rotation.
//!
//! This struct is typically used internally by the `Sh1106` driver, but can be interacted with
//! via methods like `set_rotation` on the driver or canvas.
//!
//! ```rust
//! use mini_oled::screen::properties::{DisplayProperties, DisplayRotation};
//!
//! // DisplayProperties is used internally but can be default initialized.
//! // The generic parameters are Width, Height, and Offset.
//! let properties: DisplayProperties<128, 64, 2> = DisplayProperties::default();
//!
//! // DisplayRotation is used to configure the screen orientation.
//! let rotation = DisplayRotation::Rotate90;
//! ```

/// Properties of the display, such as dimensions and rotation.
///
/// This struct is typically used internally by the `Sh1106` driver, but can be interacted with
/// via methods like `set_rotation` on the driver or canvas.
///
/// ```rust
/// use mini_oled::screen::properties::{DisplayProperties, DisplayRotation};
///
/// // DisplayProperties is used internally but can be default initialized.
/// // The generic parameters are Width, Height, and Offset.
/// let properties: DisplayProperties<128, 64, 2> = DisplayProperties::default();
///
/// // DisplayRotation is used to configure the screen orientation.
/// let rotation = DisplayRotation::Rotate90;
/// ```
pub struct DisplayProperties<const W: u32, const H: u32, const O: u8> {
    display_rotation: DisplayRotation,
}

impl<const W: u32, const H: u32, const O: u8> DisplayProperties<W, H, O> {
    pub(crate) fn new(display_rotation: DisplayRotation) -> Self {
        DisplayProperties { display_rotation }
    }

    pub(crate) fn set_rotation(&mut self, display_rotation: DisplayRotation) {
        self.display_rotation = display_rotation;
    }

    pub(crate) fn get_column_offset(&self) -> u8 {
        O
    }

    pub(crate) fn get_rotation(&self) -> &DisplayRotation {
        &self.display_rotation
    }

    pub(crate) const fn get_display_size(&self) -> (u32, u32) {
        (W, H)
    }
}

impl<const W: u32, const H: u32, const O: u8> Default for DisplayProperties<W, H, O> {
    fn default() -> Self {
        Self {
            display_rotation: DisplayRotation::Rotate0,
        }
    }
}

/// Display rotation configuration.
///
/// # Example
///
/// ```rust
/// use mini_oled::screen::properties::DisplayRotation;
///
/// let rotation = DisplayRotation::Rotate90;
/// ```
#[derive(Clone, Copy)]
pub enum DisplayRotation {
    /// No rotation, normal display
    Rotate0,
    /// Rotate by 90 degress clockwise
    Rotate90,
    /// Rotate by 180 degress clockwise
    Rotate180,
    /// Rotate 270 degress clockwise
    Rotate270,
}
