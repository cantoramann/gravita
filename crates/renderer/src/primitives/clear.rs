//! Clear the entire frame to a solid color.

use crate::color::Color;

/// Clear the entire frame to a solid color (RGBA).
pub fn clear(frame: &mut [u8], color: Color) {
    for px in frame.chunks_exact_mut(4) {
        px.copy_from_slice(&color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::pixel_index;

    const WIDTH: u32 = 100;
    const HEIGHT: u32 = 100;

    fn make_frame() -> Vec<u8> {
        vec![0u8; (WIDTH * HEIGHT * 4) as usize]
    }

    fn pixel_at(frame: &[u8], x: u32, y: u32) -> [u8; 4] {
        let idx = pixel_index(x, y, WIDTH);
        [frame[idx], frame[idx + 1], frame[idx + 2], frame[idx + 3]]
    }

    #[test]
    fn clear_fills_entire_frame() {
        let mut f = make_frame();
        let color = [0x12, 0x34, 0x56, 0x78];
        clear(&mut f, color);
        assert_eq!(pixel_at(&f, 0, 0), color);
        assert_eq!(pixel_at(&f, WIDTH - 1, HEIGHT - 1), color);
        assert_eq!(pixel_at(&f, 50, 50), color);
    }

    #[test]
    fn clear_overwrites_previous_color() {
        let mut f = make_frame();
        clear(&mut f, [0xff, 0xff, 0xff, 0xff]);
        clear(&mut f, [0x00, 0x00, 0x00, 0xff]);
        assert_eq!(pixel_at(&f, 50, 50), [0x00, 0x00, 0x00, 0xff]);
    }
}
