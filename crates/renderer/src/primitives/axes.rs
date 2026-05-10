//! Draw a horizontal/vertical axis cross for debug overlays.

use gravita_math::Vec2;

use crate::{color::Color, primitives::line::draw_line};

/// Draw simple X/Y axes crossing at `origin`, useful for debugging.
pub fn draw_axes(frame: &mut [u8], origin: Vec2, color: Color, width: u32, height: u32) {
    draw_line(
        frame,
        Vec2::new(0.0, origin.y),
        Vec2::new(width as f32, origin.y),
        color,
        width,
        height,
    );
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

    #[test]
    fn axes_draws_cross() {
        let mut f = make_frame();
        draw_axes(&mut f, Vec2::new(50.0, 50.0), palette::BLUE, WIDTH, HEIGHT);
        assert_eq!(pixel_at(&f, 0, 50), palette::BLUE);
        assert_eq!(pixel_at(&f, 99, 50), palette::BLUE);
        assert_eq!(pixel_at(&f, 50, 0), palette::BLUE);
        assert_eq!(pixel_at(&f, 50, 99), palette::BLUE);
    }

    #[test]
    fn axes_at_origin_does_not_panic() {
        let mut f = make_frame();
        draw_axes(&mut f, Vec2::new(0.0, 0.0), palette::BLUE, WIDTH, HEIGHT);
    }
}
