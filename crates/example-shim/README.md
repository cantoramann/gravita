# gravita-example-shim

Internal crate (`publish = false`). Provides the windowing and event-loop scaffolding that every 2D example would otherwise re-roll.

If you're building a real app: this is a template, not a library. Copy the pattern.

## What it provides

- `App` trait — implement `update(dt, &Input)` + `render(&mut [u8])`, get a fixed-timestep loop.
- `Input` — keyboard + mouse + cursor snapshot, with `key_held` / `key_pressed` / `mouse_held` / `mouse_pressed` / `cursor`.
- `WindowConfig` — title, width, height, fixed timestep.
- `run(config, app)` — owns the `winit::ApplicationHandler` + `pixels::Pixels` framebuffer + a `ControlFlow::WaitUntil` scheduler so the demo doesn't pin a CPU core when idle.

## Minimal example

```rust
use gravita_example_shim::{App, Input, ShimKeyCode, WindowConfig, run};
use gravita_renderer::{clear, palette};

struct Demo;

impl App for Demo {
    fn update(&mut self, _dt: f32, input: &Input) {
        if input.key_pressed(ShimKeyCode::Space) {
            // …
        }
    }
    fn render(&self, frame: &mut [u8]) {
        clear(frame, palette::DARK_BLUE_BG);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run(WindowConfig::default(), Demo)
}
```

See [`examples/bouncing-balls`](../../examples/bouncing-balls/) for a fully-wired demo with physics, click-to-spawn, and rendering.

## License

MIT — see [../../LICENSE](../../LICENSE).
