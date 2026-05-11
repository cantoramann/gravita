//! # gravita-wasm — JavaScript-friendly Gravita physics bindings
//!
//! This crate is gated to `target_arch = "wasm32"` and exposes a small
//! handwritten `wasm-bindgen` surface so web developers can use Gravita
//! from JavaScript or TypeScript without writing any Rust.
//!
//! ## Build
//!
//! ```bash
//! # One-time install: rustup target add wasm32-unknown-unknown
//! # And: cargo install wasm-pack
//! wasm-pack build crates/wasm --target web --release
//! ```
//!
//! That produces an `npm`-style `pkg/` directory you can import directly:
//!
//! ```js
//! import init, { World2D } from "./pkg/gravita_wasm.js";
//!
//! await init();
//! const w = new World2D(0, -500);                 // gravity
//! const ball = w.addCircle(0, 100, 10);            // x, y, radius
//! w.setBodyRestitution(ball, 0.8);
//! for (let i = 0; i < 60; i++) w.step(1 / 60);
//! console.log(w.bodyPosition(ball));               // Float32Array [x, y]
//! ```
//!
//! ## Design
//!
//! - **Handwritten, not auto-derived.** Bindings are shaped for JS readers:
//!   plain numeric arguments, `Float32Array` returns.
//! - **Two top-level types**: `World2D` and `World3D`.
//!
//! On non-wasm targets, the crate compiles to an empty hull so
//! `cargo build --workspace` doesn't have to pull `wasm-bindgen`.

#![cfg(target_arch = "wasm32")]
#![warn(missing_docs)]
// JS-style camelCase exports; the rest of the workspace is snake_case.
#![allow(non_snake_case)]

use gravita_math::{Aabb, Aabb3, Circle, Sphere, Vec2, Vec3};
use gravita_physics::{
    BodyType as BodyType2D, CollisionShape as Shape2D, PhysicsWorld as World2DInner,
    RigidBody as Body2D, Snapshot as Snapshot2D,
};
use gravita_physics_3d::{
    BodyType as BodyType3D, CollisionShape as Shape3D, PhysicsWorld as World3DInner,
    RigidBody as Body3D, Snapshot as Snapshot3D,
};
use wasm_bindgen::prelude::*;

// ───────────────────────────────────────────────────────────────────────────
// Body kind (JS-friendly enum)
// ───────────────────────────────────────────────────────────────────────────

/// Body kind. Pass to [`World2D::setBodyKind`] / [`World3D::setBodyKind`].
///
/// In JS: `BodyKind.Dynamic`, `BodyKind.Kinematic`, `BodyKind.Static`.
#[wasm_bindgen]
#[derive(Copy, Clone, Debug)]
pub enum BodyKind {
    /// Fully simulated by forces (the default).
    Dynamic = 0,
    /// Moved by user code each step; not affected by forces.
    Kinematic = 1,
    /// Never moves.
    Static = 2,
}

impl From<BodyKind> for BodyType2D {
    fn from(k: BodyKind) -> Self {
        match k {
            BodyKind::Dynamic => Self::Dynamic,
            BodyKind::Kinematic => Self::Kinematic,
            BodyKind::Static => Self::Static,
        }
    }
}

impl From<BodyKind> for BodyType3D {
    fn from(k: BodyKind) -> Self {
        match k {
            BodyKind::Dynamic => Self::Dynamic,
            BodyKind::Kinematic => Self::Kinematic,
            BodyKind::Static => Self::Static,
        }
    }
}

// ───────────────────────────────────────────────────────────────────────────
// 2D
// ───────────────────────────────────────────────────────────────────────────

/// 2D physics world.
///
/// ```js
/// const w = new World2D(0, -500);
/// const id = w.addCircle(0, 100, 10);
/// w.setBodyRestitution(id, 0.8);
/// w.step(1 / 60);
/// console.log(w.bodyPosition(id));    // Float32Array [x, y]
/// ```
#[wasm_bindgen]
pub struct World2D {
    inner: World2DInner,
}

#[wasm_bindgen]
impl World2D {
    /// Construct a world with the given linear gravity (Y is up — positive Y
    /// is "up", negative Y is "down" like real physics).
    #[wasm_bindgen(constructor)]
    pub fn new(gravity_x: f32, gravity_y: f32) -> World2D {
        let mut inner = World2DInner::new();
        inner.set_gravity(Vec2::new(gravity_x, gravity_y));
        Self { inner }
    }

    /// Update gravity.
    pub fn setGravity(&mut self, gx: f32, gy: f32) {
        self.inner.set_gravity(Vec2::new(gx, gy));
    }

    /// Add a circle body. Returns its id.
    pub fn addCircle(&mut self, x: f32, y: f32, radius: f32) -> usize {
        let body = Body2D::new(0, Shape2D::Circle(Circle::new(Vec2::ZERO, radius)))
            .with_position(Vec2::new(x, y))
            .with_density(1.0);
        self.inner.add_body(body)
    }

    /// Add an axis-aligned box body. Returns its id.
    pub fn addBox(&mut self, x: f32, y: f32, width: f32, height: f32) -> usize {
        let body = Body2D::new(
            0,
            Shape2D::Aabb(Aabb::from_center_size(Vec2::ZERO, Vec2::new(width, height))),
        )
        .with_position(Vec2::new(x, y))
        .with_density(1.0);
        self.inner.add_body(body)
    }

    /// Set body kind. Pass one of `BodyKind.Dynamic`, `BodyKind.Kinematic`, `BodyKind.Static`.
    pub fn setBodyKind(&mut self, id: usize, kind: BodyKind) {
        if let Some(body) = self.inner.body_mut(id) {
            body.set_body_type(kind.into());
        }
    }

    /// Set body velocity.
    pub fn setBodyVelocity(&mut self, id: usize, vx: f32, vy: f32) {
        if let Some(body) = self.inner.body_mut(id) {
            body.velocity = Vec2::new(vx, vy);
        }
    }

    /// Set body restitution (`[0, 1]`).
    pub fn setBodyRestitution(&mut self, id: usize, restitution: f32) {
        if let Some(body) = self.inner.body_mut(id) {
            body.restitution = restitution;
        }
    }

    /// Set body friction.
    pub fn setBodyFriction(&mut self, id: usize, friction: f32) {
        if let Some(body) = self.inner.body_mut(id) {
            body.friction = friction;
        }
    }

    /// Step the simulation by `dt` seconds.
    pub fn step(&mut self, dt: f32) {
        self.inner.step(dt);
    }

    /// Body position as `[x, y]`. Returns an empty array if the id is invalid.
    pub fn bodyPosition(&self, id: usize) -> js_sys::Float32Array {
        let buf = self
            .inner
            .body(id)
            .map(|b| [b.position.x, b.position.y])
            .unwrap_or([0.0, 0.0]);
        copy_f32(&buf)
    }

    /// Body velocity as `[vx, vy]`.
    pub fn bodyVelocity(&self, id: usize) -> js_sys::Float32Array {
        let buf = self
            .inner
            .body(id)
            .map(|b| [b.velocity.x, b.velocity.y])
            .unwrap_or([0.0, 0.0]);
        copy_f32(&buf)
    }

    /// Body rotation in radians, or `0.0` if the id is invalid.
    pub fn bodyRotation(&self, id: usize) -> f32 {
        self.inner.body(id).map_or(0.0, |b| b.rotation)
    }

    /// Disable a body (skipped by the solver until re-enabled).
    pub fn disableBody(&mut self, id: usize) {
        self.inner.disable_body(id);
    }

    /// Re-enable a previously disabled body.
    pub fn enableBody(&mut self, id: usize) {
        self.inner.enable_body(id);
    }

    /// Number of bodies in the world.
    pub fn bodyCount(&self) -> usize {
        self.inner.bodies().len()
    }

    /// All body positions packed `[x0, y0, x1, y1, …]`. Lets a JS renderer
    /// iterate every body's position with a single bridge call.
    pub fn allPositions(&self) -> js_sys::Float32Array {
        let bodies = self.inner.bodies();
        let mut out = Vec::with_capacity(bodies.len() * 2);
        for b in bodies {
            out.push(b.position.x);
            out.push(b.position.y);
        }
        copy_f32(&out)
    }

    /// Serialize the world state into a `Uint8Array`.
    ///
    /// Bit-stable across runs of the same compiled binary. Save the bytes to
    /// disk for replays, or pass them straight into [`World2D::restoreFrom`]
    /// to rewind the world.
    pub fn snapshot(&self) -> js_sys::Uint8Array {
        copy_u8(self.inner.snapshot().as_bytes())
    }

    /// Restore the world to the state captured by `bytes`. Throws on a
    /// malformed buffer.
    pub fn restoreFrom(&mut self, bytes: &[u8]) -> Result<(), JsValue> {
        let snap = Snapshot2D::from_bytes(bytes.to_vec());
        self.inner
            .restore_from(&snap)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

// ───────────────────────────────────────────────────────────────────────────
// 3D
// ───────────────────────────────────────────────────────────────────────────

/// 3D physics world. Right-handed, Y up. Same shape as [`World2D`] but with
/// three-component vectors and quaternion rotation.
#[wasm_bindgen]
pub struct World3D {
    inner: World3DInner,
}

#[wasm_bindgen]
impl World3D {
    /// Construct a world with the given linear gravity.
    #[wasm_bindgen(constructor)]
    pub fn new(gx: f32, gy: f32, gz: f32) -> World3D {
        let mut inner = World3DInner::new();
        inner.set_gravity(Vec3::new(gx, gy, gz));
        Self { inner }
    }

    /// Update gravity.
    pub fn setGravity(&mut self, gx: f32, gy: f32, gz: f32) {
        self.inner.set_gravity(Vec3::new(gx, gy, gz));
    }

    /// Add a sphere body. Returns its id.
    pub fn addSphere(&mut self, x: f32, y: f32, z: f32, radius: f32) -> usize {
        let body = Body3D::new(0, Shape3D::Sphere(Sphere::new(Vec3::ZERO, radius)))
            .with_position(Vec3::new(x, y, z))
            .with_density(1.0);
        self.inner.add_body(body)
    }

    /// Add an axis-aligned box body. Returns its id.
    pub fn addBox(&mut self, x: f32, y: f32, z: f32, w: f32, h: f32, d: f32) -> usize {
        let body = Body3D::new(
            0,
            Shape3D::Aabb(Aabb3::from_center_size(Vec3::ZERO, Vec3::new(w, h, d))),
        )
        .with_position(Vec3::new(x, y, z))
        .with_density(1.0);
        self.inner.add_body(body)
    }

    /// Set body kind. Pass one of `BodyKind.Dynamic`, `BodyKind.Kinematic`, `BodyKind.Static`.
    pub fn setBodyKind(&mut self, id: usize, kind: BodyKind) {
        if let Some(body) = self.inner.body_mut(id) {
            body.set_body_type(kind.into());
        }
    }

    /// Set body velocity.
    pub fn setBodyVelocity(&mut self, id: usize, vx: f32, vy: f32, vz: f32) {
        if let Some(body) = self.inner.body_mut(id) {
            body.velocity = Vec3::new(vx, vy, vz);
        }
    }

    /// Set body restitution.
    pub fn setBodyRestitution(&mut self, id: usize, restitution: f32) {
        if let Some(body) = self.inner.body_mut(id) {
            body.restitution = restitution;
        }
    }

    /// Set body friction.
    pub fn setBodyFriction(&mut self, id: usize, friction: f32) {
        if let Some(body) = self.inner.body_mut(id) {
            body.friction = friction;
        }
    }

    /// Step the simulation by `dt` seconds.
    pub fn step(&mut self, dt: f32) {
        self.inner.step(dt);
    }

    /// Body position as `[x, y, z]`.
    pub fn bodyPosition(&self, id: usize) -> js_sys::Float32Array {
        let buf = self
            .inner
            .body(id)
            .map(|b| [b.position.x, b.position.y, b.position.z])
            .unwrap_or([0.0, 0.0, 0.0]);
        copy_f32(&buf)
    }

    /// Body velocity as `[vx, vy, vz]`.
    pub fn bodyVelocity(&self, id: usize) -> js_sys::Float32Array {
        let buf = self
            .inner
            .body(id)
            .map(|b| [b.velocity.x, b.velocity.y, b.velocity.z])
            .unwrap_or([0.0, 0.0, 0.0]);
        copy_f32(&buf)
    }

    /// Body orientation as a quaternion `[qx, qy, qz, qw]`.
    pub fn bodyRotation(&self, id: usize) -> js_sys::Float32Array {
        let buf = self
            .inner
            .body(id)
            .map(|b| {
                let q = b.rotation;
                [q.x, q.y, q.z, q.w]
            })
            .unwrap_or([0.0, 0.0, 0.0, 1.0]);
        copy_f32(&buf)
    }

    /// Disable a body.
    pub fn disableBody(&mut self, id: usize) {
        self.inner.disable_body(id);
    }

    /// Re-enable a disabled body.
    pub fn enableBody(&mut self, id: usize) {
        self.inner.enable_body(id);
    }

    /// Number of bodies.
    pub fn bodyCount(&self) -> usize {
        self.inner.bodies().len()
    }

    /// All body positions packed `[x0, y0, z0, x1, y1, z1, …]`.
    pub fn allPositions(&self) -> js_sys::Float32Array {
        let bodies = self.inner.bodies();
        let mut out = Vec::with_capacity(bodies.len() * 3);
        for b in bodies {
            out.push(b.position.x);
            out.push(b.position.y);
            out.push(b.position.z);
        }
        copy_f32(&out)
    }

    /// Serialize the world state into a `Uint8Array`.
    ///
    /// Bit-stable across runs of the same compiled binary.
    pub fn snapshot(&self) -> js_sys::Uint8Array {
        copy_u8(self.inner.snapshot().as_bytes())
    }

    /// Restore the world to the state captured by `bytes`. Throws on a
    /// malformed buffer.
    pub fn restoreFrom(&mut self, bytes: &[u8]) -> Result<(), JsValue> {
        let snap = Snapshot3D::from_bytes(bytes.to_vec());
        self.inner
            .restore_from(&snap)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

fn copy_f32(slice: &[f32]) -> js_sys::Float32Array {
    let arr = js_sys::Float32Array::new_with_length(slice.len() as u32);
    arr.copy_from(slice);
    arr
}

fn copy_u8(slice: &[u8]) -> js_sys::Uint8Array {
    let arr = js_sys::Uint8Array::new_with_length(slice.len() as u32);
    arr.copy_from(slice);
    arr
}
