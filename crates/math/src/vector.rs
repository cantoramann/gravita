//! Trait shared by [`Vec2`](crate::Vec2) and [`Vec3`](crate::Vec3).
//!
//! Most dim-agnostic algorithms only need the basic algebra (`dot`, `length`,
//! `lerp`, `normalize`, ...) — this trait lets you write them once and run them
//! against either dimension.
//!
//! ```
//! use gravita_math::{Vec2, Vec3, Vector};
//!
//! fn move_toward<V: Vector>(p: V, target: V, max_step: f32) -> V {
//!     let delta = target - p;
//!     let dist = delta.length();
//!     if dist <= max_step {
//!         target
//!     } else {
//!         p + delta * (max_step / dist)
//!     }
//! }
//!
//! let p2 = move_toward(Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), 3.0);
//! let p3 = move_toward(Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 0.0, 0.0), 3.0);
//! assert!((p2.x - 3.0).abs() < 1e-5);
//! assert!((p3.x - 3.0).abs() < 1e-5);
//! ```

use std::ops::{Add, Mul, Sub};

/// Common interface for `Vec2` and `Vec3`.
///
/// Implementors are expected to be `Copy` and define vector addition,
/// subtraction, scalar multiplication, dot product, and length. The trait
/// provides defaulted helpers (`length`, `normalize`, `distance`, `lerp`,
/// `reflect`) so implementations only need to supply the primitives.
pub trait Vector:
    Sized
    + Copy
    + Default
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<f32, Output = Self>
{
    /// The zero vector.
    const ZERO: Self;

    /// Dot product.
    fn dot(self, other: Self) -> f32;

    /// Squared length. Cheaper than [`length`](Self::length) when comparing.
    fn length_squared(self) -> f32 {
        self.dot(self)
    }

    /// Euclidean length.
    fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    /// Unit-length copy. Returns [`Self::ZERO`] for the zero vector.
    fn normalize(self) -> Self {
        let len = self.length();
        if len > 0.0 { self * (1.0 / len) } else { Self::ZERO }
    }

    /// Euclidean distance.
    fn distance(self, other: Self) -> f32 {
        (self - other).length()
    }

    /// Squared distance (avoids the sqrt).
    fn distance_squared(self, other: Self) -> f32 {
        (self - other).length_squared()
    }

    /// Linearly interpolate towards `other` by factor `t`.
    fn lerp(self, other: Self, t: f32) -> Self {
        self + (other - self) * t
    }

    /// Reflect this vector around `normal`. `normal` must be unit length.
    fn reflect(self, normal: Self) -> Self {
        self - normal * (2.0 * self.dot(normal))
    }
}
