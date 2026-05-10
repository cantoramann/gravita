//! Bresenham line rasterizer.

use gravita_math::Vec2;

use crate::{color::Color, frame::put_pixel};

/// Draw a 1-pixel wide line between two points using Bresenham's algorithm.
pub fn draw_line(
    frame: &mut [u8],
    start: Vec2,
    end: Vec2,
    color: Color,
    width: u32,
    height: u32,
) {
    let x0 = start.x.round() as i32;
    let y0 = start.y.round() as i32;
    let x1 = end.x.round() as i32;
    let y1 = end.y.round() as i32;

    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;

    let mut x = x0;
    let mut y = y0;

    loop {
        put_pixel(frame, x, y, width, height, color);
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{color::palette, frame::pixel_index};

    const WIDTH: u32 = 100;
    const HEIGHT: u32 = 100;

    fn make_frame() -> Vec<u8> {
        vec![0u8; (WIDTH * HEIGHT * 4) as usize]
    }

    fn pixel_at(frame: &[u8], x: u32, y: u32) -> [u8; 4] {
        let idx = pixel_index(x, y, WIDTH);
        [frame[idx], frame[idx + 1], frame[idx + 2], frame[idx + 3]]
    }

    fn count_pixels_with_color(frame: &[u8], color: Color) -> usize {
        frame.chunks_exact(4).filter(|px| px == &color).count()
    }

    #[test]
    fn line_horizontal() {
        let mut f = make_frame();
        draw_line(&mut f, Vec2::new(10.0, 50.0), Vec2::new(90.0, 50.0), palette::GREEN, WIDTH, HEIGHT);
        assert_eq!(pixel_at(&f, 10, 50), palette::GREEN);
        assert_eq!(pixel_at(&f, 90, 50), palette::GREEN);
        assert_eq!(pixel_at(&f, 50, 50), palette::GREEN);
    }

    #[test]
    fn line_vertical() {
        let mut f = make_frame();
        draw_line(&mut f, Vec2::new(50.0, 10.0), Vec2::new(50.0, 90.0), palette::GREEN, WIDTH, HEIGHT);
        assert_eq!(pixel_at(&f, 50, 10), palette::GREEN);
        assert_eq!(pixel_at(&f, 50, 90), palette::GREEN);
        assert_eq!(pixel_at(&f, 50, 50), palette::GREEN);
    }

    #[test]
    fn line_diagonal() {
        let mut f = make_frame();
        draw_line(&mut f, Vec2::new(10.0, 10.0), Vec2::new(90.0, 90.0), palette::GREEN, WIDTH, HEIGHT);
        assert_eq!(pixel_at(&f, 10, 10), palette::GREEN);
        assert_eq!(pixel_at(&f, 90, 90), palette::GREEN);
    }

    #[test]
    fn line_clipped_does_not_panic() {
        let mut f = make_frame();
        draw_line(&mut f, Vec2::new(-50.0, 50.0), Vec2::new(150.0, 50.0), palette::GREEN, WIDTH, HEIGHT);
        assert_eq!(pixel_at(&f, 0, 50), palette::GREEN);
        assert_eq!(pixel_at(&f, 99, 50), palette::GREEN);
    }

    #[test]
    fn line_completely_outside_does_not_panic() {
        let mut f = make_frame();
        draw_line(&mut f, Vec2::new(-50.0, -50.0), Vec2::new(-10.0, -10.0), palette::GREEN, WIDTH, HEIGHT);
        assert_eq!(pixel_at(&f, 0, 0), [0, 0, 0, 0]);
    }

    #[test]
    fn line_single_point() {
        let mut f = make_frame();
        draw_line(&mut f, Vec2::new(50.0, 50.0), Vec2::new(50.0, 50.0), palette::GREEN, WIDTH, HEIGHT);
        assert_eq!(pixel_at(&f, 50, 50), palette::GREEN);
    }

    #[test]
    fn line_no_gaps_horizontal() {
        let mut f = make_frame();
        draw_line(&mut f, Vec2::new(10.0, 50.0), Vec2::new(90.0, 50.0), palette::GREEN, WIDTH, HEIGHT);
        for x in 10..=90 {
            assert_eq!(pixel_at(&f, x, 50), palette::GREEN, "gap at x={x}");
        }
    }

    #[test]
    fn line_no_gaps_vertical() {
        let mut f = make_frame();
        draw_line(&mut f, Vec2::new(50.0, 10.0), Vec2::new(50.0, 90.0), palette::GREEN, WIDTH, HEIGHT);
        for y in 10..=90 {
            assert_eq!(pixel_at(&f, 50, y), palette::GREEN, "gap at y={y}");
        }
    }

    #[test]
    fn line_steep_slope_continuous() {
        let mut f = make_frame();
        draw_line(&mut f, Vec2::new(45.0, 10.0), Vec2::new(55.0, 90.0), palette::GREEN, WIDTH, HEIGHT);
        let filled = count_pixels_with_color(&f, palette::GREEN);
        assert!(filled >= 75, "Steep line should have ~80 pixels, got {filled}");
    }
}
