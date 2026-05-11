use gravita_example_shim::{App, Input, ShimKeyCode, ShimMouseButton, WindowConfig, run};
use gravita_math::{Aabb, Circle, Vec2};
use gravita_physics::{BodyType, CollisionShape, PhysicsWorld, RigidBody};
use gravita_renderer::{
    Color, PixelRect, clear, draw_circle, draw_line, draw_rect_filled, palette,
};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

struct Demo {
    physics_world: PhysicsWorld,
}

impl Demo {
    fn new() -> Self {
        let mut physics_world = PhysicsWorld::new();
        physics_world.set_gravity(Vec2::new(0.0, -500.0));

        let ground = RigidBody::new(
            0,
            CollisionShape::Aabb(Aabb::from_center_size(Vec2::ZERO, Vec2::new(700.0, 50.0))),
        )
        .with_type(BodyType::Static)
        .with_position(Vec2::new(400.0, 50.0));
        physics_world.add_body(ground);

        let left_wall = RigidBody::new(
            0,
            CollisionShape::Aabb(Aabb::from_center_size(Vec2::ZERO, Vec2::new(50.0, 600.0))),
        )
        .with_type(BodyType::Static)
        .with_position(Vec2::new(25.0, 300.0));
        physics_world.add_body(left_wall);

        let right_wall = RigidBody::new(
            0,
            CollisionShape::Aabb(Aabb::from_center_size(Vec2::ZERO, Vec2::new(50.0, 600.0))),
        )
        .with_type(BodyType::Static)
        .with_position(Vec2::new(775.0, 300.0));
        physics_world.add_body(right_wall);

        for i in 0..5 {
            let radius = 20.0 + (i as f32) * 5.0;
            let ball = RigidBody::new(0, CollisionShape::Circle(Circle::new(Vec2::ZERO, radius)))
                .with_type(BodyType::Dynamic)
                .with_position(Vec2::new(200.0 + i as f32 * 100.0, 400.0 + i as f32 * 30.0))
                .with_density(1.0)
                .with_restitution(0.7)
                .with_friction(0.3);
            physics_world.add_body(ball);
        }

        Self { physics_world }
    }

    fn spawn_ball_at(&mut self, screen_x: f32, screen_y: f32) {
        let ball = RigidBody::new(0, CollisionShape::Circle(Circle::new(Vec2::ZERO, 15.0)))
            .with_type(BodyType::Dynamic)
            // Click coordinates are screen space (Y-down); world is Y-up.
            .with_position(Vec2::new(screen_x, HEIGHT as f32 - screen_y))
            .with_density(1.0)
            .with_restitution(0.8)
            .with_friction(0.2);
        self.physics_world.add_body(ball);
    }

    fn draw_body_circle(frame: &mut [u8], center: Vec2, radius: f32, body_type: BodyType) {
        let color = body_color(body_type, true);
        let screen_center = Vec2::new(center.x, HEIGHT as f32 - center.y);
        draw_circle(frame, screen_center, radius, color, WIDTH, HEIGHT);
    }

    fn draw_body_aabb(frame: &mut [u8], aabb: Aabb, rotation: f32, body_type: BodyType) {
        let color = body_color(body_type, false);
        let min_x = aabb.min.x.floor() as i32;
        let max_x = aabb.max.x.ceil() as i32;
        let min_y = (HEIGHT as f32 - aabb.max.y).floor() as i32;
        let max_y = (HEIGHT as f32 - aabb.min.y).ceil() as i32;
        draw_rect_filled(
            frame,
            PixelRect::new(min_x, min_y, max_x - min_x, max_y - min_y),
            color,
            WIDTH,
            HEIGHT,
        );

        if rotation.abs() > 0.01 {
            let center = aabb.center();
            let direction = Vec2::new(rotation.cos(), rotation.sin()) * 20.0;
            let start = Vec2::new(center.x, HEIGHT as f32 - center.y);
            let end = Vec2::new(
                center.x + direction.x,
                HEIGHT as f32 - (center.y + direction.y),
            );
            draw_line(frame, start, end, palette::YELLOW, WIDTH, HEIGHT);
        }
    }
}

fn body_color(body_type: BodyType, is_circle: bool) -> Color {
    match (body_type, is_circle) {
        (BodyType::Static, true) => [0x60, 0x60, 0x60, 0xff],
        (BodyType::Static, false) => [0x80, 0x80, 0x80, 0xff],
        (BodyType::Dynamic, true) => [0x40, 0xa0, 0xff, 0xff],
        (BodyType::Dynamic, false) => [0xff, 0x60, 0x60, 0xff],
        (BodyType::Kinematic, _) => [0xff, 0xa0, 0x40, 0xff],
    }
}

impl App for Demo {
    fn update(&mut self, dt: f32, input: &Input) {
        if input.mouse_pressed(ShimMouseButton::Left)
            && let Some((x, y)) = input.cursor()
        {
            self.spawn_ball_at(x, y);
        }
        let _ = input.key_pressed(ShimKeyCode::Escape); // Esc handled by the shim itself.
        self.physics_world.step(dt);
    }

    fn render(&self, frame: &mut [u8]) {
        clear(frame, palette::DARK_BLUE_BG);

        for body in self.physics_world.bodies() {
            match &body.shape {
                CollisionShape::Circle(circle) => {
                    Self::draw_body_circle(
                        frame,
                        body.position + circle.center,
                        circle.radius,
                        body.body_type(),
                    );
                },
                CollisionShape::Aabb(aabb) => {
                    Self::draw_body_aabb(
                        frame,
                        aabb.translate(body.position),
                        body.rotation,
                        body.body_type(),
                    );
                },
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run(
        WindowConfig {
            title: "Physics Engine - Bouncing Balls",
            width: WIDTH,
            height: HEIGHT,
            ..WindowConfig::default()
        },
        Demo::new(),
    )
}
