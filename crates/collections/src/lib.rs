//! Reusable higher-level objects built on top of Gravita primitives.
//!
//! This crate provides ready-made game objects that combine physics,
//! rendering, and gameplay logic. Use these as starting points for
//! your own game characters and entities.
//!
//! # Available Objects
//!
//! - [`Stickman`] — Animated humanoid character with walking and jumping
//! - [`Spaceship`] — Thrust-based vehicle with rotation controls
//! - [`Planet`] — Static celestial body for orbital mechanics demos
//!
//! # Example
//!
//! ```ignore
//! use gravita_collections::Stickman;
//!
//! let mut stickman = Stickman::new(ground_y, screen_width);
//!
//! // Game loop
//! stickman.set_move_direction(1.0); // Move right
//! stickman.jump();
//! stickman.update(dt, screen_width);
//! stickman.render(&mut frame, width, height);
//! ```

#![warn(missing_docs)]

mod planet;
mod spaceship;
mod stickman;

pub use planet::Planet;
pub use spaceship::Spaceship;
pub use stickman::Stickman;
