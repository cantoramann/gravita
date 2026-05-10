//! 5x7 bitmap font rendering.
//!
//! Each glyph is 5 columns wide × 7 rows tall, packed into seven `u8` rows
//! (top bit unused). The font is intentionally minimal — just enough for
//! FPS counters, scores, and short HUD strings. It originated in the tetris
//! example, where it lived inline; lifting it into the renderer means every
//! example (and downstream user) gets a working text overlay without rolling
//! their own.

use crate::{color::Color, frame::put_pixel};

/// Width of a single glyph in pixels (5 columns + 3-pixel kerning advance).
pub const CHAR_ADVANCE: i32 = 8;

/// Visible glyph width.
pub const CHAR_WIDTH: i32 = 5;

/// Glyph height.
pub const CHAR_HEIGHT: i32 = 7;

/// Look up the 7-row bitmap for `ch`. Letters are normalized to uppercase.
///
/// Unknown characters render as a blank glyph.
#[must_use]
pub fn char_bitmap(ch: char) -> [u8; 7] {
    match ch.to_ascii_uppercase() {
        'A' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'B' => [0b11110, 0b10001, 0b11110, 0b10001, 0b10001, 0b10001, 0b11110],
        'C' => [0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110],
        'D' => [0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
        'E' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
        'F' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
        'G' => [0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110],
        'H' => [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'I' => [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        'J' => [0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100],
        'K' => [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
        'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'M' => [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001],
        'N' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
        'O' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'P' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
        'Q' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
        'R' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
        'S' => [0b01110, 0b10001, 0b10000, 0b01110, 0b00001, 0b10001, 0b01110],
        'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        'U' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'V' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
        'W' => [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001],
        'X' => [0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001],
        'Y' => [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
        'Z' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
        '0' => [0b01110, 0b10011, 0b10101, 0b10101, 0b11001, 0b10001, 0b01110],
        '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        '2' => [0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111],
        '3' => [0b01110, 0b10001, 0b00001, 0b00110, 0b00001, 0b10001, 0b01110],
        '4' => [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
        '5' => [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
        '6' => [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
        '7' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
        '8' => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
        '9' => [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100],
        ':' => [0b00000, 0b00100, 0b00000, 0b00000, 0b00100, 0b00000, 0b00000],
        '!' => [0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100],
        _ => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000],
    }
}

/// Draw a single character at `(x, y)` (top-left corner, screen space).
pub fn draw_char(
    frame: &mut [u8],
    ch: char,
    x: i32,
    y: i32,
    color: Color,
    width: u32,
    height: u32,
) {
    let bitmap = char_bitmap(ch);
    for (row, &bits) in bitmap.iter().enumerate() {
        for col in 0..CHAR_WIDTH {
            if (bits >> (CHAR_WIDTH - 1 - col)) & 1 == 1 {
                put_pixel(frame, x + col, y + row as i32, width, height, color);
            }
        }
    }
}

/// Draw a single character scaled by an integer factor.
#[allow(clippy::too_many_arguments)] // text rendering needs all of these; refactor when adding TextStyle.
pub fn draw_char_scaled(
    frame: &mut [u8],
    ch: char,
    x: i32,
    y: i32,
    color: Color,
    scale: i32,
    width: u32,
    height: u32,
) {
    if scale <= 1 {
        draw_char(frame, ch, x, y, color, width, height);
        return;
    }
    let bitmap = char_bitmap(ch);
    for (row, &bits) in bitmap.iter().enumerate() {
        for col in 0..CHAR_WIDTH {
            if (bits >> (CHAR_WIDTH - 1 - col)) & 1 == 1 {
                for sy in 0..scale {
                    for sx in 0..scale {
                        put_pixel(
                            frame,
                            x + col * scale + sx,
                            y + row as i32 * scale + sy,
                            width,
                            height,
                            color,
                        );
                    }
                }
            }
        }
    }
}

/// Draw a left-aligned string.
pub fn draw_text(
    frame: &mut [u8],
    text: &str,
    x: i32,
    y: i32,
    color: Color,
    width: u32,
    height: u32,
) {
    let mut cx = x;
    for ch in text.chars() {
        draw_char(frame, ch, cx, y, color, width, height);
        cx += CHAR_ADVANCE;
    }
}

/// Draw a left-aligned string scaled by an integer factor.
#[allow(clippy::too_many_arguments)] // text rendering needs all of these; refactor when adding TextStyle.
pub fn draw_text_scaled(
    frame: &mut [u8],
    text: &str,
    x: i32,
    y: i32,
    color: Color,
    scale: i32,
    width: u32,
    height: u32,
) {
    let advance = CHAR_ADVANCE * scale.max(1);
    let mut cx = x;
    for ch in text.chars() {
        draw_char_scaled(frame, ch, cx, y, color, scale, width, height);
        cx += advance;
    }
}

/// Draw `text` horizontally centered on column `cx`.
pub fn draw_text_centered(
    frame: &mut [u8],
    text: &str,
    cx: i32,
    y: i32,
    color: Color,
    width: u32,
    height: u32,
) {
    let pixel_width = text.chars().count() as i32 * CHAR_ADVANCE;
    draw_text(frame, text, cx - pixel_width / 2, y, color, width, height);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{color::palette, frame::pixel_index};

    const W: u32 = 60;
    const H: u32 = 20;

    fn make_frame() -> Vec<u8> {
        vec![0u8; (W * H * 4) as usize]
    }

    fn pixel_at(frame: &[u8], x: u32, y: u32) -> [u8; 4] {
        let idx = pixel_index(x, y, W);
        [frame[idx], frame[idx + 1], frame[idx + 2], frame[idx + 3]]
    }

    fn count(frame: &[u8], color: Color) -> usize {
        frame.chunks_exact(4).filter(|px| px == &color).count()
    }

    #[test]
    fn char_bitmap_normalizes_case() {
        assert_eq!(char_bitmap('a'), char_bitmap('A'));
    }

    #[test]
    fn unknown_char_renders_blank() {
        assert_eq!(char_bitmap('~'), [0; 7]);
    }

    #[test]
    fn space_is_blank() {
        assert_eq!(char_bitmap(' '), [0; 7]);
    }

    #[test]
    fn draw_char_paints_inside_glyph() {
        let mut f = make_frame();
        draw_char(&mut f, 'I', 0, 0, palette::WHITE, W, H);
        // 'I' bitmap: top row is 0b01110 → cols 1,2,3 painted at row 0
        assert_eq!(pixel_at(&f, 1, 0), palette::WHITE);
        assert_eq!(pixel_at(&f, 2, 0), palette::WHITE);
        assert_eq!(pixel_at(&f, 3, 0), palette::WHITE);
        assert_eq!(pixel_at(&f, 0, 0), [0, 0, 0, 0]);
        assert_eq!(pixel_at(&f, 4, 0), [0, 0, 0, 0]);
    }

    #[test]
    fn draw_char_scaled_doubles_paint_size() {
        let mut f = make_frame();
        draw_char_scaled(&mut f, 'I', 0, 0, palette::WHITE, 2, W, H);
        // 'I' top row 0b01110 spans cols 1-3 originally; at scale 2 that becomes cols 2-7
        // and rows 0-1 (each original row is 2 px tall).
        let painted_before = count(&f, palette::WHITE);
        assert!(painted_before >= 4, "scaled glyph should paint many pixels");
        // Compare with unscaled
        let mut f1 = make_frame();
        draw_char(&mut f1, 'I', 0, 0, palette::WHITE, W, H);
        let painted_unscaled = count(&f1, palette::WHITE);
        assert_eq!(painted_before, painted_unscaled * 4, "scale 2 should 4× the painted pixels");
    }

    #[test]
    fn draw_text_kerns_characters() {
        let mut f = make_frame();
        draw_text(&mut f, "II", 0, 0, palette::WHITE, W, H);
        // Each 'I' takes 5 pixels visible, plus 3 pixels advance.
        // First 'I' at cols 1-3 row 0; second 'I' at cols 9-11 row 0.
        assert_eq!(pixel_at(&f, 1, 0), palette::WHITE);
        assert_eq!(pixel_at(&f, 9, 0), palette::WHITE);
        assert_eq!(pixel_at(&f, 6, 0), [0, 0, 0, 0], "kerning gap unpainted");
    }

    #[test]
    fn draw_text_centered_aligns_around_cx() {
        let mut f = make_frame();
        draw_text_centered(&mut f, "II", 30, 0, palette::WHITE, W, H);
        // Two chars × 8 px advance = 16 px wide; cx=30 means start at 22.
        // First 'I' starts at col 22+1=23.
        assert_eq!(pixel_at(&f, 23, 0), palette::WHITE);
    }

    #[test]
    fn out_of_bounds_does_not_panic() {
        let mut f = make_frame();
        draw_text(&mut f, "HELLO", -100, -100, palette::WHITE, W, H);
        draw_text(&mut f, "HELLO", 1000, 1000, palette::WHITE, W, H);
    }
}
