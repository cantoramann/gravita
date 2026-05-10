# gravita-renderer

CPU framebuffer renderer for 2D. No GPU required — works with any `&mut [u8]` of RGBA pixels.

Used as a debug fallback and by all 2D examples; if you need real-time 3D, look at [`gravita-renderer-3d`](../renderer-3d).

## Module layout

```text
gravita-renderer/
├── color.rs       — Color type alias + rgb/rgba helpers + palette constants
├── frame.rs       — pixel_index, put_pixel, blend_pixel (one bounds-check path)
├── text.rs        — 5×7 bitmap font + draw_text / draw_text_scaled / draw_text_centered
└── primitives/
    ├── clear.rs   — clear(frame, color)
    ├── circle.rs  — draw_circle, draw_circle_alpha
    ├── line.rs    — draw_line (Bresenham)
    ├── axes.rs    — draw_axes (debug crosshair)
    └── rect.rs    — PixelRect, draw_rect_filled, draw_rect_filled_alpha, draw_rect_stroke
```

All primitive functions funnel their pixel writes through `frame::put_pixel` (opaque) or `frame::blend_pixel` (alpha blend). That's the only place bounds-check + pixel-index arithmetic lives — making the renderer auditable in one read.

## Quick example

```rust
use gravita_math::Vec2;
use gravita_renderer::{clear, draw_circle, draw_line, draw_text, palette, PixelRect, draw_rect_filled};

let mut frame = vec![0u8; 800 * 600 * 4];

clear(&mut frame, palette::DARK_BLUE_BG);

draw_circle(
    &mut frame,
    Vec2::new(400.0, 300.0),
    50.0,
    palette::WHITE,
    800, 600,
);

draw_line(
    &mut frame,
    Vec2::new(100.0, 100.0), Vec2::new(700.0, 500.0),
    palette::RED,
    800, 600,
);

draw_rect_filled(
    &mut frame,
    PixelRect::new(50, 50, 200, 100),
    palette::CYAN,
    800, 600,
);

draw_text(&mut frame, "HELLO", 10, 10, palette::WHITE, 800, 600);
```

## Coordinate system

- Origin `(0, 0)` is at the **top-left** (screen space).
- X increases right, Y increases **down**.
- World-space code typically uses Y-up. Flip when drawing:

```rust
let screen_y = (height as f32) - world_y;
```

## Alpha primitives

The `_alpha` variants run pixel writes through `blend_pixel` instead of `put_pixel`. Use them for particles, ghost previews, glow effects:

```rust
draw_circle_alpha(&mut frame, center, radius, [0xff, 0x80, 0x40, 0x60], w, h); // 60/255 ≈ 24% alpha
```

For fully opaque colors (`alpha == 0xff`), prefer the non-alpha variants — they're a few percent faster.

## License

MIT — see [../../LICENSE](../../LICENSE).
