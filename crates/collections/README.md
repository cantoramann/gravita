# gravita-collections

Pre-built game objects for rapid prototyping.

## Available Objects

### Stickman

Animated humanoid character with walking and jumping.

```rust
use gravita_collections::Stickman;

let mut stickman = Stickman::new(ground_y, screen_width);

// Input handling
stickman.set_move_direction(1.0);  // Move right (-1 = left, 0 = stop)
stickman.jump();                   // Jump if grounded

// Update and render
stickman.update(dt, screen_width);
stickman.render(&mut frame, width, height);
```

### Spaceship

Thrust-based vehicle with rotation controls.

```rust
use gravita_collections::Spaceship;

let mut ship = Spaceship::new(Vec2::new(400.0, 300.0));

// Input handling
ship.set_input(thrust, turn);  // thrust: 0-1, turn: -1 to 1

// Update and render
ship.update(dt);
ship.render(&mut frame, width, height);
```

### Planet

Static celestial body for orbital mechanics demos.

```rust
use gravita_collections::Planet;

let planet = Planet::new(Vec2::new(512.0, 384.0), 50.0);
planet.render(&mut frame, width, height);
```

## License

MIT OR Apache-2.0

