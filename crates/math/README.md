# gravita-math

Dependency-free 2D and 3D math primitives for game development and physics simulation.

## Types

**2D**

| Type | Purpose |
|---|---|
| `Vec2` | 2D vector (`x`, `y` public fields) |
| `Aabb` | Axis-aligned bounding box |
| `Circle` | Circle primitive |
| `Ray2D` | Ray + intersection tests against `Aabb` / `Circle` |
| `Transform2D` | Position + rotation transform |

**3D**

| Type | Purpose |
|---|---|
| `Vec3` | 3D vector (`x`, `y`, `z` public fields) |
| `Quat` | Unit quaternion (`x`, `y`, `z`, `w`) |
| `Aabb3` | Axis-aligned 3D bounding box |
| `Sphere` | Sphere primitive |
| `Obb` | Oriented bounding box (center + half-extents + `Quat` rotation) |
| `Ray3D` + `RayHit3D` | Ray + intersection tests against `Aabb3` / `Sphere` |
| `Transform3D` | Translate-Rotate-Scale, outputs 4×4 column-major matrices |

**Shared**

The [`Vector`](src/vector.rs) trait is implemented by both `Vec2` and `Vec3`. Use it to write dimension-agnostic algorithms:

```rust
use gravita_math::{Vec2, Vec3, Vector};

fn move_toward<V: Vector>(p: V, target: V, max_step: f32) -> V {
    let delta = target - p;
    let dist = delta.length();
    if dist <= max_step { target } else { p + delta * (max_step / dist) }
}

let p_2d = move_toward(Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), 3.0);
let p_3d = move_toward(Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 0.0, 0.0), 3.0);
```

## Design

- **Zero dependencies.** The crate compiles instantly and ships nothing transitively.
- **Public fields.** `v.x` reads naturally; this beats const generics' indexing ergonomics.
- **`#[inline]` everywhere.** Vector ops are designed to vanish under optimisation.
- **`mul_add` in hot paths.** `length_squared`, quaternion multiplication, etc. use FMA when the workspace lints push for it.

## Conventions

- **3D coordinate system**: right-handed, **Y up**, forward is **-Z** (`Vec3::FORWARD == -Vec3::Z`).
- **Quaternion convention**: `(x, y, z, w)` with `w` scalar; Hamilton product `a * b` applies `b` then `a`.
- **`Aabb` (not `AABB`)**: per RFC 430 acronym casing.
- **`length()` uses `length_squared().sqrt()`** — not `hypot()`. Faster at game scale; overflow protection of `hypot` isn't worth the cost.

## Quick example

```rust
use gravita_math::{Aabb, Circle, Vec2, Vec3, Quat, Transform3D};

// 2D
let v = Vec2::new(3.0, 4.0);
assert_eq!(v.length(), 5.0);
let aabb = Aabb::from_center_size(Vec2::ZERO, Vec2::new(10.0, 10.0));
assert!(aabb.contains_point(Vec2::ZERO));

// 3D
let cross = Vec3::X.cross(Vec3::Y);
assert_eq!(cross, Vec3::Z);

let q = Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2);
// 90° around Y takes +X to -Z.
let rotated = q.rotate_vec(Vec3::X);
assert!((rotated - Vec3::FORWARD).length() < 1e-5);

let t = Transform3D::IDENTITY
    .with_position(Vec3::new(1.0, 2.0, 3.0))
    .with_rotation(q);
let matrix: [[f32; 4]; 4] = t.to_matrix(); // wgpu/glam-compatible column-major
```

## Test coverage

`cargo test -p gravita-math` runs 246+ tests including:

- Vec2/Vec3 algebra, edge cases (zero, large values, NaN propagation)
- Quaternion: axis-angle, Euler, rotation arc, composition, inverse
- AABB / Aabb3 / Sphere / Obb intersection + closest-point
- Ray2D / Ray3D vs AABB / Sphere
- IEEE-754 precision tests (split into `tests/precision.rs` as an integration test)

## License

MIT — see [../LICENSE](../../LICENSE).
