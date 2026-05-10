//! Filled-circle rasterizer.

use gravita_math::Vec2;

use crate::{
    color::Color,
    frame::{blend_pixel, pixel_index, put_pixel},
};

/// Draw a filled circle into the frame.
///
/// Walks the circle's bounding rows and fills each scanline via a single
/// `copy_from_slice` per span. Pixels outside `width`×`height` are skipped.
pub fn draw_circle(
    frame: &mut [u8],
    center: Vec2,
    radius: f32,
    color: Color,
    width: u32,
    height: u32,
) {
    let cx = center.x.round() as i32;
    let cy = center.y.round() as i32;
    let r = radius.round() as i32;
    if r <= 0 {
        if r == 0 {
            put_pixel(frame, cx, cy, width, height, color);
        }
        return;
    }
    let r_sq = r * r;
    let y_min = (cy - r).max(0);
    let y_max = (cy + r).min(height as i32 - 1);

    for y in y_min..=y_max {
        let dy = y - cy;
        let dx_max_sq = r_sq - dy * dy;
        if dx_max_sq < 0 {
            continue;
        }
        let dx_max = (dx_max_sq as f32).sqrt() as i32;
        let x_start = (cx - dx_max).max(0);
        let x_end = (cx + dx_max).min(width as i32 - 1);
        if x_start > x_end {
            continue;
        }
        let row_start = pixel_index(x_start as u32, y as u32, width);
        let span_pixels = (x_end - x_start + 1) as usize;
        let row_end = row_start + span_pixels * 4;
        if row_end > frame.len() {
            // Conservative fallback: write pixel-by-pixel within bounds.
            for x in x_start..=x_end {
                put_pixel(frame, x, y, width, height, color);
            }
            continue;
        }
        for px in frame[row_start..row_end].chunks_exact_mut(4) {
            px.copy_from_slice(&color);
        }
    }
}

/// Draw a filled circle, alpha-blending each pixel over the existing frame.
///
/// Use this for particles, glow effects, or any overlay where the existing
/// background must show through. Fully opaque colors (`alpha == 0xff`) work
/// but [`draw_circle`] is faster in that case.
pub fn draw_circle_alpha(
    frame: &mut [u8],
    center: Vec2,
    radius: f32,
    color: Color,
    width: u32,
    height: u32,
) {
    let cx = center.x.round() as i32;
    let cy = center.y.round() as i32;
    let r = radius.round() as i32;
    if r <= 0 {
        if r == 0 {
            blend_pixel(frame, cx, cy, width, height, color);
        }
        return;
    }
    let r_sq = r * r;
    let y_min = (cy - r).max(0);
    let y_max = (cy + r).min(height as i32 - 1);
    for y in y_min..=y_max {
        let dy = y - cy;
        let dx_max_sq = r_sq - dy * dy;
        if dx_max_sq < 0 {
            continue;
        }
        let dx_max = (dx_max_sq as f32).sqrt() as i32;
        let x_start = (cx - dx_max).max(0);
        let x_end = (cx + dx_max).min(width as i32 - 1);
        for x in x_start..=x_end {
            blend_pixel(frame, x, y, width, height, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::palette;

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
    fn circle_at_center() {
        let mut f = make_frame();
        draw_circle(
            &mut f,
            Vec2::new(50.0, 50.0),
            10.0,
            palette::RED,
            WIDTH,
            HEIGHT,
        );
        assert_eq!(pixel_at(&f, 50, 50), palette::RED);
    }

    #[test]
    fn circle_does_not_affect_outside() {
        let mut f = make_frame();
        draw_circle(
            &mut f,
            Vec2::new(50.0, 50.0),
            5.0,
            palette::RED,
            WIDTH,
            HEIGHT,
        );
        assert_eq!(pixel_at(&f, 0, 0), [0, 0, 0, 0]);
    }

    #[test]
    fn circle_clipped_at_edge() {
        let mut f = make_frame();
        draw_circle(
            &mut f,
            Vec2::new(-5.0, 50.0),
            10.0,
            palette::RED,
            WIDTH,
            HEIGHT,
        );
        assert_eq!(pixel_at(&f, 0, 50), palette::RED);
    }

    #[test]
    fn circle_completely_outside_does_not_panic() {
        let mut f = make_frame();
        draw_circle(
            &mut f,
            Vec2::new(-100.0, -100.0),
            10.0,
            palette::RED,
            WIDTH,
            HEIGHT,
        );
        assert_eq!(pixel_at(&f, 0, 0), [0, 0, 0, 0]);
    }

    #[test]
    fn circle_zero_radius_paints_center() {
        let mut f = make_frame();
        draw_circle(
            &mut f,
            Vec2::new(50.0, 50.0),
            0.0,
            palette::RED,
            WIDTH,
            HEIGHT,
        );
        assert_eq!(pixel_at(&f, 50, 50), palette::RED);
    }

    #[test]
    fn circle_area_matches_pi_r_squared() {
        let mut f = make_frame();
        let radius = 10.0;
        draw_circle(
            &mut f,
            Vec2::new(50.0, 50.0),
            radius,
            palette::RED,
            WIDTH,
            HEIGHT,
        );
        let filled = count_pixels_with_color(&f, palette::RED);
        let expected = std::f32::consts::PI * radius * radius;
        let tolerance = expected * 0.15;
        assert!(
            (filled as f32 - expected).abs() < tolerance,
            "Circle area mismatch: got {filled} pixels, expected ~{expected:.0} \
             (\u{00b1}{tolerance:.0})"
        );
    }

    #[test]
    fn circle_is_symmetric_horizontally() {
        let mut f = make_frame();
        draw_circle(
            &mut f,
            Vec2::new(50.0, 50.0),
            15.0,
            palette::RED,
            WIDTH,
            HEIGHT,
        );
        for offset in 1..=14 {
            assert_eq!(
                pixel_at(&f, 50 - offset, 50),
                pixel_at(&f, 50 + offset, 50),
                "Horizontal asymmetry at offset {offset}"
            );
        }
    }

    #[test]
    fn circle_is_symmetric_vertically() {
        let mut f = make_frame();
        draw_circle(
            &mut f,
            Vec2::new(50.0, 50.0),
            15.0,
            palette::RED,
            WIDTH,
            HEIGHT,
        );
        for offset in 1..=14 {
            assert_eq!(
                pixel_at(&f, 50, 50 - offset),
                pixel_at(&f, 50, 50 + offset),
                "Vertical asymmetry at offset {offset}"
            );
        }
    }

    #[test]
    fn circle_boundary_pixels_inside_radius_painted() {
        let mut f = make_frame();
        draw_circle(
            &mut f,
            Vec2::new(50.0, 50.0),
            10.0,
            palette::RED,
            WIDTH,
            HEIGHT,
        );
        assert_eq!(pixel_at(&f, 50, 41), palette::RED);
        assert_eq!(pixel_at(&f, 50, 59), palette::RED);
        assert_eq!(pixel_at(&f, 41, 50), palette::RED);
        assert_eq!(pixel_at(&f, 59, 50), palette::RED);
        assert_eq!(pixel_at(&f, 50, 38), [0, 0, 0, 0]);
        assert_eq!(pixel_at(&f, 50, 62), [0, 0, 0, 0]);
    }

    #[test]
    fn circle_alpha_preserved() {
        let mut f = make_frame();
        let semi = [0xff, 0x00, 0x00, 0x80];
        draw_circle(&mut f, Vec2::new(50.0, 50.0), 10.0, semi, WIDTH, HEIGHT);
        assert_eq!(pixel_at(&f, 50, 50)[3], 0x80);
    }

    #[test]
    fn circle_alpha_blends_over_background() {
        let mut f = make_frame();
        // Solid white background
        for px in f.chunks_exact_mut(4) {
            px.copy_from_slice(&palette::WHITE);
        }
        // Half-alpha red circle
        draw_circle_alpha(
            &mut f,
            Vec2::new(50.0, 50.0),
            10.0,
            [0xff, 0x00, 0x00, 0x80],
            WIDTH,
            HEIGHT,
        );
        let mid = pixel_at(&f, 50, 50);
        // R stays high (both src and dst contribute), G/B drop to ~half.
        assert!(mid[0] > 200);
        assert!(mid[1] < 200 && mid[1] > 100);
        assert!(mid[2] < 200 && mid[2] > 100);
    }
}
