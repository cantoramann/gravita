# gravita-renderer-3d

`wgpu`-based 3D renderer + `winit` runner. Single dep for 3D examples — pulls in the GPU pipeline and the windowing event loop together.

If you only need 2D, use [`gravita-renderer`](../renderer) (CPU framebuffer, no GPU dependency).

## What it does

- **Instanced mesh rendering.** One shared mesh, many transforms; one `draw_indexed` per registered mesh per frame.
- **Per-vertex Lambert shading** against a directional light + ambient, configurable on the `Renderer3D`.
- **Right-handed, Y-up, forward = -Z** coordinate system.
- **Depth buffer** (`Depth32Float`, `Less`, write-on), **back-face culling** (CCW winding).
- **`App3D` trait + `run(WindowConfig, app)`** runner — same shape as `gravita_example_shim::App` but constructs a wgpu surface instead of a `pixels` framebuffer.
- **`ControlFlow::WaitUntil`** scheduling — no CPU pegging when idle.

## Quick example

```rust
use gravita_math::{Quat, Transform3D, Vec3};
use gravita_renderer_3d::{
    App3D, Camera, Input, Instance, Mesh, MeshHandle, Renderer3D, ShimKeyCode, WindowConfig, run,
};

struct Demo {
    cube: Option<MeshHandle>,
    t: f32,
}

impl App3D for Demo {
    fn setup(&mut self, r: &mut Renderer3D) {
        // Register a mesh once. The renderer keeps the GPU buffers alive for the
        // life of the app; the returned handle is what you reference in `Instance`.
        self.cube = Some(r.register_mesh("cube", &Mesh::cube()));
    }

    fn update(&mut self, dt: f32, _input: &Input) {
        self.t += dt;
    }

    fn render(&self, r: &mut Renderer3D) {
        let Some(cube) = self.cube else { return };
        let transform = Transform3D::IDENTITY
            .with_position(Vec3::new(0.0, 1.0, 0.0))
            .with_rotation(Quat::from_axis_angle(Vec3::Y, self.t));

        let camera = Camera {
            eye: Vec3::new(0.0, 2.0, 5.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            fov_y: std::f32::consts::FRAC_PI_3,
            aspect: r.aspect(),
            near: 0.1,
            far: 100.0,
        };

        let _ = r.render(&camera, &[Instance {
            mesh: cube,
            transform,
            tint: [1.0, 1.0, 1.0, 1.0],
        }]);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run(
        WindowConfig { title: "Spinning Cube", width: 1280, height: 720, ..Default::default() },
        Demo { cube: None, t: 0.0 },
    )
}
```

For a complete demo, see [`examples/cube-3d`](../../examples/cube-3d/) (just a spinning cube on a plane) and [`examples/spheres-3d`](../../examples/spheres-3d/) (full physics-3d driving the renderer).

## Built-in meshes

```rust
Mesh::cube();                          // 1×1×1, distinct color per face
Mesh::plane(40.0, [0.18, 0.20, 0.24]); // XZ plane at Y=0, facing +Y
Mesh::uv_sphere(1.0, 24, 16, [1.0; 3]);// radius, segments, rings, color
```

Or build your own `Mesh { vertices, indices }`.

## WGSL shader

The complete shader is at [`src/shader.wgsl`](src/shader.wgsl). It does:

```text
vertex stage:
    world_pos    = model * vec4(position, 1.0)
    world_normal = normalize((model * vec4(normal, 0.0)).xyz)
    clip_position = view_proj * world_pos
    color        = vertex_color * instance_tint

fragment stage:
    lambert = max(dot(normalize(world_normal), normalize(light_dir)), 0.0)
    color   = vertex_color * (ambient + vec3(lambert))
```

If you want PBR, shadows, or post-processing, fork the shader and the pipeline state — the rest of the renderer is structured to make that easy.

## License

MIT — see [../../LICENSE](../../LICENSE).
