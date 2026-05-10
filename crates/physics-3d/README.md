# gravita-physics-3d

3D rigid-body physics. Parallel to [`gravita-physics`](../physics) — same architecture, same builder API, same solver shape, but operates on `Vec3` / `Quat` / `Aabb3` / `Sphere` / `Obb`.

## What it does

- 3D rigid bodies with quaternion rotation and diagonal inertia tensors
- Collision: sphere-sphere, sphere-AABB, AABB-AABB, sphere-OBB, OBB-OBB (15-axis SAT)
- Contact resolution with restitution + Coulomb friction (in-cone tangential impulse)
- `SemiImplicitEuler` integrator with stable quaternion update + drift renormalisation
- Broad phase: `SimpleCollisionDetector` (O(N²)) or `SpatialHashDetector` (flat sorted Vec, 3D cell hash)

## Quick example

```rust
use gravita_math::{Sphere, Vec3};
use gravita_physics_3d::{BodyType, CollisionShape, PhysicsWorld, RigidBody};

let mut world = PhysicsWorld::new();
world.set_gravity(Vec3::new(0.0, -9.81, 0.0));

// Drop a sphere from 5 m up.
world.add_body(
    RigidBody::new(0, CollisionShape::Sphere(Sphere::new(Vec3::ZERO, 0.5)))
        .with_position(Vec3::new(0.0, 5.0, 0.0))
        .with_restitution(0.6)
        .with_friction(0.4),
);

for _ in 0..120 {
    world.step(1.0 / 60.0);
}
```

## Collision shapes

```rust
use gravita_math::{Aabb3, Obb, Quat, Sphere, Vec3};
use gravita_physics_3d::CollisionShape;

// Sphere
CollisionShape::Sphere(Sphere::new(Vec3::ZERO, 0.5));

// World-axis-aligned box
CollisionShape::Aabb(Aabb3::from_center_size(Vec3::ZERO, Vec3::splat(1.0)));

// Oriented box: 1×1×1 rotated 45° around Y
CollisionShape::Obb(Obb::new(
    Vec3::ZERO,
    Vec3::splat(0.5),
    Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_4),
));
```

The narrow-phase dispatcher routes every box-vs-box pair through the SAT path (an axis-aligned `Aabb3` is promoted to an `Obb` with identity rotation), so adding `Obb` was strictly additive — Aabb-Aabb still has its own faster axis-of-minimum-penetration path for symmetry.

## Quaternion integration

Each frame:

```text
q' = normalize(q + 0.5 · (ω_vec_quat · q) · dt)
```

A test runs 10 000 frames with arbitrary `ω` and asserts `|q| ≈ 1`. The renormalisation step is mandatory — without it, the quaternion drifts off the unit sphere.

## Friction (Coulomb cone)

Inside the velocity-constraint solver, after the normal impulse `jn` is applied:

1. Recompute relative velocity at the contact point (`v + ω × r` for both bodies).
2. Project out the normal component to get the tangent direction.
3. Compute the tangential impulse `jt` to zero out the tangential velocity, including the angular contribution `(r × t) · I⁻¹ · (r × t)`.
4. Clamp `|jt| ≤ μ · |jn|` (the Coulomb friction cone).
5. Apply `jt` linearly and angularly.

## Broad phase

`SpatialHashGrid` stores a single flat sorted `Vec<(u64, u32)>`:

- `u64` packs three i32 cell coordinates into 63 bits (21 bits per axis, ±1M cells around origin, plenty for any game scale).
- Sort once per `update`, then a linear walk emits pairs from each equal-key run.
- The pair-dedup scratch buffer lives on the grid, surviving across frames — no per-step allocation.

## Bench coverage

```bash
cargo bench -p gravita-physics-3d
```

Covers `world.step` at 50/100/500 bodies, `SimpleCollisionDetector` vs `SpatialHashDetector` at 50/200/500 bodies, and OBB-OBB SAT cost (overlap vs no-overlap cases).

## License

MIT — see [../../LICENSE](../../LICENSE).
