<p align="center">
  <img src="https://img.shields.io/badge/rust-nightly-orange?logo=rust" alt="Rust nightly">
  <img src="https://img.shields.io/github/license/cantoramann/gravita" alt="MIT licensed">
  <img src="https://img.shields.io/badge/2D-pixels-blue" alt="2D pixels">
  <img src="https://img.shields.io/badge/3D-wgpu-purple" alt="3D wgpu">
  <img src="https://img.shields.io/badge/WASM-handwritten%20bindings-yellow" alt="WASM bindings">
  <img src="https://img.shields.io/badge/snapshots-deterministic-green" alt="Deterministic snapshots">
</p>

# Gravita

A modular physics engine and game framework for Rust, with **2D and 3D pipelines side by side**. Designed to be readable end-to-end — every layer (math, physics, renderer) is small enough to understand in an afternoon.

```text
2D (CPU)                 3D (GPU, wgpu)
─────────                ──────────────
gravita-renderer    ←→   gravita-renderer-3d
gravita-physics     ←→   gravita-physics-3d
        ╲                   ╱
         ╲    gravita-math    ╱   ← shared 2D/3D math, Vec2 + Vec3
          ╲       ↑↓        ╱
           gravita (umbrella with feature flags)
```

If you're new here, jump to **[Five-minute tour](#five-minute-tour)** below.

---

## What's inside

**2D pipeline.** [`gravita-physics`](crates/physics) runs a rigid-body simulation (circles + AABBs, semi-implicit Euler / Verlet, restitution + friction, spatial-hash broad phase, collision events). [`gravita-renderer`](crates/renderer) is a CPU framebuffer rasterizer — `clear`, `draw_circle`, `draw_line`, `draw_rect_filled`, `draw_axes`, a 5×7 bitmap font, alpha blending. No GPU required.

**3D pipeline.** [`gravita-physics-3d`](crates/physics-3d) handles 3D rigid bodies (`Sphere`, `Aabb3`, `Obb` with SAT), quaternion rotation, 3D spatial-hash broad phase, friction impulses. [`gravita-renderer-3d`](crates/renderer-3d) is a `wgpu` instanced-mesh renderer with a built-in `winit` runner.

**Shared math.** [`gravita-math`](crates/math) is dependency-free and used by both pipelines: `Vec2`, `Vec3`, `Quat`, `Aabb`, `Aabb3`, `Sphere`, `Obb`, `Ray2D`, `Ray3D`, `Transform2D`, `Transform3D`, plus a `Vector` trait so dim-agnostic code (`move_toward`, `attract`, …) can run against either dimension.

**Examples-only shim.** [`gravita-example-shim`](crates/example-shim) and the runner module inside `gravita-renderer-3d` factor out the `winit::ApplicationHandler` boilerplate; the 2D examples are ~60–100 LOC of actual game code each.

---

## Five-minute tour

### 1. Install Rust nightly

```bash
git clone https://github.com/cantoramann/gravita.git
cd gravita
# The repo pins a nightly toolchain in rust-toolchain.toml; rustup will use it automatically.
cargo --version
```

### 2. Run a 2D demo

```bash
cargo run -p bouncing-balls
# Click anywhere to spawn a new ball. Esc to quit.
```

You should see balls falling into a pit and bouncing realistically. Other 2D demos:

| Demo | Run | Controls |
|---|---|---|
| `bouncing-balls` | `cargo run -p bouncing-balls` | Click to spawn, Esc to quit |
| `gravity-arena` | `cargo run -p gravity-arena` | Arrow keys thrust/turn |
| `rotating-rod` | `cargo run -p rotating-rod` | (autonomous; Esc to quit) |
| `stickman-walk` | `cargo run -p stickman-walk` | A/D walk, Space jump |
| `tetris` | `cargo run -p tetris` | Arrows, Space hard-drop |
| `froggy-jump` | `cargo build --target wasm32-unknown-unknown -p froggy-jump` | (WASM build only — open `examples/froggy-jump/index.html` after a build) |

### 3. Run a 3D demo

```bash
cargo run -p spheres-3d
# Spheres bounce on a static floor. Arrow keys orbit the camera. Space spawns a fresh sphere. Esc to quit.
```

| Demo | Run | What it shows |
|---|---|---|
| `cube-3d` | `cargo run -p cube-3d` | Smoke test: spinning multicolored cube on a plane |
| `spheres-3d` | `cargo run -p spheres-3d` | `gravita-physics-3d` driving `gravita-renderer-3d` |

### 4. Write your own 2D sim

```rust
use gravita_math::{Aabb, Circle, Vec2};
use gravita_physics::{BodyType, CollisionShape, PhysicsWorld, RigidBody};

fn main() {
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
    }
    println!("Ball ended at {:?}", world.bodies()[1].position);
}
```

### 5. Write your own 3D sim

```rust
use gravita_math::{Sphere, Vec3};
use gravita_physics_3d::{BodyType, CollisionShape, PhysicsWorld, RigidBody};

fn main() {
    let mut world = PhysicsWorld::new();
    world.set_gravity(Vec3::new(0.0, -9.81, 0.0));

    // Drop a sphere from 5 m up.
    world.add_body(
        RigidBody::new(0, CollisionShape::Sphere(Sphere::new(Vec3::ZERO, 0.5)))
            .with_position(Vec3::new(0.0, 5.0, 0.0))
            .with_restitution(0.6),
    );

    for _ in 0..120 {
        world.step(1.0 / 60.0);
    }
    println!("Sphere is now at {:?}", world.bodies()[0].position);
}
```

For graphics, take `examples/spheres-3d/src/main.rs` as the template — it's <200 LOC of game code plus the `App3D` trait.

### 6. Use it from JavaScript / TypeScript

The [`gravita-wasm`](crates/wasm) crate ships **handwritten** WebAssembly bindings — every method is shaped for JS readers (plain numeric arguments, `Float32Array` returns), not the verbose builder pattern an auto-generated `wasm-bindgen` surface would produce.

```bash
# One-time:
rustup target add wasm32-unknown-unknown
cargo install wasm-pack

# Build the JS package:
wasm-pack build crates/wasm --target web --release
```

That writes a ~75 KB `gravita_wasm_bg.wasm` plus a JS loader and TypeScript `.d.ts` into `crates/wasm/pkg/`. Then in your page:

```js
import init, { World2D, BodyKind } from "./pkg/gravita_wasm.js";

await init();

const world = new World2D(0, -500);
const floor = world.addBox(0, 0, 800, 50);
world.setBodyKind(floor, BodyKind.Static);

const ball = world.addCircle(0, 300, 20);
world.setBodyRestitution(ball, 0.8);

for (let i = 0; i < 600; i++) world.step(1 / 60);
console.log(world.bodyPosition(ball));   // Float32Array [x, y]
```

3D works the same way — substitute `World3D` and three-component vectors. See [`crates/wasm/README.md`](crates/wasm/README.md) for the full surface.

### 7. Deterministic snapshot & time-rewind

Every `PhysicsWorld` can be serialized to a flat `Vec<u8>` and restored later, byte-for-byte. The step path is deterministic on a fixed binary — no `HashMap` iteration, no parallelism, no randomness — so the same input always produces the same output.

```rust
let snap = world.snapshot();           // Vec<u8> (no serde dep, no encoding overhead)

for _ in 0..120 { world.step(1.0 / 60.0); }

world.restore_from(&snap).unwrap();    // wind back two seconds
```

Identical surface on `gravita-physics-3d`. The WASM bindings expose them as `world.snapshot()` (returns `Uint8Array`) and `world.restoreFrom(bytes)`.

What this unlocks: **lockstep multiplayer** (ship the same binary to every client, exchange inputs, save snapshots as keyframes), **time-rewind gameplay** (Braid-style — see `cargo run -p snapshot-replay` and hold `R`), **replay-based debugging** (capture the bytes that triggered a bug, restore + step under a debugger), **RL/sim training** (snapshot the starting state once, fork it for thousands of rollouts).

Try it:

```bash
cargo run -p snapshot-replay
# Click anywhere to spawn balls. Hold R to rewind the last two seconds.
```

---

## Repo layout

```text
gravita/
├── crates/
│   ├── math/              # Vec2/Vec3, Quat, Aabb/Aabb3, Sphere, Obb, Ray2D/3D, transforms
│   ├── physics/           # 2D rigid body sim
│   ├── physics-3d/        # 3D rigid body sim (Quat rotation, SAT for OBB)
│   ├── renderer/          # CPU 2D rasterizer
│   ├── renderer-3d/       # wgpu 3D renderer + winit runner
│   ├── collections/       # Pre-built 2D game objects (Stickman, Spaceship, Planet)
│   ├── input/             # Cross-app input state (keys, mouse, cursor)
│   ├── wasm/              # JS-friendly WebAssembly bindings (World2D + World3D)
│   ├── example-shim/      # Internal: winit/pixels glue for 2D examples
│   └── gravita/           # Umbrella crate — re-exports everything via features
├── examples/
│   ├── bouncing-balls/    # 2D physics
│   ├── gravity-arena/     # 2D orbital mechanics
│   ├── rotating-rod/      # 2D pendulum
│   ├── stickman-walk/     # 2D animated character
│   ├── tetris/            # 2D classic game
│   ├── froggy-jump/       # 2D + WASM
│   ├── cube-3d/           # 3D spinning cube
│   ├── spheres-3d/        # 3D bouncing-spheres
│   └── snapshot-replay/   # Time-rewind via deterministic snapshots
├── ARCHITECTURE.md        # System map + per-frame dataflow
├── CLAUDE.md              # Onboarding notes for Claude Code and other AI agents
├── CONTRIBUTING.md        # How to contribute
└── README.md              # You are here
```

For the conceptual map of how those crates talk to each other, see **[ARCHITECTURE.md](ARCHITECTURE.md)**.

---

## Feature flags

The umbrella `gravita` crate exposes everything behind cargo features so you only compile what you need.

| Feature | Default | Pulls in | Use when |
|---|---|---|---|
| `math` | ✅ | nothing | Always — 2D **and** 3D types live here |
| `physics` | ✅ | `math` | 2D rigid-body sim |
| `physics-3d` | ❌ | `math` | 3D rigid-body sim |
| `renderer` | ✅ | nothing | 2D CPU framebuffer drawing |
| `renderer-3d` | ❌ | wgpu + winit | GPU 3D rendering + windowed runner |
| `collections` | ❌ | `math`, `renderer` | Pre-built 2D characters |
| `full` | ❌ | everything | Get the whole engine |

Examples — 2D only:

```toml
[dependencies]
gravita = { version = "0.1", default-features = false, features = ["math", "physics", "renderer"] }
```

3D only:

```toml
[dependencies]
gravita = { version = "0.1", default-features = false, features = ["math", "physics-3d", "renderer-3d"] }
```

---

## Development

```bash
cargo build --workspace               # all crates + examples
cargo test --workspace                # 513 tests
cargo clippy --workspace --all-targets # workspace lints (see Cargo.toml)
cargo fmt --all                       # rustfmt
cargo bench -p gravita-math           # math benches (criterion)
cargo bench -p gravita-physics        # 2D physics benches
cargo bench -p gravita-physics-3d     # 3D physics benches (step + broad phase + OBB SAT)
cargo doc --workspace --no-deps --open # render API docs
```

---

## Roadmap

- [x] 2D rigid body physics (circles, AABBs, friction, restitution, contact events)
- [x] 2D spatial-hash broad phase
- [x] 3D rigid body physics (Sphere, AABB, OBB with SAT)
- [x] 3D spatial-hash broad phase
- [x] Quaternion-based 3D rotation
- [x] Friction impulses (Coulomb cone) in both 2D and 3D
- [x] CPU 2D renderer (lines, circles, rects, text, alpha blend)
- [x] GPU 3D renderer (wgpu, instanced colored meshes, depth, directional light)
- [ ] Constraint / joint system (distance, hinge)
- [ ] Polygon (`Convex`) collision shape with SAT in 2D
- [ ] Capsule shape (both 2D and 3D)
- [ ] Continuous collision detection (CCD) for fast-moving bodies
- [ ] Audio crate
- [ ] ECS integration (`bevy_ecs` compat layer)

---

## License

MIT — see [LICENSE](LICENSE).

## Contributing

Pull requests welcome. Style and review expectations are in [CONTRIBUTING.md](CONTRIBUTING.md). If you're using an AI coding assistant on this repo, point it at [CLAUDE.md](CLAUDE.md) first — it's the agent-specific onboarding.
