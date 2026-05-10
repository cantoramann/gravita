//! Minimal 2D rendering helpers shared across examples.
//!
//! This crate intentionally stays lightweight: it provides simple drawing
//! primitives on top of a raw RGBA frame buffer. Higher-level scene logic
//! lives in the individual examples or in `gravita_collections`.
//!
//! # Design Philosophy
//!
//! - **CPU-based**: All rendering happens on the CPU, no GPU required
//! - **Frame buffer**: Works with any `&mut [u8]` RGBA buffer
//! - **Coordinate system**: Y increases downward (screen space). Callers that
//!   work in world-space Y-up must flip before drawing.
//!
//! # Module Layout
//!
//! - [`color`] — [`Color`] type alias, [`rgb`]/[`rgba`] helpers, [`palette`]
//! - [`frame`] — Low-level pixel helpers ([`pixel_index`], [`put_pixel`],
//!   [`blend_pixel`])
//! - [`primitives`] — Drawing routines (`clear`, `draw_circle`, `draw_line`,
//!   `draw_axes`, `draw_rect_filled`, `draw_rect_stroke`)
//!
//! All primitives are re-exported at the crate root for convenience.
//!
//! # Example
//!
//! ```ignore
//! use gravita_math::Vec2;
//! use gravita_renderer::{clear, draw_circle, palette};
//!
//! let mut frame = vec![0u8; 800 * 600 * 4];
//! clear(&mut frame, palette::DARK_BLUE_BG);
//! draw_circle(&mut frame, Vec2::new(400.0, 300.0), 50.0, palette::WHITE, 800, 600);
//! ```

#![warn(missing_docs)]

pub mod color;
pub mod frame;
pub mod primitives;
pub mod text;

pub use color::{Color, palette, rgb, rgba};
pub use frame::{blend_pixel, pixel_index, put_pixel};
pub use primitives::{
    PixelRect, clear, draw_axes, draw_circle, draw_circle_alpha, draw_line, draw_rect_filled,
    draw_rect_filled_alpha, draw_rect_stroke,
};
pub use text::{draw_char, draw_char_scaled, draw_text, draw_text_centered, draw_text_scaled};
