// examples/snapshot-replay/src/main.rs
//
// Time-rewind demo for `gravita_physics::Snapshot`.
//
// Every frame we push a snapshot into a fixed-size ring buffer (default 2
// seconds at 60 FPS). Holding `R` pops snapshots off the buffer one per
// frame, restoring the world to that earlier state — bodies visibly fly
// backwards. Release `R` to resume forward. Click to spawn a fresh ball.
//
// The same restored bytes, stepped forward, would produce the same
// trajectory as a never-rewound run. That's the determinism guarantee in
// motion.

use std::collections::VecDeque;

use gravita_example_shim::{App, Input, ShimKeyCode, ShimMouseButton, WindowConfig, run};
use gravita_math::{Aabb, Circle, Vec2};
use gravita_physics::{BodyType, CollisionShape, PhysicsWorld, RigidBody, Snapshot};
use gravita_renderer::{
    Color, PixelRect, clear, draw_circle, draw_rect_filled, draw_text_scaled, palette,
};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
const HISTORY_FRAMES: usize = 120; // 2 seconds at 60 FPS.

struct Demo {
    world: PhysicsWorld,
    history: VecDeque<Snapshot>,
    rewinding: bool,
}

impl Demo {
    fn new() -> Self {
        let mut world = PhysicsWorld::new();
        world.set_gravity(Vec2::new(0.0, -500.0));

        // Floor + left/right walls.
        for (size, center) in [
            (Vec2::new(700.0, 50.0), Vec2::new(400.0, 50.0)),
            (Vec2::new(50.0, 600.0), Vec2::new(25.0, 300.0)),
            (Vec2::new(50.0, 600.0), Vec2::new(775.0, 300.0)),
        ] {
            world.add_body(
                RigidBody::new(
                    0,
                    CollisionShape::Aabb(Aabb::from_center_size(Vec2::ZERO, size)),
                )
                .with_type(BodyType::Static)
                .with_position(center),
            );
        }

        // Six bouncy balls staggered across the top.
        for i in 0..6 {
            world.add_body(
                RigidBody::new(0, CollisionShape::Circle(Circle::new(Vec2::ZERO, 18.0)))
                    .with_position(Vec2::new(150.0 + i as f32 * 100.0, 450.0))
                    .with_density(1.0)
                    .with_restitution(0.85)
                    .with_friction(0.2),
            );
        }

        Self {
            world,
            history: VecDeque::with_capacity(HISTORY_FRAMES),
            rewinding: false,
        }
    }

    fn spawn_ball_at(&mut self, screen_x: f32, screen_y: f32) {
        let ball = RigidBody::new(0, CollisionShape::Circle(Circle::new(Vec2::ZERO, 12.0)))
            .with_position(Vec2::new(screen_x, HEIGHT as f32 - screen_y))
            .with_density(1.0)
            .with_restitution(0.9)
            .with_friction(0.15);
        self.world.add_body(ball);
        // Spawning invalidates rewind history (those snapshots have fewer
        // bodies and don't match the new sim).
        self.history.clear();
    }
}

impl App for Demo {
    fn update(&mut self, dt: f32, input: &Input) {
        if input.mouse_pressed(ShimMouseButton::Left)
            && let Some((x, y)) = input.cursor()
        {
            self.spawn_ball_at(x, y);
            return;
        }

        self.rewinding = input.key_held(ShimKeyCode::KeyR);

        if self.rewinding {
            // Pop the most recent snapshot and restore. When the buffer is
            // drained we sit still — there's nothing further back to visit.
            if let Some(snap) = self.history.pop_back() {
                // Unwrap is safe: we only ever push snapshots we just produced.
                self.world.restore_from(&snap).unwrap();
            }
        } else {
            // Save before stepping so the ring buffer holds pre-step states.
            if self.history.len() == HISTORY_FRAMES {
                self.history.pop_front();
            }
            self.history.push_back(self.world.snapshot());
            self.world.step(dt);
        }
    }

    fn render(&self, frame: &mut [u8]) {
        clear(frame, palette::DARK_BLUE_BG);

        for body in self.world.bodies() {
            let color = body_color(body.body_type());
            match &body.shape {
                CollisionShape::Circle(c) => {
                    let p = body.position + c.center;
                    draw_circle(
                        frame,
                        Vec2::new(p.x, HEIGHT as f32 - p.y),
                        c.radius,
                        color,
                        WIDTH,
                        HEIGHT,
                    );
                },
                CollisionShape::Aabb(aabb) => {
                    let translated = aabb.translate(body.position);
                    draw_rect_filled(
                        frame,
                        PixelRect::new(
                            translated.min.x as i32,
                            (HEIGHT as f32 - translated.max.y) as i32,
                            translated.size().x as i32,
                            translated.size().y as i32,
                        ),
                        color,
                        WIDTH,
                        HEIGHT,
                    );
                },
            }
        }

        // HUD: status + buffer state, top-left.
        let frames_left = self.history.len();
        let line1 = "Click to spawn a ball.  Hold R to rewind.";
        let line2 = if self.rewinding {
            format!("REWINDING  ({frames_left:>3} frames remaining)")
        } else {
            format!("Forward    ({frames_left:>3} frames buffered)")
        };
        draw_text_scaled(frame, line1, 12, 12, palette::WHITE, 2, WIDTH, HEIGHT);
        draw_text_scaled(frame, &line2, 12, 36, palette::WHITE, 2, WIDTH, HEIGHT);
    }
}

fn body_color(body_type: BodyType) -> Color {
    match body_type {
        BodyType::Static => [0x60, 0x60, 0x60, 0xff],
        BodyType::Dynamic => [0x40, 0xa0, 0xff, 0xff],
        BodyType::Kinematic => [0xff, 0xa0, 0x40, 0xff],
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run(
        WindowConfig {
            title: "Gravita - Snapshot Replay (hold R to rewind)",
            width: WIDTH,
            height: HEIGHT,
            ..WindowConfig::default()
        },
        Demo::new(),
    )
}
