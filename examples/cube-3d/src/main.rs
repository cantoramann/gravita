//! Spinning colored cube: smoke test for `gravita-renderer-3d`.
//!
//! Controls:
//! - `Esc` to quit
//! - Arrow keys to orbit the camera around the cube

use gravita_math::{Quat, Transform3D, Vec3};
use gravita_renderer_3d::{
    App3D, Camera, Input, Instance, Mesh, MeshHandle, Renderer3D, ShimKeyCode, WindowConfig, run,
};

struct CubeDemo {
    cube: Option<MeshHandle>,
    plane: Option<MeshHandle>,
    angle: f32,
    camera_yaw: f32,
    camera_pitch: f32,
    camera_distance: f32,
}

impl CubeDemo {
    fn new() -> Self {
        Self {
            cube: None,
            plane: None,
            angle: 0.0,
            camera_yaw: 0.6,
            camera_pitch: 0.5,
            camera_distance: 6.0,
        }
    }

    fn camera(&self, aspect: f32) -> Camera {
        // Orbit camera: spherical coords around the origin.
        let (sy, cy) = self.camera_yaw.sin_cos();
        let (sp, cp) = self.camera_pitch.sin_cos();
        let eye = Vec3::new(
            self.camera_distance * cp * sy,
            self.camera_distance * sp,
            self.camera_distance * cp * cy,
        );
        Camera {
            eye,
            target: Vec3::ZERO,
            up: Vec3::Y,
            fov_y: std::f32::consts::FRAC_PI_3,
            aspect,
            near: 0.1,
            far: 100.0,
        }
    }
}

impl App3D for CubeDemo {
    fn setup(&mut self, renderer: &mut Renderer3D) {
        self.cube = Some(renderer.register_mesh("cube", &Mesh::cube()));
        self.plane = Some(renderer.register_mesh("ground", &Mesh::plane(8.0, [0.18, 0.20, 0.24])));
    }

    fn update(&mut self, dt: f32, input: &Input) {
        self.angle += dt;
        let rot_speed = 1.2;
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
                (self.camera_pitch - rot_speed * dt).max(-std::f32::consts::FRAC_PI_2 + 0.05);
        }
    }

    fn render(&self, renderer: &mut Renderer3D) {
        let Some(cube) = self.cube else { return };
        let Some(plane) = self.plane else { return };

        let cube_transform = Transform3D::IDENTITY
            .with_position(Vec3::new(0.0, 1.0, 0.0))
            .with_rotation(Quat::from_axis_angle(
                Vec3::new(0.3, 1.0, 0.2).normalize(),
                self.angle,
            ));

        let plane_transform = Transform3D::IDENTITY.with_position(Vec3::new(0.0, -1.0, 0.0));

        let instances = [
            Instance {
                mesh: cube,
                transform: cube_transform,
                tint: [1.0, 1.0, 1.0, 1.0],
            },
            Instance {
                mesh: plane,
                transform: plane_transform,
                tint: [1.0, 1.0, 1.0, 1.0],
            },
        ];

        let camera = self.camera(renderer.aspect());
        // Surface errors (`Outdated`, `Lost`) are recovered on the next frame
        // by the runner's resize logic; nothing to do here.
        let _ = renderer.render(&camera, &instances);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run(
        WindowConfig {
            title: "Gravita — Spinning Cube",
            width: 1280,
            height: 720,
            ..WindowConfig::default()
        },
        CubeDemo::new(),
    )
}
