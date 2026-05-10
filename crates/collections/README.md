# gravita-collections

Pre-built 2D game objects that demonstrate how to wire `gravita-math` + `gravita-renderer` together.

These exist as concrete tutorials — they're useful as starting points, not as a production game-object library. Copy what you need and modify freely.

## What's in the box

| Object | Behaviour | Constructor |
|---|---|---|
| `Stickman` | Walks left/right, jumps, gravity-clamped to a ground line | `Stickman::new(base_y, screen_width)` |
| `Spaceship` | Thrust + rotate + linear/angular damping, screen-wrap | `Spaceship::new(position)` |
| `Planet` | Static gravitational body (just a circle for now) | `Planet::new(center, radius)` |

All three implement the shared [`Drawable`](src/lib.rs) trait:

```rust
pub trait Drawable {
    fn render(&self, frame: &mut [u8], width: u32, height: u32);
}
```

so example runners can iterate `&[Box<dyn Drawable>]` and call `render` uniformly.

## Quick example

```rust
use gravita_collections::{Drawable, Spaceship, Planet, Stickman};
use gravita_math::Vec2;

let scene: Vec<Box<dyn Drawable>> = vec![
    Box::new(Stickman::new(560.0, 800.0)),
    Box::new(Spaceship::new(Vec2::new(400.0, 300.0))),
    Box::new(Planet::new(Vec2::new(400.0, 300.0), 100.0)),
];

let mut frame = vec![0u8; 800 * 600 * 4];
for entity in &scene {
    entity.render(&mut frame, 800, 600);
}
```

For complete usage, see [`examples/stickman-walk`](../../examples/stickman-walk/) and [`examples/gravity-arena`](../../examples/gravity-arena/).

## License

MIT — see [../../LICENSE](../../LICENSE).
