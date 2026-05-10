use gravita_collections::Stickman;
use gravita_example_shim::{App, Input, ShimKeyCode, WindowConfig, run};
use gravita_math::Vec2;
use gravita_renderer::{clear, draw_line, palette};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

struct Demo {
    stickman: Stickman,
}

impl Demo {
    fn new() -> Self {
        // Ground 40 pixels above the bottom of the window.
        let ground_offset = 40.0;
        let base_y = HEIGHT as f32 - ground_offset;
        Self {
            stickman: Stickman::new(base_y, WIDTH as f32),
        }
    }
}

impl App for Demo {
    fn update(&mut self, dt: f32, input: &Input) {
        let mut dir = 0.0;
        if input.key_held(ShimKeyCode::ArrowLeft) || input.key_held(ShimKeyCode::KeyA) {
            dir -= 1.0;
        }
        if input.key_held(ShimKeyCode::ArrowRight) || input.key_held(ShimKeyCode::KeyD) {
            dir += 1.0;
        }
        self.stickman.set_move_direction(dir);

        if input.key_pressed(ShimKeyCode::Space) {
            self.stickman.jump();
        }

        self.stickman.update(dt, WIDTH as f32);
    }

    fn render(&self, frame: &mut [u8]) {
        clear(frame, palette::WHITE);
        let ground_y = self.stickman.ground_y().round();
        draw_line(
            frame,
            Vec2::new(0.0, ground_y),
            Vec2::new(WIDTH as f32, ground_y),
            palette::BLACK,
            WIDTH,
            HEIGHT,
        );
        self.stickman.render(frame, WIDTH, HEIGHT);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run(
        WindowConfig {
            title: "Stickman Walk Demo",
            width: WIDTH,
            height: HEIGHT,
            ..WindowConfig::default()
        },
        Demo::new(),
    )
}
