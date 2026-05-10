use gravita_collections::{Planet, Spaceship};
use gravita_example_shim::{App, Input, ShimKeyCode, WindowConfig, run};
use gravita_math::Vec2;
use gravita_renderer::{clear, draw_axes};

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 768;

struct Demo {
    spaceship: Spaceship,
    planets: Vec<Planet>,
}

impl Demo {
    fn new() -> Self {
        let center = Vec2::new(WIDTH as f32 * 0.5, HEIGHT as f32 * 0.5);
        let planets = vec![
            Planet::new(center, 40.0),
            Planet::new(center + Vec2::new(-220.0, -120.0), 30.0),
            Planet::new(center + Vec2::new(260.0, 160.0), 24.0),
        ];
        let spaceship = Spaceship::new(center + Vec2::new(0.0, -220.0));
        Self { spaceship, planets }
    }

    fn apply_gravity(&mut self, dt: f32) {
        // Accumulate gravitational acceleration from all planets (inverse square).
        let mut accel = Vec2::ZERO;
        for planet in &self.planets {
            let dir = planet.center - self.spaceship.position;
            let dist_sq = dir.length_squared().max(1.0);
            let dist = dist_sq.sqrt();
            // Avoid extreme forces very close to the center.
            if dist < planet.radius * 0.8 {
                continue;
            }
            // Tuned gravity constant for visible orbits at pixel scale.
            let g = 200_000.0;
            let strength = g / dist_sq;
            accel += dir * (strength / dist);
        }
        self.spaceship.velocity += accel * dt;
    }
}

impl App for Demo {
    fn update(&mut self, dt: f32, input: &Input) {
        let thrust = if input.key_held(ShimKeyCode::ArrowUp) || input.key_held(ShimKeyCode::Space) {
            1.0
        } else {
            0.0
        };
        let mut turn = 0.0;
        if input.key_held(ShimKeyCode::ArrowLeft) {
            turn -= 1.0;
        }
        if input.key_held(ShimKeyCode::ArrowRight) {
            turn += 1.0;
        }
        self.spaceship.set_input(thrust, turn);

        self.apply_gravity(dt);
        self.spaceship.update(dt);

        // Wrap the ship around the screen so it never leaves view.
        let mut pos = self.spaceship.position;
        if pos.x < 0.0 {
            pos.x += WIDTH as f32;
        } else if pos.x > WIDTH as f32 {
            pos.x -= WIDTH as f32;
        }
        if pos.y < 0.0 {
            pos.y += HEIGHT as f32;
        } else if pos.y > HEIGHT as f32 {
            pos.y -= HEIGHT as f32;
        }
        self.spaceship.position = pos;
    }

    fn render(&self, frame: &mut [u8]) {
        clear(frame, [0x06, 0x08, 0x10, 0xff]);
        let center = Vec2::new(WIDTH as f32 * 0.5, HEIGHT as f32 * 0.5);
        draw_axes(frame, center, [0x20, 0x20, 0x40, 0xff], WIDTH, HEIGHT);
        for planet in &self.planets {
            planet.render(frame, WIDTH, HEIGHT);
        }
        self.spaceship.render(frame, WIDTH, HEIGHT);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run(
        WindowConfig {
            title: "Gravity Arena",
            width: WIDTH,
            height: HEIGHT,
            ..WindowConfig::default()
        },
        Demo::new(),
    )
}
