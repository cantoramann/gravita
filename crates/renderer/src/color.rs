//! RGBA color type, constructors, and a palette of common opaque colors.

/// RGBA color, one byte per channel.
pub type Color = [u8; 4];

/// Build an opaque RGB color (alpha = 255).
#[inline]
#[must_use]
pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
    [r, g, b, 0xff]
}

/// Build an RGBA color.
#[inline]
#[must_use]
pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    [r, g, b, a]
}

/// Common palette constants. Use these instead of inlining hex literals.
pub mod palette {
    use super::Color;

    /// Opaque black.
    pub const BLACK: Color = [0x00, 0x00, 0x00, 0xff];
    /// Opaque white.
    pub const WHITE: Color = [0xff, 0xff, 0xff, 0xff];
    /// Pure red.
    pub const RED: Color = [0xff, 0x00, 0x00, 0xff];
    /// Pure green.
    pub const GREEN: Color = [0x00, 0xff, 0x00, 0xff];
    /// Pure blue.
    pub const BLUE: Color = [0x00, 0x00, 0xff, 0xff];
    /// Yellow.
    pub const YELLOW: Color = [0xff, 0xff, 0x00, 0xff];
    /// Cyan.
    pub const CYAN: Color = [0x00, 0xff, 0xff, 0xff];
    /// Magenta.
    pub const MAGENTA: Color = [0xff, 0x00, 0xff, 0xff];
    /// Standard mid-grey.
    pub const GRAY: Color = [0x80, 0x80, 0x80, 0xff];
    /// Common background dark blue used by the bouncing-balls example.
    pub const DARK_BLUE_BG: Color = [0x20, 0x20, 0x40, 0xff];
    /// Fully transparent.
    pub const TRANSPARENT: Color = [0x00, 0x00, 0x00, 0x00];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb_sets_alpha_to_opaque() {
        assert_eq!(rgb(0x12, 0x34, 0x56), [0x12, 0x34, 0x56, 0xff]);
    }

    #[test]
    fn rgba_passes_through_all_channels() {
        assert_eq!(rgba(0x12, 0x34, 0x56, 0x78), [0x12, 0x34, 0x56, 0x78]);
    }

    #[test]
    fn palette_white_is_full_intensity() {
        assert_eq!(palette::WHITE, [0xff, 0xff, 0xff, 0xff]);
    }

    #[test]
    fn palette_transparent_has_zero_alpha() {
        assert_eq!(palette::TRANSPARENT[3], 0x00);
    }

    #[test]
    fn palette_constants_have_opaque_alpha() {
        for c in [
            palette::BLACK,
            palette::WHITE,
            palette::RED,
            palette::GREEN,
            palette::BLUE,
            palette::YELLOW,
            palette::CYAN,
            palette::MAGENTA,
            palette::GRAY,
            palette::DARK_BLUE_BG,
        ] {
            assert_eq!(c[3], 0xff, "palette color {c:?} should be opaque");
        }
    }
}
