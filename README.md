<p align="center">
  <img src="https://img.shields.io/badge/rust-nightly--2025-orange?logo=rust" alt="Rust Version">
  <img src="https://img.shields.io/github/license/cantoramann/gravita" alt="License">
  <img src="https://img.shields.io/badge/wasm-supported-green" alt="WASM Support">
</p>

# 🌌 Gravita

A modular 2D physics engine and game framework written in Rust. Designed for real-time simulations, games, and educational exploration of physics concepts.

## ✨ Features

- **🔬 Physics Simulation**
  - Rigid body dynamics with linear and angular motion
  - Collision detection (circles, AABBs)
  - Contact resolution with friction and restitution
  - Pluggable integrators (Semi-implicit Euler, Verlet)
  - Configurable gravity and damping

- **🧮 Math Library**
  - Dependency-free 2D vector math (`Vec2`)
  - Axis-aligned bounding boxes (`AABB`)
  - Circles, rays, and transforms
  - Utility functions: lerp, clamp, remap, smooth_step

- **🎨 Renderer**
  - Minimal CPU-based 2D rendering
  - Drawing primitives: circles, lines, rectangles
  - Frame buffer rendering via [pixels](https://github.com/parasyte/pixels)

- **🎮 Game Collections**
  - Pre-built game objects: Stickman, Spaceship, Planet
  - Ready-to-use animated characters with movement logic

- **🌐 Cross-Platform**
  - Native: macOS (ARM64/x86_64), Linux (ARM64/x86_64)
  - Web: WebAssembly (wasm32-unknown-unknown)

## 📦 Crate Structure

```
gravita/
├── crates/
│   ├── math/           # 2D vector math, geometry primitives
│   ├── physics/        # Rigid body dynamics, collision detection
│   ├── renderer/       # Minimal 2D rendering utilities
│   ├── collections/    # Pre-built game objects (Stickman, Spaceship, etc.)
│   ├── engine-core/    # Core engine orchestration (WIP)
│   ├── input/          # Input handling abstraction (WIP)
│   └── assets/         # Asset loading utilities (WIP)
└── examples/
    ├── bouncing-balls/ # Physics demo with bouncing balls
    ├── gravity-arena/  # Orbital mechanics with spaceship
    ├── rotating-rod/   # Angular momentum demonstration
    ├── stickman-walk/  # Animated character with controls
    └── tetris/         # Classic Tetris implementation
```

## 🚀 Quick Start

### Prerequisites

- **Rust nightly** (2025-08-08 or later)

The project uses `rust-toolchain.toml` to automatically select the correct toolchain.

### Running an Example

```bash
# Clone the repository
git clone https://github.com/cantoramann/gravita.git
cd gravita

# Run the bouncing balls demo
cargo run --example bouncing-balls

# Run the gravity arena (spaceship simulation)
cargo run -p gravity-arena

# Run the stickman walking demo
cargo run -p stickman-walk
```

### Using as a Library

Add the crates you need to your `Cargo.toml`:

```toml
[dependencies]
gravita-math = { git = "https://github.com/cantoramann/gravita" }
gravita-physics = { git = "https://github.com/cantoramann/gravita" }
```

## 📖 Usage Examples

### Basic Physics Simulation

```rust
use gravita_math::{Vec2, Circle};
use gravita_physics::{PhysicsWorld, RigidBody, CollisionShape, BodyType};

fn main() {
    // Create a physics world
    let mut world = PhysicsWorld::new();
    world.set_gravity(Vec2::new(0.0, -500.0));

    // Create a static ground
    let ground = RigidBody::new(0, CollisionShape::AABB(
        AABB::from_center_size(Vec2::ZERO, Vec2::new(800.0, 50.0))
    ))
    .with_type(BodyType::Static)
    .with_position(Vec2::new(400.0, 25.0));
    world.add_body(ground);

    // Create a bouncing ball
    let ball = RigidBody::new(0, CollisionShape::Circle(
        Circle::new(Vec2::ZERO, 20.0)
    ))
    .with_position(Vec2::new(400.0, 300.0))
    .with_density(1.0);
    world.add_body(ball);

    // Simulate
    let dt = 1.0 / 60.0;
    for _ in 0..600 {
        world.step(dt);
        
        for body in world.get_bodies() {
            println!("Body {} at {:?}", body.id, body.position);
        }
    }
}
```

### Vector Math

```rust
use gravita_math::{Vec2, lerp, clamp};

let a = Vec2::new(3.0, 4.0);
let b = Vec2::new(1.0, 2.0);

// Basic operations
let sum = a + b;                    // Vec2(4.0, 6.0)
let scaled = a * 2.0;               // Vec2(6.0, 8.0)
let length = a.length();            // 5.0
let normalized = a.normalize();     // Vec2(0.6, 0.8)
let dot = a.dot(b);                 // 11.0

// Rotation
let rotated = Vec2::RIGHT.rotate(std::f32::consts::FRAC_PI_2); // Vec2(0.0, 1.0)

// Interpolation
let mid = a.lerp(b, 0.5);           // Vec2(2.0, 3.0)
```

### Collision Events

```rust
// After stepping the physics world
for event in world.get_collision_events() {
    println!(
        "Collision between {} and {} with impulse {}",
        event.body_a,
        event.body_b,
        event.impulse_magnitude
    );
    
    // Use impulse_magnitude for sound/damage systems
    if event.impulse_magnitude > 100.0 {
        play_impact_sound(event.impulse_magnitude);
    }
}
```

## 🛠️ Development

### Building

```bash
# Build all crates
cargo build --workspace

# Build in release mode
cargo build --workspace --release

# Build for WebAssembly
cargo build --target wasm32-unknown-unknown -p stickman-walk
```

### Testing

```bash
# Run all tests
cargo test --workspace

# Run tests with output
cargo test --workspace -- --nocapture
```

### Linting

```bash
# Check formatting
cargo fmt --check

# Run clippy
cargo clippy --workspace --all-targets
```

### Documentation

```bash
# Generate documentation
cargo doc --workspace --no-deps --open
```

## 🎯 Examples in Detail

| Example | Description | Controls |
|---------|-------------|----------|
| `bouncing-balls` | Physics demo with bouncing balls | Click to spawn balls, Esc to exit |
| `gravity-arena` | Orbital mechanics simulation | Arrow keys to thrust/turn, Esc to exit |
| `rotating-rod` | Angular momentum demonstration | Mouse to interact |
| `stickman-walk` | Animated stickman character | A/D to walk, Space to jump |
| `tetris` | Classic Tetris game | Arrow keys, Space to drop |

## 🗺️ Roadmap

- [ ] Polygon collision shapes
- [ ] Constraint/joint system
- [ ] Spatial hashing broad-phase optimization
- [ ] GPU-accelerated rendering (wgpu)
- [ ] Audio integration
- [ ] ECS integration (bevy compatibility)

## 📄 License

This project is licensed under the [MIT License](LICENSE).

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

