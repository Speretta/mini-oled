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

/// Display rotation
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
