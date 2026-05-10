//! Bouncing-spheres 3D demo: physics-3d driving renderer-3d.
//!
//! Controls:
//! - `Esc` to quit
//! - Arrow keys to orbit the camera
//! - `Space` to drop a fresh sphere from above

use gravita_math::{Aabb3, Quat, Sphere, Transform3D, Vec3};
use gravita_physics_3d::{BodyType, CollisionShape, PhysicsWorld, RigidBody};
use gravita_renderer_3d::{
    App3D, Camera, Input, Instance, Mesh, MeshHandle, Renderer3D, ShimKeyCode, WindowConfig, run,
};

const FIXED_TIMESTEP: f32 = 1.0 / 60.0;

struct Demo {
    world: PhysicsWorld,
    sphere_mesh: Option<MeshHandle>,
    plane_mesh: Option<MeshHandle>,
    floor_id: usize,
    sphere_ids: Vec<usize>,
    camera_yaw: f32,
    camera_pitch: f32,
    camera_distance: f32,
}

impl Demo {
    fn new() -> Self {
        let mut world = PhysicsWorld::new();

        // Static floor: thick AABB centred slightly below origin.
        let floor = RigidBody::new(
            0,
            CollisionShape::Aabb(Aabb3::from_center_size(
                Vec3::new(0.0, -0.5, 0.0),
                Vec3::new(40.0, 1.0, 40.0),
            )),
        )
        .with_type(BodyType::Static)
        .with_restitution(0.4)
        .with_friction(0.6);
        let floor_id = world.add_body(floor);

        // A few spheres at varying heights and densities.
        let mut sphere_ids = Vec::new();
        for i in 0..6 {
            let x = (i as f32 - 2.5) * 1.5;
            let y = 6.0 + (i as f32) * 0.7;
            let z = ((i % 3) as f32 - 1.0) * 1.2;
            let r = 0.4 + (i as f32 % 3.0) * 0.15;
            let body = RigidBody::new(0, CollisionShape::Sphere(Sphere::new(Vec3::ZERO, r)))
                .with_position(Vec3::new(x, y, z))
                .with_density(1.0)
                .with_restitution(0.75)
                .with_friction(0.3);
            sphere_ids.push(world.add_body(body));
        }

        Self {
            world,
            sphere_mesh: None,
            plane_mesh: None,
            floor_id,
            sphere_ids,
            camera_yaw: 0.7,
            camera_pitch: 0.45,
            camera_distance: 14.0,
        }
    }

    fn camera(&self, aspect: f32) -> Camera {
        let (sy, cy) = self.camera_yaw.sin_cos();
        let (sp, cp) = self.camera_pitch.sin_cos();
        let eye = Vec3::new(
            self.camera_distance * cp * sy,
            self.camera_distance * sp + 1.5,
            self.camera_distance * cp * cy,
        );
        Camera {
            eye,
            target: Vec3::new(0.0, 1.0, 0.0),
            up: Vec3::Y,
            fov_y: std::f32::consts::FRAC_PI_3,
            aspect,
            near: 0.1,
            far: 200.0,
        }
    }

    fn spawn_sphere(&mut self) {
        let r = 0.5;
        let body = RigidBody::new(0, CollisionShape::Sphere(Sphere::new(Vec3::ZERO, r)))
            .with_position(Vec3::new(0.0, 10.0, 0.0))
            .with_velocity(Vec3::new(
                (self.sphere_ids.len() as f32 * 0.3).sin() * 2.0,
                0.0,
                (self.sphere_ids.len() as f32 * 0.3).cos() * 2.0,
            ))
            .with_density(1.0)
            .with_restitution(0.8)
            .with_friction(0.25);
        let id = self.world.add_body(body);
        self.sphere_ids.push(id);
    }
}

impl App3D for Demo {
    fn setup(&mut self, renderer: &mut Renderer3D) {
        // Sphere mesh — middling tessellation, off-white so per-vertex Lambert
        // shading reads clearly from any orbit angle.
        self.sphere_mesh = Some(
            renderer.register_mesh("sphere", &Mesh::uv_sphere(1.0, 24, 16, [0.85, 0.85, 0.92])),
        );
        self.plane_mesh =
            Some(renderer.register_mesh("floor", &Mesh::plane(40.0, [0.18, 0.20, 0.24])));
    }

    fn update(&mut self, dt: f32, input: &Input) {
        let rot_speed = 1.4;
        if input.key_held(ShimKeyCode::ArrowLeft) {
            self.camera_yaw -= rot_speed * dt;
        }
        if input.key_held(ShimKeyCode::ArrowRight) {
            self.camera_yaw += rot_speed * dt;
        }
        if input.key_held(ShimKeyCode::ArrowUp) {
            self.camera_pitch =
                (self.camera_pitch + rot_speed * dt).min(std::f32::consts::FRAC_PI_2 - 0.05);
        }
        if input.key_held(ShimKeyCode::ArrowDown) {
            self.camera_pitch =
                (self.camera_pitch - rot_speed * dt).max(0.05 - std::f32::consts::FRAC_PI_2);
        }
        if input.key_pressed(ShimKeyCode::Space) {
            self.spawn_sphere();
        }

        // Drive the sim at a fixed step regardless of the renderer's dt.
        self.world.step(FIXED_TIMESTEP.min(dt.max(1e-4)));
    }

    fn render(&self, renderer: &mut Renderer3D) {
        let Some(sphere_mesh) = self.sphere_mesh else {
            return;
        };
        let Some(plane_mesh) = self.plane_mesh else {
            return;
        };

        let mut instances: Vec<Instance> = Vec::with_capacity(self.sphere_ids.len() + 1);

        // Floor (mesh is already a horizontal plane at Y=0; offset to sit on
        // top of the physics floor block).
        instances.push(Instance {
            mesh: plane_mesh,
            transform: Transform3D::from_position(Vec3::new(0.0, 0.0, 0.0)),
            tint: [1.0, 1.0, 1.0, 1.0],
        });
        let _ = self.floor_id; // physics-only, not drawn directly

        // One tinted sphere per dynamic body, scaled to its radius.
        for (i, &id) in self.sphere_ids.iter().enumerate() {
            let body = &self.world.bodies()[id];
            let r = match &body.shape {
                CollisionShape::Sphere(s) => s.radius,
                CollisionShape::Aabb(_) | CollisionShape::Obb(_) => 0.5,
            };
            let tint = palette(i);
            instances.push(Instance {
                mesh: sphere_mesh,
                transform: Transform3D::IDENTITY
                    .with_position(body.position)
                    .with_rotation(body.rotation)
                    .with_scale(Vec3::splat(r)),
                tint,
            });
        }

        let camera = self.camera(renderer.aspect());
        let _ = renderer.render(&camera, &instances);
    }
}

fn palette(i: usize) -> [f32; 4] {
    // Distinct hues cycled by index. Hand-tuned six-stop ramp.
    const COLORS: &[[f32; 4]] = &[
        [0.95, 0.35, 0.35, 1.0],
        [0.95, 0.75, 0.30, 1.0],
        [0.50, 0.90, 0.45, 1.0],
        [0.30, 0.85, 0.95, 1.0],
        [0.45, 0.40, 0.95, 1.0],
        [0.95, 0.45, 0.85, 1.0],
    ];
    COLORS[i % COLORS.len()]
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Ensure we depend on Quat in scope so the example doc reflects the path
    // through the runtime even if the visual doesn't currently use rotation.
    let _: Quat = Quat::IDENTITY;
    run(
        WindowConfig {
            title: "Gravita — Bouncing Spheres (3D)",
            width: 1280,
            height: 720,
            ..WindowConfig::default()
        },
        Demo::new(),
    )
}
