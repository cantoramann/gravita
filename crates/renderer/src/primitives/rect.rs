//! Filled and outlined rectangle primitives.
//!
//! Rectangles take integer pixel coordinates because they are used by
//! examples that already round once (game tiles, panels, HUD frames).

use crate::{
    color::Color,
    frame::{blend_pixel, pixel_index, put_pixel},
};

/// Integer pixel rectangle: `[x, x + w) × [y, y + h)`.
#[derive(Debug, Copy, Clone)]
pub struct PixelRect {
    /// Left edge (pixel column).
    pub x: i32,
    /// Top edge (pixel row).
    pub y: i32,
    /// Width in pixels.
    pub w: i32,
    /// Height in pixels.
    pub h: i32,
}

impl PixelRect {
    /// Build a new pixel rectangle from corner + extents.
    #[inline]
    #[must_use]
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }
}

/// Draw a filled axis-aligned rectangle.
///
/// The rectangle covers `[x, x + w) × [y, y + h)` in screen-space pixels and
/// is clipped to `[0, width) × [0, height)`.
pub fn draw_rect_filled(
    frame: &mut [u8],
    rect: PixelRect,
    color: Color,
    width: u32,
    height: u32,
) {
    if rect.w <= 0 || rect.h <= 0 {
        return;
    }
    let x0 = rect.x.max(0);
    let y0 = rect.y.max(0);
    let x1 = (rect.x + rect.w).min(width as i32);
    let y1 = (rect.y + rect.h).min(height as i32);
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    let span = (x1 - x0) as usize;
    for row in y0..y1 {
        let row_start = pixel_index(x0 as u32, row as u32, width);
        let row_end = row_start + span * 4;
        if row_end > frame.len() {
            continue;
        }
        for px in frame[row_start..row_end].chunks_exact_mut(4) {
            px.copy_from_slice(&color);
        }
    }
}

/// Draw a filled rectangle, alpha-blending each pixel over the existing frame.
///
/// Uses the per-pixel [`crate::frame::blend_pixel`] routine. With
/// `color[3] == 0xff` this is functionally [`draw_rect_filled`] but slower —
/// prefer that for fully opaque fills.
pub fn draw_rect_filled_alpha(
    frame: &mut [u8],
    rect: PixelRect,
    color: Color,
    width: u32,
    height: u32,
) {
    if rect.w <= 0 || rect.h <= 0 {
        return;
    }
    let x0 = rect.x.max(0);
    let y0 = rect.y.max(0);
    let x1 = (rect.x + rect.w).min(width as i32);
    let y1 = (rect.y + rect.h).min(height as i32);
    for y in y0..y1 {
        for x in x0..x1 {
            blend_pixel(frame, x, y, width, height, color);
        }
    }
}

/// Draw a 1-pixel rectangle outline.
pub fn draw_rect_stroke(
    frame: &mut [u8],
    rect: PixelRect,
    color: Color,
    width: u32,
    height: u32,
) {
    if rect.w <= 0 || rect.h <= 0 {
        return;
    }
    let x1 = rect.x + rect.w - 1;
    let y1 = rect.y + rect.h - 1;
    for xi in rect.x..=x1 {
        put_pixel(frame, xi, rect.y, width, height, color);
        put_pixel(frame, xi, y1, width, height, color);
    }
    for yi in rect.y..=y1 {
        put_pixel(frame, rect.x, yi, width, height, color);
        put_pixel(frame, x1, yi, width, height, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::palette;

    const W: u32 = 20;
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
    fn filled_rect_paints_exact_area() {
        let mut f = make_frame();
        draw_rect_filled(&mut f, PixelRect::new(2, 3, 5, 4), palette::RED, W, H);
        assert_eq!(count(&f, palette::RED), 5 * 4);
        assert_eq!(pixel_at(&f, 2, 3), palette::RED);
        assert_eq!(pixel_at(&f, 6, 6), palette::RED);
        // Outside the rect should be untouched
        assert_eq!(pixel_at(&f, 1, 3), [0, 0, 0, 0]);
        assert_eq!(pixel_at(&f, 7, 6), [0, 0, 0, 0]);
    }

    #[test]
    fn filled_rect_clipped_to_frame() {
        let mut f = make_frame();
        draw_rect_filled(&mut f, PixelRect::new(-2, -2, 5, 5), palette::RED, W, H);
        // Only the (0..3, 0..3) portion is inside
        assert_eq!(count(&f, palette::RED), 3 * 3);
    }

    #[test]
    fn filled_rect_zero_size_no_op() {
        let mut f = make_frame();
        draw_rect_filled(&mut f, PixelRect::new(0, 0, 0, 0), palette::RED, W, H);
        draw_rect_filled(&mut f, PixelRect::new(0, 0, -5, 10), palette::RED, W, H);
        assert!(f.iter().all(|b| *b == 0));
    }

    #[test]
    fn filled_rect_alpha_blends_over_background() {
        let mut f = make_frame();
        // Start with opaque white background
        draw_rect_filled(&mut f, PixelRect::new(0, 0, W as i32, H as i32), palette::WHITE, W, H);
        // Blend 50% red on top
        draw_rect_filled_alpha(&mut f, PixelRect::new(2, 2, 4, 4), [0xff, 0x00, 0x00, 0x80], W, H);
        // Pixel inside should be mixed (high R, lower G/B), not pure white or pure red
        let mid = pixel_at(&f, 4, 4);
        assert!(mid[0] > 200, "R should remain high");
        assert!(mid[1] < 200 && mid[1] > 100, "G should be halved");
        assert!(mid[2] < 200 && mid[2] > 100, "B should be halved");
        // Pixel outside the blended rect should still be white
        assert_eq!(pixel_at(&f, 0, 0), palette::WHITE);
    }

    #[test]
    fn stroke_rect_paints_perimeter_only() {
        let mut f = make_frame();
        draw_rect_stroke(&mut f, PixelRect::new(5, 5, 4, 4), palette::WHITE, W, H);
        // 4x4 outline: 4*4 - 2*2 = 12 perimeter pixels
        assert_eq!(count(&f, palette::WHITE), 12);
        // Interior is unpainted
        assert_eq!(pixel_at(&f, 6, 6), [0, 0, 0, 0]);
        assert_eq!(pixel_at(&f, 7, 7), [0, 0, 0, 0]);
        // Corners are painted
        assert_eq!(pixel_at(&f, 5, 5), palette::WHITE);
        assert_eq!(pixel_at(&f, 8, 8), palette::WHITE);
    }
}
