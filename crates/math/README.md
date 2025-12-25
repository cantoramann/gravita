# gravita-math

Minimal 2D math primitives for game development and physics simulation.

## Features

- **`Vec2`** — 2D vector with full operator support
- **`AABB`** — Axis-aligned bounding box
- **`Circle`** — Circle primitive
- **`Ray2D`** — Ray for intersection tests
- **`Transform2D`** — Position + rotation

## Design Goals

- Zero external dependencies
- All operations inlined for performance
- Simple, predictable API

## Usage

```rust
use gravita_math::{Vec2, AABB, Circle, lerp, clamp};

// Vector operations
let v = Vec2::new(3.0, 4.0);
assert_eq!(v.length(), 5.0);
assert_eq!(v.normalize().length(), 1.0);

// AABB operations
let box1 = AABB::from_center_size(Vec2::ZERO, Vec2::new(10.0, 10.0));
let box2 = AABB::from_center_size(Vec2::new(5.0, 0.0), Vec2::new(10.0, 10.0));
assert!(box1.intersects(&box2));

// Utility functions
let clamped = clamp(150.0, 0.0, 100.0); // 100.0
let interpolated = lerp(0.0, 100.0, 0.5); // 50.0
```

## License

MIT OR Apache-2.0

