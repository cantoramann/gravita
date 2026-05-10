# Architecture

This is the system map. For "how do I use Gravita?" see [README.md](README.md). For "I'm an AI agent, what's the style guide?" see [CLAUDE.md](CLAUDE.md).

---

## Crate graph

```text
                    ┌─────────────────────────┐
                    │   gravita (umbrella)    │  ← user-facing, feature flags
                    └────────────┬────────────┘
                                 │
        ┌────────────────────────┼────────────────────────────────┐
        │            │           │            │            │
        ▼            ▼           ▼            ▼            ▼
   gravita-       gravita-   gravita-     gravita-      gravita-
   physics        physics-3d collections  renderer-3d   renderer
        │            │           │            │            │
        │            │           │            │            │
        └────────┬───┴────────┬──┴────────┬───┴──────┬─────┘
                 │            │           │          │
                 └────────────┼───────────┘          │
                              │                      │
                              ▼                      │
                       ┌─────────────┐               │
                       │ gravita-math│ ◀─────────────┘
                       └─────────────┘
                       (no deps — pure Rust)
```

Examples depend on the engine crates directly:

```text
examples/bouncing-balls  →  gravita-math + gravita-physics + gravita-renderer + gravita-example-shim
examples/spheres-3d      →  gravita-math + gravita-physics-3d + gravita-renderer-3d
```

---

## Frame pipeline (2D)

```text
┌────────────────────────────────────────────────────────────────┐
│  gravita-example-shim::run(WindowConfig, app)                  │
│  ──────────────────────────────────────                        │
│  • winit::ApplicationHandler with ControlFlow::WaitUntil       │
│  • Pixels framebuffer (via pixels crate)                       │
│  • Fixed-timestep accumulator                                  │
└────────────┬───────────────────────────────────────────────────┘
             │
             │  on each frame: dt, &Input → user's App
             ▼
┌────────────────────────────────────────────────────────────────┐
│  App::update(dt, input)                                        │
│  ──────────────────────                                        │
│  for each sub-step (dt = 1/60s):                               │
│    physics_world.step(dt)                                      │
│      ├── apply_gravity_and_damping  (single dynamic pass)      │
│      ├── integrator.integrate_velocity for each body           │
│      ├── SimpleCollisionDetector or SpatialHashDetector        │
│      │     ├── broad phase: AABB cull + cell hash              │
│      │     └── narrow phase: Circle/Aabb pair tests            │
│      ├── solve_velocity_constraint × velocity_iterations       │
│      │     (normal impulse + Coulomb friction tangent impulse) │
│      ├── integrator.integrate_position                         │
│      ├── solve_position_constraint × position_iterations       │
│      └── apply_sleeping (snap micro-velocities to zero)        │
└────────────┬───────────────────────────────────────────────────┘
             │
             │  framebuffer: &mut [u8] of RGBA pixels
             ▼
┌────────────────────────────────────────────────────────────────┐
│  App::render(frame)                                            │
│  ──────────────────                                            │
│  gravita_renderer::{clear, draw_circle, draw_line, …}          │
│    ├── all primitives funnel through frame::put_pixel /        │
│    │    frame::blend_pixel (one bounds-checked write helper)   │
│    └── coordinates are screen-space (Y-down). Examples flip Y. │
└────────────────────────────────────────────────────────────────┘
```

Files to read in order:

1. [`crates/example-shim/src/lib.rs`](crates/example-shim/src/lib.rs) — runner + `App` trait
2. [`crates/physics/src/world.rs`](crates/physics/src/world.rs) — `PhysicsWorld::step`
3. [`crates/physics/src/collision/detector.rs`](crates/physics/src/collision/detector.rs) — broad → narrow
4. [`crates/renderer/src/lib.rs`](crates/renderer/src/lib.rs) — module re-exports + design rationale

---

## Frame pipeline (3D)

```text
┌────────────────────────────────────────────────────────────────┐
│  gravita_renderer_3d::run(WindowConfig, app)                   │
│  ────────────────────────────────────                          │
│  • winit::ApplicationHandler                                   │
│  • Inside resumed():                                           │
│      Box::leak(Window) → &'static Window                       │
│      pollster::block_on(wgpu init) → Renderer3D                │
│  • Fixed-timestep accumulator drives App3D::update             │
└────────────┬───────────────────────────────────────────────────┘
             │
             ▼
┌────────────────────────────────────────────────────────────────┐
│  App3D::update(dt, input)                                      │
│  ────────────────────────                                      │
│  for each sub-step:                                            │
│    physics_world_3d.step(dt)                                   │
│      ├── apply_gravity_and_damping                             │
│      ├── integrator.integrate_velocity (Vec3 force/torque)     │
│      ├── SimpleCollisionDetector or SpatialHashDetector        │
│      │     ├── broad phase: world Aabb3 hash (3D cell key)     │
│      │     └── narrow phase: dispatch by shape pair            │
│      │           • sphere-sphere                                │
│      │           • sphere-Aabb3                                 │
│      │           • Aabb3-Aabb3 (axis of min penetration)        │
│      │           • sphere-Obb (closest-point in OBB local)      │
│      │           • Obb-Obb (15-axis SAT)                        │
│      │           • Aabb3-Obb promoted to Obb-Obb                │
│      ├── solve_velocity (normal + friction; at-point velocity) │
│      ├── integrator.integrate_position                         │
│      │     ├── pos += vel · dt                                  │
│      │     └── rot' = normalize(rot + 0.5 · ω · rot · dt)       │
│      ├── solve_position (penetration correction)               │
│      └── apply_sleeping                                        │
└────────────┬───────────────────────────────────────────────────┘
             │
             ▼
┌────────────────────────────────────────────────────────────────┐
│  App3D::render(&mut Renderer3D)                                │
│  ──────────────────────────────                                │
│  build Vec<Instance> { mesh, transform, tint }                 │
│  renderer.render(camera, &instances)                           │
│    ├── upload globals (view·proj, light, ambient)              │
│    ├── group instances by mesh → one draw_indexed per mesh     │
│    └── instance buffer is rebuilt per frame (allocated once    │
│         per non-empty mesh — fine for demo scale, lift to a    │
│         ring buffer for production scale)                      │
└────────────────────────────────────────────────────────────────┘
```

WGSL shader is at [`crates/renderer-3d/src/shader.wgsl`](crates/renderer-3d/src/shader.wgsl). It runs a per-vertex Lambert against the directional light and multiplies by the per-instance tint.

Files to read in order:

1. [`crates/renderer-3d/src/runner.rs`](crates/renderer-3d/src/runner.rs) — runner + `App3D` trait
2. [`crates/physics-3d/src/world.rs`](crates/physics-3d/src/world.rs) — step pipeline (especially `solve_velocity` for friction math)
3. [`crates/physics-3d/src/collision.rs`](crates/physics-3d/src/collision.rs) — SAT for OBB
4. [`crates/renderer-3d/src/renderer.rs`](crates/renderer-3d/src/renderer.rs) — wgpu setup + draw call

---

## Why 2D and 3D are parallel crates

We considered a const-generic `Vector<const D: usize>` and a single physics crate parameterised by `D`. Decided against because:

- **Ergonomics matter.** `v.x` / `v.y` / `v.z` as fields read more naturally than `v[0]`. Const-generic structs can't have named fields.
- **Specialised methods diverge.** `Vec2::perpendicular()`, `Vec2::rotate(angle)`, `Vec2::angle()` are 2D-only. `Vec3::cross()`, `Quat`, OBB SAT are 3D-only. Trying to share via traits with associated types becomes more confusing than just having two types.
- **Inertia tensor differs.** 2D inertia is a scalar; 3D is a `Vec3` (diagonal, with a real 3×3 tensor as a future possibility).
- **Each pipeline has a stable surface.** Existing 2D users don't pay for 3D's wgpu/winit deps. 3D users don't see legacy 2D-only conveniences.

The shared layer is the [`Vector`](crates/math/src/vector.rs) trait — `dot`, `length`, `normalize`, `distance`, `lerp`, `reflect`. Dim-agnostic algorithms are written against it:

```rust
fn move_toward<V: Vector>(p: V, target: V, max_step: f32) -> V { … }
```

---

## Solver design notes

### 2D and 3D both use semi-implicit (symplectic) Euler

```text
acceleration = force / mass     // per body, per step
velocity  += acceleration · dt
position  += velocity · dt
```

It's not the most accurate integrator (no error bound like RK4), but it preserves energy reasonably well for games and is dead simple. The 2D crate also offers `Verlet` as a pluggable alternative.

### Quaternion integration (3D only)

Discrete update from continuous `dq/dt = (1/2) · (0, ω) · q`:

```text
q' = normalize(q + 0.5 · (ω_vec_quat · q) · dt)
```

The `normalize` step is mandatory — without it, the quaternion drifts off the unit sphere over thousands of steps. A test (`dynamic_body_quaternion_stays_unit_after_long_run`) runs 10 000 frames and asserts `|q| ≈ 1`.

### Contact resolution

Both 2D and 3D run the same shape, iterated:

```text
for i in 0..velocity_iterations:               // 8 by default
    for each contact:
        compute relative velocity at contact point (includes ω × r)
        compute normal impulse jn with restitution
        apply jn to both bodies (linear + angular via apply_impulse_at_point)
        // 3D only: also Coulomb friction
        compute tangent direction from post-normal relative velocity
        compute jt clamped to ±μ · |jn|
        apply jt
```

Then once per step, `solve_position_constraint` does projected position correction (Baumgarte-style with a "slop" tolerance to avoid jitter on resting stacks).

### Broad phase

Both pipelines use a flat sorted-`Vec` spatial hash:

- **Storage**: `Vec<(packed_cell_key, body_idx)>`. Sort once per `update`.
- **2D cell key**: `u64` = (i32 x biased) << 32 | (i32 y biased).
- **3D cell key**: `u64` = three 21-bit biased coords packed (so ±1M cell range, plenty).
- **Pair generation**: walk runs of equal cell keys, emit pairs, sort + dedup.
- **Query**: binary-search by packed key, dedup the appended slice.

No `HashMap` or `HashSet` is touched in the per-step hot path. The pair-dedup scratch buffer is owned by the grid and survives across frames.

---

## Renderer design notes

### 2D (`gravita-renderer`)

Pure CPU. Frame buffer is an `&mut [u8]` of RGBA pixels. Every primitive funnels through one of two helpers in [`crates/renderer/src/frame.rs`](crates/renderer/src/frame.rs):

- `put_pixel(frame, x, y, w, h, color)` — bounds-checked opaque write.
- `blend_pixel(frame, x, y, w, h, color)` — bounds-checked alpha blend (premul-style over).

That's the only place pixel-index arithmetic lives. Adding a new primitive means writing the iteration pattern and calling these helpers — no new bounds-check code path to audit.

Modules: `color` (palette + `rgb`/`rgba` helpers), `frame` (the two helpers above), `text` (5×7 bitmap font), `primitives::{clear, circle, line, axes, rect}`.

### 3D (`gravita-renderer-3d`)

Built on `wgpu` 0.19. Pipeline state is fully owned by `Renderer3D`. One pipeline, one bind group, one render pass per frame.

- Vertex layout: `position: vec3 + color: vec3 + normal: vec3`.
- Instance layout: 4 vec4 rows of the model matrix + RGBA tint.
- Globals uniform: view-projection mat4 + light dir + ambient.
- Depth: `Depth32Float`, `Less`, write-enabled.
- Culling: back face, CCW winding.
- Surface format: prefer sRGB, fallback to whatever the adapter offers.

The wgpu init is async; we call `pollster::block_on(adapter)` and `pollster::block_on(device)` inside `Renderer3D::new`. The runner stores `Pixels<'static>`-style — `Box::leak(Window)` and `Pixels<'static>` references the leaked window for the life of the app (single-window apps only).

---

## Known constraints

- **Rust nightly is required.** The workspace uses `edition = "2024"` (recently stabilised) plus some nightly clippy lints. The `rust-toolchain.toml` pins the version.
- **`wgpu` 0.19 specifically.** Newer versions changed `RenderPipelineDescriptor` (added `cache` and `compilation_options` fields). Don't bump without updating the pipeline definition.
- **WASM target works only for the dedicated WASM examples** (`froggy-jump`). The other examples assume desktop winit.
- **Inertia in 3D is diagonal.** Sphere and AABB have diagonal tensors by symmetry; for arbitrary OBB shapes, a real 3×3 tensor would be more accurate. Open issue, see Roadmap.

---

## File-size reference (for context)

These are the largest crates by source LOC (excluding tests):

```text
crates/physics/             ~3500 LOC
crates/physics-3d/          ~1300 LOC
crates/renderer-3d/         ~ 700 LOC
crates/renderer/            ~ 600 LOC (split across primitives modules)
crates/math/                ~2400 LOC (mostly Vec2/Vec3/Quat with tests inline)
crates/collections/         ~ 800 LOC
crates/example-shim/        ~ 250 LOC
```

Test count by crate (`cargo test --workspace`): math 246, physics 100+, physics-3d 42, renderer 38, collections ~40, others minor. Workspace total ≈ 513 passing.
