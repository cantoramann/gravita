# gravita-renderer

Minimal CPU-based 2D rendering utilities.

## Features

- Frame buffer rendering (works with any RGBA `&mut [u8]`)
- Drawing primitives: circles, lines, rectangles
- No GPU dependencies
- Suitable for prototyping and simple games

## Usage

```rust
use gravita_renderer::{clear, draw_circle, draw_line, draw_axes};
use gravita_math::Vec2;

// Create a frame buffer (800x600, RGBA)
let mut frame = vec![0u8; 800 * 600 * 4];

// Clear to dark blue
clear(&mut frame, [0x20, 0x20, 0x40, 0xff]);

// Draw a white circle
draw_circle(
    &mut frame,
    Vec2::new(400.0, 300.0), // center
    50.0,                     // radius
    [0xff, 0xff, 0xff, 0xff], // color (RGBA)
    800, 600                  // frame dimensions
);

// Draw a red line
draw_line(
    &mut frame,
    Vec2::new(100.0, 100.0), // start
    Vec2::new(700.0, 500.0), // end
    [0xff, 0x00, 0x00, 0xff], // color
    800, 600
);

// Draw debug axes
draw_axes(&mut frame, Vec2::new(400.0, 300.0), [0x80; 4], 800, 600);
```

## Coordinate System

The renderer works in screen coordinates where:
- Origin (0, 0) is at top-left
- X increases to the right
- Y increases downward

For physics integration, you typically need to flip Y:
```rust
let screen_y = screen_height - world_y;
```

## License

MIT OR Apache-2.0

