//! # Froggy Jump
//!
//! A minimal Doodle Jump-style game built with Gravita, running in the browser via WASM.
//!
//! ## Controls
//! - **Left/Right Arrow** or **A/D**: Move horizontally
//! - **Space**: Restart when game over
//!
//! ## Build & Run
//! ```bash
//! cd examples/froggy-jump
//! wasm-pack build --target web
//! python3 -m http.server 8080
//! # Open http://localhost:8080
//! ```

use std::{cell::RefCell, rc::Rc};

use gravita_math::Vec2;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, KeyboardEvent};

// ═══════════════════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════════════════

const CANVAS_WIDTH: f64 = 400.0;
const CANVAS_HEIGHT: f64 = 600.0;

const PLAYER_SIZE: f64 = 30.0;
const PLAYER_JUMP_VELOCITY: f32 = -15.0;
const GRAVITY: f32 = 0.5;
const MOVE_SPEED: f32 = 6.0;

const PLATFORM_WIDTH: f64 = 70.0;
const PLATFORM_HEIGHT: f64 = 15.0;
const PLATFORM_COUNT: usize = 8;
const PLATFORM_SPACING: f32 = 80.0;

// ═══════════════════════════════════════════════════════════════════════════════
// Game State
// ═══════════════════════════════════════════════════════════════════════════════

struct Player {
    pos: Vec2,
    vel: Vec2,
}

struct Platform {
    pos: Vec2,
    width: f32,
}

struct Game {
    player: Player,
    platforms: Vec<Platform>,
    score: i32,
    high_score: i32,
    camera_y: f32,
    game_over: bool,
    move_left: bool,
    move_right: bool,
}

impl Game {
    fn new() -> Self {
        let mut game = Self {
            player: Player {
                pos: Vec2::new(CANVAS_WIDTH as f32 / 2.0, CANVAS_HEIGHT as f32 - 100.0),
                vel: Vec2::new(0.0, PLAYER_JUMP_VELOCITY),
            },
            platforms: Vec::new(),
            score: 0,
            high_score: 0,
            camera_y: 0.0,
            game_over: false,
            move_left: false,
            move_right: false,
        };
        game.generate_initial_platforms();
        game
    }

    fn generate_initial_platforms(&mut self) {
        self.platforms.clear();

        // Starting platform directly under player
        self.platforms.push(Platform {
            pos: Vec2::new(
                CANVAS_WIDTH as f32 / 2.0 - PLATFORM_WIDTH as f32 / 2.0,
                CANVAS_HEIGHT as f32 - 50.0,
            ),
            width: PLATFORM_WIDTH as f32,
        });

        // Generate platforms going up
        for i in 1..PLATFORM_COUNT {
            let x = pseudo_random(i as u32) * (CANVAS_WIDTH as f32 - PLATFORM_WIDTH as f32);
            let y = CANVAS_HEIGHT as f32 - 50.0 - (i as f32 * PLATFORM_SPACING);
            self.platforms.push(Platform {
                pos: Vec2::new(x, y),
                width: PLATFORM_WIDTH as f32,
            });
        }
    }

    fn reset(&mut self) {
        if self.score > self.high_score {
            self.high_score = self.score;
        }
        self.player.pos = Vec2::new(CANVAS_WIDTH as f32 / 2.0, CANVAS_HEIGHT as f32 - 100.0);
        self.player.vel = Vec2::new(0.0, PLAYER_JUMP_VELOCITY);
        self.score = 0;
        self.camera_y = 0.0;
        self.game_over = false;
        self.generate_initial_platforms();
    }

    fn update(&mut self) {
        if self.game_over {
            return;
        }

        // Horizontal movement
        if self.move_left {
            self.player.vel.x = -MOVE_SPEED;
        } else if self.move_right {
            self.player.vel.x = MOVE_SPEED;
        } else {
            self.player.vel.x *= 0.9; // Friction
        }

        // Apply gravity
        self.player.vel.y += GRAVITY;

        // Update position
        self.player.pos += self.player.vel;

        // Screen wrapping
        if self.player.pos.x < -PLAYER_SIZE as f32 {
            self.player.pos.x = CANVAS_WIDTH as f32;
        } else if self.player.pos.x > CANVAS_WIDTH as f32 {
            self.player.pos.x = -PLAYER_SIZE as f32;
        }

        // Platform collision (only when falling)
        if self.player.vel.y > 0.0 {
            let player_bottom = self.player.pos.y + PLAYER_SIZE as f32;
            let player_left = self.player.pos.x;
            let player_right = self.player.pos.x + PLAYER_SIZE as f32;

            for platform in &self.platforms {
                let plat_top = platform.pos.y;
                let plat_left = platform.pos.x;
                let plat_right = platform.pos.x + platform.width;

                // Check if player is landing on platform
                if player_bottom >= plat_top
                    && player_bottom <= plat_top + PLATFORM_HEIGHT as f32
                    && player_right > plat_left
                    && player_left < plat_right
                {
                    self.player.vel.y = PLAYER_JUMP_VELOCITY;
                    self.player.pos.y = plat_top - PLAYER_SIZE as f32;
                    break;
                }
            }
        }

        // Camera follows player (when going up)
        let screen_player_y = self.player.pos.y - self.camera_y;
        if screen_player_y < CANVAS_HEIGHT as f32 * 0.4 {
            let diff = CANVAS_HEIGHT as f32 * 0.4 - screen_player_y;
            self.camera_y -= diff;
            self.score = (-self.camera_y / 10.0) as i32;

            // Generate new platforms as we go up
            self.generate_new_platforms();
        }

        // Game over if player falls below screen
        if self.player.pos.y - self.camera_y > CANVAS_HEIGHT as f32 + PLAYER_SIZE as f32 {
            self.game_over = true;
        }
    }

    fn generate_new_platforms(&mut self) {
        // Remove platforms that are below the screen
        self.platforms
            .retain(|p| p.pos.y - self.camera_y < CANVAS_HEIGHT as f32 + 100.0);

        // Find the highest platform
        let highest_y = self
            .platforms
            .iter()
            .map(|p| p.pos.y)
            .fold(f32::INFINITY, f32::min);

        // Generate new platforms above
        let mut y = highest_y - PLATFORM_SPACING;
        while y > self.camera_y - 100.0 {
            let seed = ((-y * 1000.0) as u32).wrapping_add(self.score as u32);
            let x = pseudo_random(seed) * (CANVAS_WIDTH as f32 - PLATFORM_WIDTH as f32);
            self.platforms.push(Platform {
                pos: Vec2::new(x, y),
                width: PLATFORM_WIDTH as f32,
            });
            y -= PLATFORM_SPACING;
        }
    }

    fn render(&self, ctx: &CanvasRenderingContext2d) {
        // Clear canvas with sky gradient
        ctx.set_fill_style_str("#1a1a2e");
        ctx.fill_rect(0.0, 0.0, CANVAS_WIDTH, CANVAS_HEIGHT);

        // Draw stars (background)
        ctx.set_fill_style_str("#ffffff");
        for i in 0..30 {
            let seed = i * 12345;
            let x = pseudo_random(seed) * CANVAS_WIDTH as f32;
            let y = (pseudo_random(seed + 1) * CANVAS_HEIGHT as f32 + self.camera_y * 0.1)
                % CANVAS_HEIGHT as f32;
            let size = 1.0 + pseudo_random(seed + 2) * 2.0;
            ctx.fill_rect(x as f64, y as f64, size as f64, size as f64);
        }

        // Draw platforms
        for platform in &self.platforms {
            let screen_y = platform.pos.y - self.camera_y;
            if screen_y > -PLATFORM_HEIGHT as f32
                && screen_y < (CANVAS_HEIGHT + PLATFORM_HEIGHT) as f32
            {
                // Platform shadow
                ctx.set_fill_style_str("#16213e");
                ctx.fill_rect(
                    platform.pos.x as f64 + 3.0,
                    screen_y as f64 + 3.0,
                    platform.width as f64,
                    PLATFORM_HEIGHT,
                );

                // Platform
                ctx.set_fill_style_str("#0f3460");
                ctx.fill_rect(
                    platform.pos.x as f64,
                    screen_y as f64,
                    platform.width as f64,
                    PLATFORM_HEIGHT,
                );

                // Platform highlight
                ctx.set_fill_style_str("#e94560");
                ctx.fill_rect(
                    platform.pos.x as f64,
                    screen_y as f64,
                    platform.width as f64,
                    3.0,
                );
            }
        }

        // Draw player
        let player_screen_y = self.player.pos.y - self.camera_y;

        // Player shadow
        ctx.set_fill_style_str("rgba(0,0,0,0.3)");
        ctx.begin_path();
        let _ = ctx.ellipse(
            self.player.pos.x as f64 + PLAYER_SIZE / 2.0,
            player_screen_y as f64 + PLAYER_SIZE + 5.0,
            PLAYER_SIZE / 2.0,
            5.0,
            0.0,
            0.0,
            std::f64::consts::PI * 2.0,
        );
        ctx.fill();

        // Player body (cute blob)
        ctx.set_fill_style_str("#00ff88");
        ctx.begin_path();
        let _ = ctx.arc(
            self.player.pos.x as f64 + PLAYER_SIZE / 2.0,
            player_screen_y as f64 + PLAYER_SIZE / 2.0,
            PLAYER_SIZE / 2.0,
            0.0,
            std::f64::consts::PI * 2.0,
        );
        ctx.fill();

        // Eyes
        ctx.set_fill_style_str("#ffffff");
        ctx.begin_path();
        let _ = ctx.arc(
            self.player.pos.x as f64 + PLAYER_SIZE / 2.0 - 6.0,
            player_screen_y as f64 + PLAYER_SIZE / 2.0 - 3.0,
            5.0,
            0.0,
            std::f64::consts::PI * 2.0,
        );
        ctx.fill();
        ctx.begin_path();
        let _ = ctx.arc(
            self.player.pos.x as f64 + PLAYER_SIZE / 2.0 + 6.0,
            player_screen_y as f64 + PLAYER_SIZE / 2.0 - 3.0,
            5.0,
            0.0,
            std::f64::consts::PI * 2.0,
        );
        ctx.fill();

        // Pupils
        ctx.set_fill_style_str("#000000");
        ctx.begin_path();
        let _ = ctx.arc(
            self.player.pos.x as f64 + PLAYER_SIZE / 2.0 - 5.0,
            player_screen_y as f64 + PLAYER_SIZE / 2.0 - 2.0,
            2.5,
            0.0,
            std::f64::consts::PI * 2.0,
        );
        ctx.fill();
        ctx.begin_path();
        let _ = ctx.arc(
            self.player.pos.x as f64 + PLAYER_SIZE / 2.0 + 7.0,
            player_screen_y as f64 + PLAYER_SIZE / 2.0 - 2.0,
            2.5,
            0.0,
            std::f64::consts::PI * 2.0,
        );
        ctx.fill();

        // Draw score
        ctx.set_fill_style_str("#ffffff");
        ctx.set_font("bold 24px monospace");
        let _ = ctx.fill_text(&format!("Score: {}", self.score), 10.0, 30.0);
        ctx.set_font("16px monospace");
        let _ = ctx.fill_text(&format!("High: {}", self.high_score), 10.0, 55.0);

        // Game over overlay
        if self.game_over {
            ctx.set_fill_style_str("rgba(0,0,0,0.7)");
            ctx.fill_rect(0.0, 0.0, CANVAS_WIDTH, CANVAS_HEIGHT);

            ctx.set_fill_style_str("#e94560");
            ctx.set_font("bold 48px monospace");
            let _ = ctx.fill_text("GAME OVER", 55.0, CANVAS_HEIGHT / 2.0 - 30.0);

            ctx.set_fill_style_str("#ffffff");
            ctx.set_font("24px monospace");
            let _ = ctx.fill_text(
                &format!("Score: {}", self.score),
                140.0,
                CANVAS_HEIGHT / 2.0 + 20.0,
            );

            ctx.set_font("18px monospace");
            ctx.set_fill_style_str("#888888");
            let _ = ctx.fill_text("Press SPACE to restart", 85.0, CANVAS_HEIGHT / 2.0 + 60.0);
        }
    }

    fn key_down(&mut self, key: &str) {
        match key {
            "ArrowLeft" | "a" | "A" => self.move_left = true,
            "ArrowRight" | "d" | "D" => self.move_right = true,
            " " if self.game_over => self.reset(),
            _ => {},
        }
    }

    fn key_up(&mut self, key: &str) {
        match key {
            "ArrowLeft" | "a" | "A" => self.move_left = false,
            "ArrowRight" | "d" | "D" => self.move_right = false,
            _ => {},
        }
    }
}

// Simple pseudo-random for deterministic platform placement
fn pseudo_random(seed: u32) -> f32 {
    let x = seed.wrapping_mul(1103515245).wrapping_add(12345);
    ((x >> 16) & 0x7fff) as f32 / 32767.0
}

// ═══════════════════════════════════════════════════════════════════════════════
// WASM Entry Point
// ═══════════════════════════════════════════════════════════════════════════════

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let window = web_sys::window().expect("no window");
    let document = window.document().expect("no document");

    // Create canvas
    let canvas = document
        .create_element("canvas")?
        .dyn_into::<HtmlCanvasElement>()?;
    canvas.set_width(CANVAS_WIDTH as u32);
    canvas.set_height(CANVAS_HEIGHT as u32);
    canvas.style().set_property("display", "block")?;
    canvas.style().set_property("margin", "20px auto")?;
    canvas.style().set_property("border", "2px solid #e94560")?;
    canvas.style().set_property("border-radius", "8px")?;

    document.body().unwrap().append_child(&canvas)?;

    // Add title
    let title = document.create_element("h1")?;
    title.set_text_content(Some("🐸 Froggy Jump"));
    title.set_attribute(
        "style",
        "text-align: center; color: #00ff88; font-family: monospace; margin: 20px;",
    )?;
    document
        .body()
        .unwrap()
        .insert_before(&title, Some(&canvas))?;

    // Add instructions
    let instructions = document.create_element("p")?;
    instructions.set_text_content(Some("Use ← → or A/D to move"));
    instructions.set_attribute(
        "style",
        "text-align: center; color: #888; font-family: monospace;",
    )?;
    document.body().unwrap().append_child(&instructions)?;

    // Style body
    document
        .body()
        .unwrap()
        .style()
        .set_property("background", "#0a0a0f")?;
    document
        .body()
        .unwrap()
        .style()
        .set_property("margin", "0")?;
    document
        .body()
        .unwrap()
        .style()
        .set_property("padding", "0")?;

    let ctx = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()?;

    let game = Rc::new(RefCell::new(Game::new()));

    // Keyboard handlers
    {
        let game = game.clone();
        let closure = Closure::<dyn FnMut(_)>::new(move |event: KeyboardEvent| {
            game.borrow_mut().key_down(&event.key());
            event.prevent_default();
        });
        document.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    {
        let game = game.clone();
        let closure = Closure::<dyn FnMut(_)>::new(move |event: KeyboardEvent| {
            game.borrow_mut().key_up(&event.key());
        });
        document.add_event_listener_with_callback("keyup", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // Game loop
    fn request_animation_frame(f: &Closure<dyn FnMut()>) {
        web_sys::window()
            .unwrap()
            .request_animation_frame(f.as_ref().unchecked_ref())
            .unwrap();
    }

    let f = Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));
    let g = f.clone();

    {
        let game = game.clone();
        *g.borrow_mut() = Some(Closure::new(move || {
            game.borrow_mut().update();
            game.borrow().render(&ctx);
            request_animation_frame(f.borrow().as_ref().unwrap());
        }));
    }

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}
