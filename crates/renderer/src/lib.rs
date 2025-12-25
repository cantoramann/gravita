//! Minimal 2D rendering helpers shared across examples.
//!
//! This crate intentionally stays lightweight: it provides
//! simple drawing primitives on top of a raw RGBA frame buffer.
//! Higher-level scene logic lives in the individual examples.
//!
//! # Design Philosophy
//!
//! - **CPU-based**: All rendering happens on the CPU, no GPU required
//! - **Frame buffer**: Works with any `&mut [u8]` RGBA buffer
//! - **Coordinate system**: Y increases upward (world space), caller handles conversion
//!
//! # Available Primitives
//!
//! - [`clear`] — Fill the entire frame with a solid color
//! - [`draw_circle`] — Draw a filled circle
//! - [`draw_line`] — Draw a 1-pixel wide line (Bresenham's algorithm)
//! - [`draw_axes`] — Draw X/Y debug axes
//!
//! # Example
//!
//! ```ignore
//! use gravita_renderer::{clear, draw_circle, draw_line};
//! use gravita_math::Vec2;
//!
//! let mut frame = vec![0u8; 800 * 600 * 4];
//!
//! // Clear to dark blue
//! clear(&mut frame, [0x20, 0x20, 0x40, 0xff]);
//!
//! // Draw a white circle
//! draw_circle(&mut frame, Vec2::new(400.0, 300.0), 50.0, [0xff; 4], 800, 600);
//! ```

#![warn(missing_docs)]

use gravita_math::Vec2;

/// Clear the entire frame to a solid color (RGBA).
pub fn clear(frame: &mut [u8], color: [u8; 4]) {
    for px in frame.chunks_exact_mut(4) {
        px.copy_from_slice(&color);
    }
}

/// Draw a filled circle into the frame.
pub fn draw_circle(
    frame: &mut [u8],
    center: Vec2,
    radius: f32,
    color: [u8; 4],
    width: u32,
    height: u32,
) {
    let cx = center.x.round() as i32;
    let cy = center.y.round() as i32;
    let r = radius.round() as i32;

    for y in (cy - r).max(0)..(cy + r).min(height as i32) {
        for x in (cx - r).max(0)..(cx + r).min(width as i32) {
            let dx = x - cx;
            let dy = y - cy;
            if dx * dx + dy * dy <= r * r {
                let idx = ((y as u32 * width + x as u32) * 4) as usize;
                if idx + 3 < frame.len() {
                    frame[idx..idx + 4].copy_from_slice(&color);
                }
            }
        }
    }
}

/// Draw a 1‑pixel wide line between two points using Bresenham's algorithm.
pub fn draw_line(
    frame: &mut [u8],
    start: Vec2,
    end: Vec2,
    color: [u8; 4],
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
        if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
            let idx = ((y as u32 * width + x as u32) * 4) as usize;
            if idx + 3 < frame.len() {
                frame[idx..idx + 4].copy_from_slice(&color);
            }
        }

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

/// Draw simple X/Y axes crossing at `origin`, useful for debugging.
pub fn draw_axes(frame: &mut [u8], origin: Vec2, color: [u8; 4], width: u32, height: u32) {
    let origin = Vec2::new(origin.x, origin.y);
    // Horizontal axis
    draw_line(
        frame,
        Vec2::new(0.0, origin.y),
        Vec2::new(width as f32, origin.y),
        color,
        width,
        height,
    );
    // Vertical axis
    draw_line(
        frame,
        Vec2::new(origin.x, 0.0),
        Vec2::new(origin.x, height as f32),
        color,
        width,
        height,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    const WIDTH: u32 = 100;
    const HEIGHT: u32 = 100;

    fn create_frame() -> Vec<u8> {
        vec![0u8; (WIDTH * HEIGHT * 4) as usize]
    }

    fn pixel_at(frame: &[u8], x: u32, y: u32) -> [u8; 4] {
        let idx = ((y * WIDTH + x) * 4) as usize;
        [frame[idx], frame[idx + 1], frame[idx + 2], frame[idx + 3]]
    }

    // =========================================================================
    // Clear
    // =========================================================================

    #[test]
    fn clear_fills_entire_frame() {
        let mut frame = create_frame();
        let color = [0x12, 0x34, 0x56, 0x78];
        clear(&mut frame, color);

        // Check first pixel
        assert_eq!(pixel_at(&frame, 0, 0), color);
        // Check last pixel
        assert_eq!(pixel_at(&frame, WIDTH - 1, HEIGHT - 1), color);
        // Check middle pixel
        assert_eq!(pixel_at(&frame, 50, 50), color);
    }

    #[test]
    fn clear_with_black() {
        let mut frame = create_frame();
        // Fill with white first
        clear(&mut frame, [0xFF, 0xFF, 0xFF, 0xFF]);
        // Then clear to black
        clear(&mut frame, [0x00, 0x00, 0x00, 0xFF]);
        assert_eq!(pixel_at(&frame, 50, 50), [0x00, 0x00, 0x00, 0xFF]);
    }

    // =========================================================================
    // Draw Circle
    // =========================================================================

    #[test]
    fn draw_circle_at_center() {
        let mut frame = create_frame();
        let color = [0xFF, 0x00, 0x00, 0xFF];
        draw_circle(
            &mut frame,
            Vec2::new(50.0, 50.0),
            10.0,
            color,
            WIDTH,
            HEIGHT,
        );

        // Center should be colored
        assert_eq!(pixel_at(&frame, 50, 50), color);
    }

    #[test]
    fn draw_circle_does_not_affect_outside() {
        let mut frame = create_frame();
        let color = [0xFF, 0x00, 0x00, 0xFF];
        draw_circle(&mut frame, Vec2::new(50.0, 50.0), 5.0, color, WIDTH, HEIGHT);

        // Far corner should still be black (unaffected)
        assert_eq!(pixel_at(&frame, 0, 0), [0, 0, 0, 0]);
    }

    #[test]
    fn draw_circle_clipped_at_edge() {
        let mut frame = create_frame();
        let color = [0xFF, 0x00, 0x00, 0xFF];
        // Circle partially outside left edge
        draw_circle(
            &mut frame,
            Vec2::new(-5.0, 50.0),
            10.0,
            color,
            WIDTH,
            HEIGHT,
        );

        // Should not panic and should affect edge pixels
        assert_eq!(pixel_at(&frame, 0, 50), color);
    }

    #[test]
    fn draw_circle_completely_outside_does_not_panic() {
        let mut frame = create_frame();
        let color = [0xFF, 0x00, 0x00, 0xFF];
        // Circle completely outside
        draw_circle(
            &mut frame,
            Vec2::new(-100.0, -100.0),
            10.0,
            color,
            WIDTH,
            HEIGHT,
        );
        // Should not panic - frame should be unchanged
        assert_eq!(pixel_at(&frame, 0, 0), [0, 0, 0, 0]);
    }

    #[test]
    fn draw_circle_zero_radius() {
        let mut frame = create_frame();
        let color = [0xFF, 0x00, 0x00, 0xFF];
        // Zero radius circle (essentially a point)
        draw_circle(&mut frame, Vec2::new(50.0, 50.0), 0.0, color, WIDTH, HEIGHT);
        // Should not panic
    }

    // =========================================================================
    // Draw Line
    // =========================================================================

    #[test]
    fn draw_line_horizontal() {
        let mut frame = create_frame();
        let color = [0x00, 0xFF, 0x00, 0xFF];
        draw_line(
            &mut frame,
            Vec2::new(10.0, 50.0),
            Vec2::new(90.0, 50.0),
            color,
            WIDTH,
            HEIGHT,
        );

        // Start point
        assert_eq!(pixel_at(&frame, 10, 50), color);
        // End point
        assert_eq!(pixel_at(&frame, 90, 50), color);
        // Middle point
        assert_eq!(pixel_at(&frame, 50, 50), color);
    }

    #[test]
    fn draw_line_vertical() {
        let mut frame = create_frame();
        let color = [0x00, 0xFF, 0x00, 0xFF];
        draw_line(
            &mut frame,
            Vec2::new(50.0, 10.0),
            Vec2::new(50.0, 90.0),
            color,
            WIDTH,
            HEIGHT,
        );

        assert_eq!(pixel_at(&frame, 50, 10), color);
        assert_eq!(pixel_at(&frame, 50, 90), color);
        assert_eq!(pixel_at(&frame, 50, 50), color);
    }

    #[test]
    fn draw_line_diagonal() {
        let mut frame = create_frame();
        let color = [0x00, 0xFF, 0x00, 0xFF];
        draw_line(
            &mut frame,
            Vec2::new(10.0, 10.0),
            Vec2::new(90.0, 90.0),
            color,
            WIDTH,
            HEIGHT,
        );

        // Start and end should be colored
        assert_eq!(pixel_at(&frame, 10, 10), color);
        assert_eq!(pixel_at(&frame, 90, 90), color);
    }

    #[test]
    fn draw_line_clipped_does_not_panic() {
        let mut frame = create_frame();
        let color = [0x00, 0xFF, 0x00, 0xFF];
        // Line extending outside frame
        draw_line(
            &mut frame,
            Vec2::new(-50.0, 50.0),
            Vec2::new(150.0, 50.0),
            color,
            WIDTH,
            HEIGHT,
        );
        // Should not panic
        assert_eq!(pixel_at(&frame, 0, 50), color);
        assert_eq!(pixel_at(&frame, 99, 50), color);
    }

    #[test]
    fn draw_line_completely_outside_does_not_panic() {
        let mut frame = create_frame();
        let color = [0x00, 0xFF, 0x00, 0xFF];
        draw_line(
            &mut frame,
            Vec2::new(-50.0, -50.0),
            Vec2::new(-10.0, -10.0),
            color,
            WIDTH,
            HEIGHT,
        );
        // Should not panic - frame unchanged
        assert_eq!(pixel_at(&frame, 0, 0), [0, 0, 0, 0]);
    }

    #[test]
    fn draw_line_single_point() {
        let mut frame = create_frame();
        let color = [0x00, 0xFF, 0x00, 0xFF];
        // Start == end
        draw_line(
            &mut frame,
            Vec2::new(50.0, 50.0),
            Vec2::new(50.0, 50.0),
            color,
            WIDTH,
            HEIGHT,
        );
        assert_eq!(pixel_at(&frame, 50, 50), color);
    }

    // =========================================================================
    // Draw Axes
    // =========================================================================

    #[test]
    fn draw_axes_draws_cross() {
        let mut frame = create_frame();
        let color = [0x00, 0x00, 0xFF, 0xFF];
        draw_axes(&mut frame, Vec2::new(50.0, 50.0), color, WIDTH, HEIGHT);

        // Check horizontal axis
        assert_eq!(pixel_at(&frame, 0, 50), color);
        assert_eq!(pixel_at(&frame, 99, 50), color);
        // Check vertical axis
        assert_eq!(pixel_at(&frame, 50, 0), color);
        assert_eq!(pixel_at(&frame, 50, 99), color);
    }

    #[test]
    fn draw_axes_at_origin() {
        let mut frame = create_frame();
        let color = [0x00, 0x00, 0xFF, 0xFF];
        draw_axes(&mut frame, Vec2::new(0.0, 0.0), color, WIDTH, HEIGHT);
        // Should not panic
    }

    // =========================================================================
    // Visual Output Correctness Tests
    // =========================================================================

    /// Count how many pixels have a specific color
    fn count_pixels_with_color(frame: &[u8], color: [u8; 4]) -> usize {
        frame.chunks_exact(4).filter(|px| px == &color).count()
    }

    /// Check if a pixel is colored (non-zero)
    #[allow(dead_code)]
    fn is_pixel_colored(frame: &[u8], x: u32, y: u32, width: u32) -> bool {
        let idx = ((y * width + x) * 4) as usize;
        frame[idx] != 0 || frame[idx + 1] != 0 || frame[idx + 2] != 0 || frame[idx + 3] != 0
    }

    #[test]
    fn visual_circle_fills_approximate_area() {
        let mut frame = create_frame();
        let color = [0xFF, 0x00, 0x00, 0xFF];
        let radius = 10.0;
        draw_circle(
            &mut frame,
            Vec2::new(50.0, 50.0),
            radius,
            color,
            WIDTH,
            HEIGHT,
        );

        let filled_pixels = count_pixels_with_color(&frame, color);
        // Theoretical area: π * r² ≈ 314.159
        // Due to rasterization, actual count will differ but should be close
        let expected_area = std::f32::consts::PI * radius * radius;
        let tolerance = expected_area * 0.15; // 15% tolerance for rasterization

        assert!(
            (filled_pixels as f32 - expected_area).abs() < tolerance,
            "Circle area mismatch: got {filled_pixels} pixels, expected ~{expected_area:.0} \
             (±{tolerance:.0})"
        );
    }

    #[test]
    fn visual_circle_is_symmetric() {
        let mut frame = create_frame();
        let color = [0xFF, 0x00, 0x00, 0xFF];
        draw_circle(
            &mut frame,
            Vec2::new(50.0, 50.0),
            15.0,
            color,
            WIDTH,
            HEIGHT,
        );

        // Check horizontal symmetry (exclude boundary at radius=15 due to rasterization)
        for offset in 1..=14 {
            let left = pixel_at(&frame, 50 - offset, 50);
            let right = pixel_at(&frame, 50 + offset, 50);
            assert_eq!(left, right, "Horizontal asymmetry at offset {offset}");
        }

        // Check vertical symmetry (exclude boundary)
        for offset in 1..=14 {
            let up = pixel_at(&frame, 50, 50 - offset);
            let down = pixel_at(&frame, 50, 50 + offset);
            assert_eq!(up, down, "Vertical asymmetry at offset {offset}");
        }
    }

    #[test]
    fn visual_circle_boundary_is_correct() {
        let mut frame = create_frame();
        let color = [0xFF, 0x00, 0x00, 0xFF];
        let radius = 10.0;
        draw_circle(
            &mut frame,
            Vec2::new(50.0, 50.0),
            radius,
            color,
            WIDTH,
            HEIGHT,
        );

        // Pixels just inside radius should be colored
        assert_eq!(pixel_at(&frame, 50, 41), color); // Just inside top
        assert_eq!(pixel_at(&frame, 50, 59), color); // Just inside bottom
        assert_eq!(pixel_at(&frame, 41, 50), color); // Just inside left
        assert_eq!(pixel_at(&frame, 59, 50), color); // Just inside right

        // Pixels outside radius should NOT be colored
        assert_eq!(pixel_at(&frame, 50, 38), [0, 0, 0, 0]); // Outside top
        assert_eq!(pixel_at(&frame, 50, 62), [0, 0, 0, 0]); // Outside bottom
    }

    #[test]
    fn visual_line_is_continuous() {
        let mut frame = create_frame();
        let color = [0x00, 0xFF, 0x00, 0xFF];
        draw_line(
            &mut frame,
            Vec2::new(10.0, 10.0),
            Vec2::new(90.0, 90.0),
            color,
            WIDTH,
            HEIGHT,
        );

        // For a 45° diagonal, Bresenham should produce a continuous line
        // Check that no pixel along the expected path is uncolored
        let mut colored_count = 0;
        for i in 10..=90 {
            // The diagonal should color pixels at approximately (i, i)
            // Due to Bresenham, check if either (i, i) or adjacent pixel is colored
            if pixel_at(&frame, i, i) == color {
                colored_count += 1;
            }
        }

        // At least 80% of expected points should be directly on the line
        assert!(
            colored_count >= 65,
            "Line should be continuous: only {colored_count} of 81 pixels colored"
        );
    }

    #[test]
    fn visual_line_no_gaps_horizontal() {
        let mut frame = create_frame();
        let color = [0x00, 0xFF, 0x00, 0xFF];
        draw_line(
            &mut frame,
            Vec2::new(10.0, 50.0),
            Vec2::new(90.0, 50.0),
            color,
            WIDTH,
            HEIGHT,
        );

        // Horizontal line should have no gaps
        for x in 10..=90 {
            assert_eq!(
                pixel_at(&frame, x, 50),
                color,
                "Gap in horizontal line at x={x}"
            );
        }
    }

    #[test]
    fn visual_line_no_gaps_vertical() {
        let mut frame = create_frame();
        let color = [0x00, 0xFF, 0x00, 0xFF];
        draw_line(
            &mut frame,
            Vec2::new(50.0, 10.0),
            Vec2::new(50.0, 90.0),
            color,
            WIDTH,
            HEIGHT,
        );

        // Vertical line should have no gaps
        for y in 10..=90 {
            assert_eq!(
                pixel_at(&frame, 50, y),
                color,
                "Gap in vertical line at y={y}"
            );
        }
    }

    #[test]
    fn visual_line_steep_slope_continuous() {
        let mut frame = create_frame();
        let color = [0x00, 0xFF, 0x00, 0xFF];
        // Steep slope (more vertical than horizontal)
        draw_line(
            &mut frame,
            Vec2::new(45.0, 10.0),
            Vec2::new(55.0, 90.0),
            color,
            WIDTH,
            HEIGHT,
        );

        // Count colored pixels to ensure line is drawn
        let filled = count_pixels_with_color(&frame, color);
        // Line length ≈ sqrt((55-45)² + (90-10)²) ≈ 80.6
        assert!(
            filled >= 75,
            "Steep line should have ~80 pixels, got {filled}"
        );
    }

    #[test]
    fn visual_color_channels_independent() {
        let mut frame = create_frame();

        // Draw with pure red
        draw_circle(
            &mut frame,
            Vec2::new(25.0, 50.0),
            5.0,
            [0xFF, 0x00, 0x00, 0xFF],
            WIDTH,
            HEIGHT,
        );
        // Draw with pure green
        draw_circle(
            &mut frame,
            Vec2::new(50.0, 50.0),
            5.0,
            [0x00, 0xFF, 0x00, 0xFF],
            WIDTH,
            HEIGHT,
        );
        // Draw with pure blue
        draw_circle(
            &mut frame,
            Vec2::new(75.0, 50.0),
            5.0,
            [0x00, 0x00, 0xFF, 0xFF],
            WIDTH,
            HEIGHT,
        );

        // Verify each circle has correct color
        let red_px = pixel_at(&frame, 25, 50);
        let green_px = pixel_at(&frame, 50, 50);
        let blue_px = pixel_at(&frame, 75, 50);

        assert_eq!(red_px, [0xFF, 0x00, 0x00, 0xFF], "Red channel incorrect");
        assert_eq!(
            green_px,
            [0x00, 0xFF, 0x00, 0xFF],
            "Green channel incorrect"
        );
        assert_eq!(blue_px, [0x00, 0x00, 0xFF, 0xFF], "Blue channel incorrect");
    }

    #[test]
    fn visual_alpha_channel_preserved() {
        let mut frame = create_frame();
        let semi_transparent = [0xFF, 0x00, 0x00, 0x80]; // 50% alpha
        draw_circle(
            &mut frame,
            Vec2::new(50.0, 50.0),
            10.0,
            semi_transparent,
            WIDTH,
            HEIGHT,
        );

        let px = pixel_at(&frame, 50, 50);
        assert_eq!(px[3], 0x80, "Alpha channel not preserved");
    }

    #[test]
    fn visual_overdraw_replaces_pixels() {
        let mut frame = create_frame();

        // Draw red circle first
        draw_circle(
            &mut frame,
            Vec2::new(50.0, 50.0),
            20.0,
            [0xFF, 0x00, 0x00, 0xFF],
            WIDTH,
            HEIGHT,
        );
        // Draw smaller blue circle on top
        draw_circle(
            &mut frame,
            Vec2::new(50.0, 50.0),
            10.0,
            [0x00, 0x00, 0xFF, 0xFF],
            WIDTH,
            HEIGHT,
        );

        // Center should be blue (overwritten)
        assert_eq!(pixel_at(&frame, 50, 50), [0x00, 0x00, 0xFF, 0xFF]);
        // Outer ring should still be red
        assert_eq!(pixel_at(&frame, 50, 35), [0xFF, 0x00, 0x00, 0xFF]);
    }

    #[test]
    fn visual_multiple_shapes_composite_correctly() {
        let mut frame = create_frame();
        let bg = [0x20, 0x20, 0x20, 0xFF];
        let circle_color = [0xFF, 0x00, 0x00, 0xFF];
        let line_color = [0x00, 0xFF, 0x00, 0xFF];

        clear(&mut frame, bg);
        draw_circle(
            &mut frame,
            Vec2::new(50.0, 50.0),
            30.0,
            circle_color,
            WIDTH,
            HEIGHT,
        );
        draw_line(
            &mut frame,
            Vec2::new(0.0, 50.0),
            Vec2::new(99.0, 50.0),
            line_color,
            WIDTH,
            HEIGHT,
        );

        // Line should overwrite circle where they intersect
        assert_eq!(pixel_at(&frame, 50, 50), line_color);
        // Circle should be visible where line doesn't cross
        assert_eq!(pixel_at(&frame, 50, 30), circle_color);
        // Background should be visible outside both
        assert_eq!(pixel_at(&frame, 5, 5), bg);
    }

    #[test]
    fn visual_frame_buffer_integrity() {
        let mut frame = create_frame();
        let original_len = frame.len();

        // Draw various shapes
        clear(&mut frame, [0x10, 0x20, 0x30, 0xFF]);
        draw_circle(
            &mut frame,
            Vec2::new(50.0, 50.0),
            25.0,
            [0xFF, 0x00, 0x00, 0xFF],
            WIDTH,
            HEIGHT,
        );
        draw_line(
            &mut frame,
            Vec2::new(0.0, 0.0),
            Vec2::new(99.0, 99.0),
            [0x00, 0xFF, 0x00, 0xFF],
            WIDTH,
            HEIGHT,
        );
        draw_axes(
            &mut frame,
            Vec2::new(50.0, 50.0),
            [0x00, 0x00, 0xFF, 0xFF],
            WIDTH,
            HEIGHT,
        );

        // Frame buffer should maintain its size
        assert_eq!(
            frame.len(),
            original_len,
            "Frame buffer size changed during rendering"
        );

        // All pixels should have exactly 4 bytes (RGBA)
        for (i, chunk) in frame.chunks_exact(4).enumerate() {
            assert_eq!(chunk.len(), 4, "Pixel {i} should have exactly 4 bytes");
        }
    }
}
