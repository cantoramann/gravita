//! GPU-accelerated 3D renderer for the Gravita engine.
//!
//! Internal crate (`publish = false`). Pairs a `wgpu` pipeline with a winit
//! `ApplicationHandler` runner so 3D examples have a single dep, like
//! [`gravita-example-shim`](../gravita_example_shim/) for 2D examples.
//!
//! # Pipeline
//!
//! - Right-handed Y-up coordinate system.
//! - Indexed triangle meshes with per-vertex `(position, color, normal)`.
//! - Instanced rendering: one mesh, many transforms.
//! - Single global uniform: view-projection matrix, directional light, ambient.
//! - Depth buffer is `Depth32Float`, comparison `Less`, write enabled.
//! - Back-face culling, CCW winding.
//!
//! # Example
//!
//! ```ignore
//! use gravita_math::{Transform3D, Vec3};
//! use gravita_renderer_3d::{App3D, Camera, Input, Instance, Mesh, MeshHandle,
//!     Renderer3D, WindowConfig, run};
//!
//! struct Demo {
//!     cube: Option<MeshHandle>,
//!     t: f32,
//! }
//!
//! impl App3D for Demo {
//!     fn setup(&mut self, r: &mut Renderer3D) {
//!         self.cube = Some(r.register_mesh("cube", &Mesh::cube()));
//!     }
//!     fn update(&mut self, dt: f32, _input: &Input) {
//!         self.t += dt;
//!     }
//!     fn render(&self, _r: &mut Renderer3D) {
//!         // Build a list of Instance values and render them. See examples/cube-3d.
//!     }
//! }
//!
//! run(WindowConfig::default(), Demo { cube: None, t: 0.0 }).unwrap();
//! ```

#![warn(missing_docs)]

/// Camera (view + projection).
pub mod camera;
/// CPU + GPU mesh types and built-in primitives (cube, plane, sphere).
pub mod mesh;
/// `wgpu` pipeline and frame rendering.
pub mod renderer;
/// Window + event-loop runner with an `App3D` trait.
pub mod runner;
/// Vertex and instance buffer layouts.
pub mod vertex;

pub use camera::Camera;
pub use mesh::{Mesh, MeshBuffer};
pub use renderer::{Instance, MeshHandle, Renderer3D};
pub use runner::{App3D, Input, ShimKeyCode, ShimMouseButton, WindowConfig, run};
pub use vertex::{GlobalsRaw, InstanceRaw, Vertex};
