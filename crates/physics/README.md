# gravita-physics

2D rigid-body physics for real-time games and simulations.

## What it does

- Linear + angular rigid-body dynamics
- Collision: Circle-Circle, Circle-Aabb, Aabb-Aabb
- Contact resolution with restitution + Coulomb friction
- Pluggable integrators: `SemiImplicitEuler` (default) or `Verlet`
- Broad phase: O(N²) `SimpleCollisionDetector` or `SpatialHashDetector`
- Force generators: gravity, drag (linear + quadratic), springs, anchored springs, bungees, radial, directional, buoyancy, field forces
- Collision events with impulse magnitude — usable for game logic (damage, sound)

## Pipeline

```text
PhysicsWorld::step(dt)
  ├── clear_forces
  ├── apply_gravity_and_damping       (single dynamic-body pass)
  ├── integrator.integrate_velocity   (per body)
  ├── collision_detector.detect       (broad → narrow → contacts)
  ├── solve_velocity_constraint       × velocity_iterations (default 8)
  │     • normal impulse with restitution
  │     • tangential friction impulse clamped to Coulomb cone
  ├── integrator.integrate_position
  ├── solve_position_constraint       × position_iterations (default 3)
  └── apply_sleeping                  (snap tiny velocities to zero)
```

## Quick example

```rust
use gravita_math::{Aabb, Circle, Vec2};
use gravita_physics::{BodyType, CollisionShape, PhysicsWorld, RigidBody};

let mut world = PhysicsWorld::new();
world.set_gravity(Vec2::new(0.0, -500.0));

// Static floor.
world.add_body(
    RigidBody::new(0, CollisionShape::Aabb(
        Aabb::from_center_size(Vec2::new(400.0, 25.0), Vec2::new(800.0, 50.0)),
    ))
    .with_type(BodyType::Static),
);

// Bouncy ball.
world.add_body(
    RigidBody::new(0, CollisionShape::Circle(Circle::new(Vec2::ZERO, 20.0)))
        .with_position(Vec2::new(400.0, 300.0))
        .with_density(1.0)
        .with_restitution(0.8),
);

for _ in 0..600 {
    world.step(1.0 / 60.0);
    for event in world.get_collision_events() {
        if event.impulse_magnitude > 100.0 {
            // hard impact — play a sound, deal damage, etc.
        }
    }
}
```

## Body types

| Type | Moved by physics? | Moved by user code? | Typical use |
|---|---|---|---|
| `Static` | No | No (treated as fixed) | Ground, walls |
| `Kinematic` | No | Yes (you set `position`) | Moving platforms |
| `Dynamic` | Yes | Yes | Players, projectiles |

Internally, non-Dynamic bodies have `inv_mass = 0` so impulses applied to them have no effect — they're "infinite mass" from the solver's perspective.

## RigidBody field visibility

We split fields into three categories. See [CLAUDE.md](../../CLAUDE.md#encapsulation-patterns) for the policy.

- **`pub`** — user-facing state: `position`, `velocity`, `rotation`, `angular_velocity`, `restitution`, `friction`, `linear_damping`, `angular_damping`, `gravity_scale`, `shape`, `is_sensor`.
- **`pub(crate)`** — solver-managed: `inv_mass`, `inv_inertia`, `force_accumulator`, `torque_accumulator`, `acceleration`, `angular_acceleration`.
- **`pub(crate)` + typed setter** — invariant-protected: `mass`, `inertia`, `body_type`, `fixed_rotation`. Setters (`set_mass`, etc.) refresh derived fields.

Use the `with_*` builders for construction:

```rust
let body = RigidBody::new(0, shape)
    .with_position(Vec2::new(100.0, 0.0))
    .with_velocity(Vec2::new(50.0, 0.0))
    .with_density(2.0)
    .with_restitution(0.6)
    .with_friction(0.4);
```

## Bench coverage

```bash
cargo bench -p gravita-physics
```

Includes narrow-phase pair tests, broad-phase scaling (`SimpleCollisionDetector` vs `SpatialHashDetector` for 50–500 bodies), and full-step throughput.

## License

MIT — see [../../LICENSE](../../LICENSE).
