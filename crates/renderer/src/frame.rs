//! Shared low-level helpers for writing into an RGBA frame buffer.
//!
//! All primitives in the [`primitives`](crate::primitives) module funnel
//! through these helpers so that pixel-index arithmetic and bounds checks
//! live in exactly one place.

use crate::color::Color;

/// Compute the byte offset of pixel `(x, y)` in a `width`-wide RGBA frame.
#[inline]
#[must_use]
pub const fn pixel_index(x: u32, y: u32, width: u32) -> usize {
    ((y * width + x) * 4) as usize
}

/// Write one opaque pixel at integer coordinates, bounds-checked.
///
/// No-op if `(x, y)` is outside `[0, width) x [0, height)` or if the
/// computed index would exceed the frame slice.
#[inline]
pub fn put_pixel(frame: &mut [u8], x: i32, y: i32, width: u32, height: u32, color: Color) {
    if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
        return;
    }
    let idx = pixel_index(x as u32, y as u32, width);
    if idx + 4 <= frame.len() {
        frame[idx..idx + 4].copy_from_slice(&color);
    }
}

/// Alpha-blend `src` over the pixel at `(x, y)` using the source alpha.
///
/// Skips writes outside the frame bounds. With `src[3] == 0xff` this is
/// equivalent to [`put_pixel`].
#[inline]
pub fn blend_pixel(frame: &mut [u8], x: i32, y: i32, width: u32, height: u32, src: Color) {
    if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
        return;
    }
    let idx = pixel_index(x as u32, y as u32, width);
    if idx + 4 > frame.len() {
        return;
    }
    let alpha = src[3] as f32 / 255.0;
    let inv = 1.0 - alpha;
    frame[idx] = (src[0] as f32).mul_add(alpha, frame[idx] as f32 * inv) as u8;
    frame[idx + 1] = (src[1] as f32).mul_add(alpha, frame[idx + 1] as f32 * inv) as u8;
    frame[idx + 2] = (src[2] as f32).mul_add(alpha, frame[idx + 2] as f32 * inv) as u8;
    // Saturate alpha: alpha_out = alpha_src + alpha_dst * (1 - alpha_src)
    let dst_a = frame[idx + 3] as f32 / 255.0;
    let out_a = dst_a.mul_add(inv, alpha);
    frame[idx + 3] = (out_a * 255.0).clamp(0.0, 255.0) as u8;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::palette;

    const W: u32 = 4;
    const H: u32 = 4;

    fn frame() -> Vec<u8> {
        vec![0u8; (W * H * 4) as usize]
    }

    #[test]
    fn pixel_index_origin() {
        assert_eq!(pixel_index(0, 0, W), 0);
    }

    #[test]
    fn pixel_index_row_stride() {
        assert_eq!(pixel_index(0, 1, W), (W * 4) as usize);
    }

    #[test]
    fn put_pixel_writes_color() {
        let mut f = frame();
        put_pixel(&mut f, 2, 1, W, H, palette::RED);
        let idx = pixel_index(2, 1, W);
        assert_eq!(&f[idx..idx + 4], &palette::RED);
    }

    #[test]
    fn put_pixel_skips_oob_negative() {
        let mut f = frame();
        put_pixel(&mut f, -1, 0, W, H, palette::RED);
        put_pixel(&mut f, 0, -1, W, H, palette::RED);
        assert!(f.iter().all(|b| *b == 0));
    }

    #[test]
    fn put_pixel_skips_oob_positive() {
        let mut f = frame();
        put_pixel(&mut f, W as i32, 0, W, H, palette::RED);
        put_pixel(&mut f, 0, H as i32, W, H, palette::RED);
        assert!(f.iter().all(|b| *b == 0));
    }

    #[test]
    fn blend_pixel_full_alpha_equivalent_to_put() {
        let mut a = frame();
        let mut b = frame();
        put_pixel(&mut a, 1, 1, W, H, palette::RED);
        blend_pixel(&mut b, 1, 1, W, H, palette::RED);
        assert_eq!(a, b);
    }

    #[test]
    fn blend_pixel_half_alpha_mixes_channels() {
        let mut f = frame();
        // Start with full-white opaque
        put_pixel(&mut f, 0, 0, W, H, palette::WHITE);
        // Blend red at 50% over white -> ~half-red, half-white
        blend_pixel(&mut f, 0, 0, W, H, [0xff, 0x00, 0x00, 0x80]);
        let idx = pixel_index(0, 0, W);
        // R channel: 255*0.498 + 255*0.5 ≈ 255 -> stays near 255 because both src and dst have R contribution
        // Actually: dst R = 255 * (1-0.5) + 255 * 0.5 = 127.5 + 127.5 = 255
        // Wait, src is [0xff, 0, 0] (red). dst was [0xff, 0xff, 0xff]. So:
        //   R_out = 255*0.5 + 255*0.5 = 255
        //   G_out = 255*0.5 + 0*0.5 = ~127
        //   B_out = 255*0.5 + 0*0.5 = ~127
        assert!(f[idx] > 200, "R channel should remain bright");
        assert!(f[idx + 1] < 200 && f[idx + 1] > 100, "G channel halved");
        assert!(f[idx + 2] < 200 && f[idx + 2] > 100, "B channel halved");
    }
}
