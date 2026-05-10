//! Drawing primitives that write into an RGBA frame buffer.
//!
//! Each primitive lives in its own submodule and is re-exported here.

pub mod axes;
pub mod circle;
pub mod clear;
pub mod line;
pub mod rect;

pub use axes::draw_axes;
pub use circle::{draw_circle, draw_circle_alpha};
pub use clear::clear;
pub use line::draw_line;
pub use rect::{PixelRect, draw_rect_filled, draw_rect_filled_alpha, draw_rect_stroke};
