# CLAUDE.md — Onboarding for AI coding assistants

This file is the canonical reference for AI agents (Claude Code, Cursor, Aider, etc.) working in this repo. It's terse on purpose — the goal is to fit the whole picture into a single read.

If you're a human reading this: it doubles as a "where does what live" cheat sheet. Start with [README.md](README.md) and [ARCHITECTURE.md](ARCHITECTURE.md) for the conceptual overview, then come back here for conventions.

---

## Workspace shape

11 library crates + 8 examples. Two parallel pipelines (2D and 3D) sit on a shared `gravita-math` crate.

```text
crates/
├── math/              # PRIMARY DEP — Vec2/Vec3/Quat/Aabb/Aabb3/Sphere/Obb/Ray/Transform.
├── physics/           # 2D rigid body sim. depends on math.
├── physics-3d/        # 3D rigid body sim. depends on math.
├── renderer/          # CPU 2D framebuffer rasterizer. depends on math.
├── renderer-3d/       # wgpu 3D renderer + winit runner. depends on math.
├── collections/       # 2D Stickman/Spaceship/Planet. depends on math + renderer.
├── example-shim/      # Internal: winit/pixels glue for 2D examples.
├── engine-core/       # Placeholder (~30 LOC of doc comments).
├── input/             # Placeholder.
├── assets/            # Placeholder.
└── gravita/           # Umbrella with feature flags + prelude.

examples/
├── bouncing-balls/    # 2D physics
├── gravity-arena/     # 2D orbits
├── rotating-rod/      # 2D pendulum (no physics crate — custom math)
├── stickman-walk/     # 2D animated character
├── tetris/            # 2D classic
├── froggy-jump/       # 2D + WASM
├── cube-3d/           # 3D smoke test for renderer
└── spheres-3d/        # 3D physics-3d + renderer-3d together
```

Placeholder crates (`engine-core`, `input`, `assets`) are intentional — they reserve names + ship as `publish = false` until real implementations land. **Do not delete them.**

---

## How to verify changes

Run these from the repo root. Everything must stay green.

```bash
cargo build --workspace --all-targets
cargo test --workspace            # current baseline: 513 passing, 0 failing, 7 ignored
cargo clippy --workspace --all-targets
cargo fmt --all -- --check
cargo doc --workspace --no-deps   # must not introduce broken intra-doc links
```

**One known pre-existing warning**: `tetris/src/main.rs` has a `draw_cell` with 8 args. That predates this work and is not in scope for ordinary changes.

For perf-sensitive changes touching `physics`, `physics-3d`, or `math`, run the relevant `cargo bench -p <crate>` before and after.

---

## Coding conventions

### Code style

- **`mul_add` preferred** for chained `a*b + c*d + …` in hot paths. The workspace enables `clippy::suboptimal_flops = "warn"`.
- **No emojis** in code or doc comments (the user has not opted in).
- **Field access stays public** for ergonomics (`v.x`, `v.y`, `v.z`, `q.w`). Use `pub(crate)` only when a field has a maintained invariant (e.g. `RigidBody.inv_mass` mirrors `mass`).
- **Comments explain WHY, not WHAT.** Well-named identifiers are the WHAT. If you'd remove a comment without confusing a future reader, remove it.
- **Errors with `#[allow]`** must include a one-line reason after the lint name. Example:
  ```rust
  #[allow(clippy::too_many_arguments)] // text rendering needs all of these; refactor when adding TextStyle.
  ```

### Naming

- Acronyms: **`Aabb`, not `AABB`** (per RFC 430 and `clippy::upper_case_acronyms`). Already applied workspace-wide.
- `with_*` for chainable builders, `set_*` for in-place mutators. Both `#[must_use]` on `with_*`.
- `*_3D` suffix in prelude re-exports to disambiguate from 2D peers when both are imported (`RigidBody3D`, `PhysicsWorld3D`).

### Encapsulation patterns

`RigidBody` (both 2D and 3D) uses the following pattern:

- **`pub`** — user-mutable state: `position`, `velocity`, `rotation`, `angular_velocity`, restitution/friction/damping, `shape`, `is_sensor`.
- **`pub(crate)`** — solver-managed state: `inv_mass`, `inv_inertia`, `force_accumulator`, `torque_accumulator`, `acceleration`, `angular_acceleration`.
- **`pub(crate)` + setter** — invariant-protected: `mass`, `inertia`, `body_type`, `fixed_rotation`. The setter (`set_mass`, `set_body_type`, …) calls `update_mass_properties()` so derived `inv_*` fields stay in sync.

When adding new fields, classify them into one of these three categories before deciding visibility.

### Math conventions

- 2D world space: **Y is up**. Screen space (renderer): Y is down. The 2D examples explicitly flip Y when drawing.
- 3D world space: **right-handed, Y is up, forward is -Z** (`Vec3::FORWARD = -Z`).
- `Vec2::length()` and `Vec3::length()` use `length_squared().sqrt()` (not `hypot()` — faster at game scale).
- Quaternion convention: `(x, y, z, w)` with `w` scalar. Hamilton product: `self * other` applies `other` then `self`.
- All collision normals point **from `body_a` toward `body_b`**.

---

## Performance gotchas

1. **Hot loops should not allocate.** The 2D and 3D spatial hashes own reusable scratch buffers; force generators take `&mut [RigidBody]` slices and use `std::slice::from_mut` for single-body iteration; the spatial hash's `get_potential_pairs` and `query` both take `&mut Vec` output buffers.
2. **`length_squared` first** for early-outs. The pattern is `if delta.length_squared() > radius*radius { skip }` rather than `if delta.length() > radius { skip }`.
3. **Verlet history** is indexed by `body.id` directly (a `Vec<Option<…>>`, lazy-grown). Don't accidentally re-introduce a linear scan.
4. **`SAT for OBB-OBB`** has 15 axes. The cross-product axes need overlap normalisation by the cross magnitude (already implemented). If you touch [`crates/physics-3d/src/collision.rs:test_obb_obb`](crates/physics-3d/src/collision.rs), re-run `cargo bench -p gravita-physics-3d`.

---

## Where things live (file-level map)

| Topic | File |
|---|---|
| Vec2/Vec3 algebra | [`crates/math/src/vector2.rs`](crates/math/src/vector2.rs), [`crates/math/src/vector3.rs`](crates/math/src/vector3.rs) |
| `Vector` trait (dim-agnostic) | [`crates/math/src/vector.rs`](crates/math/src/vector.rs) |
| Quaternion | [`crates/math/src/quat.rs`](crates/math/src/quat.rs) |
| AABB / Sphere / OBB | [`crates/math/src/aabb.rs`](crates/math/src/aabb.rs), [`aabb3.rs`](crates/math/src/aabb3.rs), [`sphere.rs`](crates/math/src/sphere.rs), [`obb.rs`](crates/math/src/obb.rs) |
| 2D rigid body | [`crates/physics/src/body.rs`](crates/physics/src/body.rs) |
| 3D rigid body | [`crates/physics-3d/src/body.rs`](crates/physics-3d/src/body.rs) |
| 2D solver loop | [`crates/physics/src/world.rs`](crates/physics/src/world.rs) |
| 3D solver loop (with friction) | [`crates/physics-3d/src/world.rs`](crates/physics-3d/src/world.rs) |
| 2D narrow phase | [`crates/physics/src/collision/narrow_phase.rs`](crates/physics/src/collision/narrow_phase.rs) |
| 3D narrow phase + OBB SAT | [`crates/physics-3d/src/collision.rs`](crates/physics-3d/src/collision.rs) |
| 2D broad phase (spatial hash) | [`crates/physics/src/collision/broad_phase.rs`](crates/physics/src/collision/broad_phase.rs) |
| 3D broad phase (spatial hash) | [`crates/physics-3d/src/broad_phase.rs`](crates/physics-3d/src/broad_phase.rs) |
| Integrators (2D) | [`crates/physics/src/integrator.rs`](crates/physics/src/integrator.rs) |
| Integrator (3D, with quat update) | [`crates/physics-3d/src/integrator.rs`](crates/physics-3d/src/integrator.rs) |
| Force generators (2D) | [`crates/physics/src/forces.rs`](crates/physics/src/forces.rs) |
| CPU draw primitives | [`crates/renderer/src/primitives/`](crates/renderer/src/primitives/) |
| Bitmap font (2D) | [`crates/renderer/src/text.rs`](crates/renderer/src/text.rs) |
| wgpu pipeline | [`crates/renderer-3d/src/renderer.rs`](crates/renderer-3d/src/renderer.rs) |
| WGSL shader | [`crates/renderer-3d/src/shader.wgsl`](crates/renderer-3d/src/shader.wgsl) |
| 2D example runner (`App` trait) | [`crates/example-shim/src/lib.rs`](crates/example-shim/src/lib.rs) |
| 3D example runner (`App3D` trait) | [`crates/renderer-3d/src/runner.rs`](crates/renderer-3d/src/runner.rs) |

---

## Common task recipes

### Add a new collision shape

1. Add the variant to `CollisionShape` in both [`crates/physics/src/body.rs`](crates/physics/src/body.rs) (2D) and/or [`crates/physics-3d/src/body.rs`](crates/physics-3d/src/body.rs) (3D).
2. Update `CollisionShape::world_aabb`, `is_valid`, `mass`, `inertia` for the new variant.
3. Add narrow-phase pair tests for every existing-shape × new-shape combo in `collision.rs` (3D) or `collision/narrow_phase.rs` (2D).
4. Wire the new variants into `test_pair` dispatch.
5. Update the renderer / example match arms (3D examples have a `match &body.shape { … }` to scale meshes).

### Add a new force generator (2D)

1. Define a struct with public tunable fields in [`crates/physics/src/forces.rs`](crates/physics/src/forces.rs).
2. Implement `ForceGenerator::apply`. Use `apply_to_dynamic(bodies, |body| { … })` to skip non-Dynamic bodies.
3. Use `length_squared` early-outs if the force has a radius cutoff.
4. Add tests in the `#[cfg(test)] mod tests` at the bottom of the same file.

### Change something in the wgpu pipeline

- The shader is in [`crates/renderer-3d/src/shader.wgsl`](crates/renderer-3d/src/shader.wgsl). Vertex inputs at locations 0–2; instance inputs at 3–7.
- New uniform fields go in `GlobalsRaw` ([`vertex.rs`](crates/renderer-3d/src/vertex.rs)) and the matching `struct Globals` in `shader.wgsl`. **Layout must match exactly** — keep padding fields explicit.
- New vertex attributes need updates to: `Vertex::layout()`, the shader `VertexIn`, and (typically) `Mesh` constructors.

### Add an example

1. Create `examples/<name>/Cargo.toml`. Use `edition.workspace = true`. Deps:
   - 2D: `gravita-math`, `gravita-physics`, `gravita-renderer`, `gravita-example-shim`.
   - 3D: `gravita-math`, `gravita-physics-3d`, `gravita-renderer-3d`.
2. Implement `gravita_example_shim::App` (2D) or `gravita_renderer_3d::App3D` (3D).
3. Add `examples/<name>` to the workspace `members` in the top-level `Cargo.toml`.

---

## Commit style

Match what's already in `git log`:

- One-line summary, lowercase verb start, ≤72 chars.
- Body paragraph(s) explaining **why** when the diff doesn't already say it.
- Bulleted list of concrete sub-changes when the commit is big.
- Footer: `Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>` when the AI assisted.

Recent examples (look at `git log --oneline -10`):

```
3d follow-up: OBB+SAT, friction, 3d spatial hash, tetris blend, umbrella+benches
physics-3d: rigid body sim with Quat rotation + Sphere/Aabb3 collision
renderer-3d: wgpu pipeline + ApplicationHandler runner + cube-3d demo
math: add 3D foundation — Vec3, Quat, Aabb3, Sphere, Ray3D, Transform3D, Vector trait
gravita v0.2 — workspace simplification + breaking API cleanup
```

The user authorises direct pushes to `main`. There's no PR flow for solo work. Don't push without explicit approval in the conversation.

---

## Things explicitly OUT of scope

Don't do these unless the user asks:

- Rename `Aabb` back to `AABB`. That rename is locked in.
- Switch from parallel `Vec2`/`Vec3` types to a single `Vector<const D: usize>`. The ergonomics trade-off was decided against.
- Replace `pixels` (2D) with `wgpu`. The CPU renderer is a deliberate debug fallback.
- Add a polished example launcher / GUI. Examples are CLI-runnable on purpose.
- Bump `wgpu` past 0.19 right now — the API changed in 0.20+ and the pipeline descriptor needs the `cache` / `compilation_options` fields.

---

## Quick reference: tests, benches, lints

| Want | Run |
|---|---|
| All tests | `cargo test --workspace` |
| Just math | `cargo test -p gravita-math` |
| Just 3D physics | `cargo test -p gravita-physics-3d` |
| Math benches | `cargo bench -p gravita-math` |
| 2D physics benches | `cargo bench -p gravita-physics` |
| 3D physics benches | `cargo bench -p gravita-physics-3d` |
| Workspace lints | `cargo clippy --workspace --all-targets` |
| Format check | `cargo fmt --all -- --check` |
| Build 2D example | `cargo run -p bouncing-balls` |
| Build 3D example | `cargo run -p spheres-3d` |
| WASM build | `cargo build --target wasm32-unknown-unknown -p froggy-jump` |
