//! # GRAVITA TETRIS
//!
//! A visually stunning Tetris implementation featuring:
//! - Synthwave/vaporwave aesthetic with glowing neon blocks
//! - Smooth animations and particle effects
//! - Ghost piece preview
//! - Hold piece functionality
//! - Next piece queue
//! - Combo system with visual feedback
//! - Screen shake on hard drops and line clears

use std::time::Instant;

use gravita_math::{Vec2, lerp};
use gravita_renderer::{blend_pixel, text as renderer_text};
use pixels::{Pixels, SurfaceTexture};
use rand::Rng;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

// ============================================================================
// CONSTANTS
// ============================================================================

const WIDTH: u32 = 520;
const HEIGHT: u32 = 680;

const GRID_COLS: usize = 10;
const GRID_ROWS: usize = 20;
const CELL_SIZE: u32 = 28;
const GRID_OFFSET_X: u32 = 30;
const GRID_OFFSET_Y: u32 = 60;

// Timing (in seconds)
const BASE_DROP_INTERVAL: f32 = 0.8;
const SOFT_DROP_INTERVAL: f32 = 0.05;
const LOCK_DELAY: f32 = 0.5;
const LINE_CLEAR_ANIMATION_DURATION: f32 = 0.4;

// DAS (Delayed Auto Shift) for better controls
const DAS_DELAY: f32 = 0.15;
const DAS_REPEAT: f32 = 0.03;

// ============================================================================
// COLOR PALETTE - Synthwave / Vaporwave
// ============================================================================

#[allow(dead_code)]
mod palette {
    // Background - deep purple/blue gradient feel
    pub const BG_TOP: [u8; 4] = [0x0d, 0x0a, 0x1a, 0xff];
    pub const BG_BOTTOM: [u8; 4] = [0x1a, 0x0a, 0x2e, 0xff];
    pub const BG_GRID: [u8; 4] = [0x18, 0x10, 0x30, 0xff];
    pub const GRID_LINE: [u8; 4] = [0x30, 0x20, 0x50, 0xff];
    pub const GRID_BORDER: [u8; 4] = [0xff, 0x00, 0x80, 0xff]; // Hot pink border

    // Piece colors - Neon synthwave palette
    pub const CYAN: [u8; 4] = [0x00, 0xff, 0xf7, 0xff]; // Electric cyan (I)
    pub const YELLOW: [u8; 4] = [0xff, 0xf7, 0x00, 0xff]; // Bright yellow (O)
    pub const PURPLE: [u8; 4] = [0xbd, 0x00, 0xff, 0xff]; // Neon purple (T)
    pub const GREEN: [u8; 4] = [0x00, 0xff, 0x9f, 0xff]; // Mint green (S)
    pub const RED: [u8; 4] = [0xff, 0x00, 0x6e, 0xff]; // Hot pink/red (Z)
    pub const BLUE: [u8; 4] = [0x00, 0x6e, 0xff, 0xff]; // Electric blue (J)
    pub const ORANGE: [u8; 4] = [0xff, 0x9f, 0x00, 0xff]; // Sunset orange (L)

    // UI colors
    pub const TEXT_TITLE: [u8; 4] = [0xff, 0x00, 0x80, 0xff]; // Hot pink
    pub const TEXT_PRIMARY: [u8; 4] = [0xff, 0xff, 0xff, 0xff];
    pub const TEXT_SECONDARY: [u8; 4] = [0x80, 0x60, 0xa0, 0xff];
    pub const TEXT_ACCENT: [u8; 4] = [0x00, 0xff, 0xf7, 0xff]; // Cyan accent
    pub const GHOST: [u8; 4] = [0x60, 0x40, 0x80, 0x60];

    // Special effects
    pub const GLOW_PINK: [u8; 4] = [0xff, 0x00, 0x80, 0x40];
    pub const GLOW_CYAN: [u8; 4] = [0x00, 0xff, 0xf7, 0x40];
}

// ============================================================================
// TETROMINO DEFINITIONS
// ============================================================================

#[derive(Clone, Copy, PartialEq, Eq)]
enum TetrominoType {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

impl TetrominoType {
    fn color(&self) -> [u8; 4] {
        match self {
            TetrominoType::I => palette::CYAN,
            TetrominoType::O => palette::YELLOW,
            TetrominoType::T => palette::PURPLE,
            TetrominoType::S => palette::GREEN,
            TetrominoType::Z => palette::RED,
            TetrominoType::J => palette::BLUE,
            TetrominoType::L => palette::ORANGE,
        }
    }

    fn glow_color(&self) -> [u8; 4] {
        let c = self.color();
        [c[0], c[1], c[2], 0x30]
    }

    fn cells(&self, rotation: usize) -> [(i32, i32); 4] {
        let base = match self {
            TetrominoType::I => [
                [(0, 1), (1, 1), (2, 1), (3, 1)],
                [(2, 0), (2, 1), (2, 2), (2, 3)],
                [(0, 2), (1, 2), (2, 2), (3, 2)],
                [(1, 0), (1, 1), (1, 2), (1, 3)],
            ],
            TetrominoType::O => [
                [(1, 0), (2, 0), (1, 1), (2, 1)],
                [(1, 0), (2, 0), (1, 1), (2, 1)],
                [(1, 0), (2, 0), (1, 1), (2, 1)],
                [(1, 0), (2, 0), (1, 1), (2, 1)],
            ],
            TetrominoType::T => [
                [(1, 0), (0, 1), (1, 1), (2, 1)],
                [(1, 0), (1, 1), (2, 1), (1, 2)],
                [(0, 1), (1, 1), (2, 1), (1, 2)],
                [(1, 0), (0, 1), (1, 1), (1, 2)],
            ],
            TetrominoType::S => [
                [(1, 0), (2, 0), (0, 1), (1, 1)],
                [(1, 0), (1, 1), (2, 1), (2, 2)],
                [(1, 1), (2, 1), (0, 2), (1, 2)],
                [(0, 0), (0, 1), (1, 1), (1, 2)],
            ],
            TetrominoType::Z => [
                [(0, 0), (1, 0), (1, 1), (2, 1)],
                [(2, 0), (1, 1), (2, 1), (1, 2)],
                [(0, 1), (1, 1), (1, 2), (2, 2)],
                [(1, 0), (0, 1), (1, 1), (0, 2)],
            ],
            TetrominoType::J => [
                [(0, 0), (0, 1), (1, 1), (2, 1)],
                [(1, 0), (2, 0), (1, 1), (1, 2)],
                [(0, 1), (1, 1), (2, 1), (2, 2)],
                [(1, 0), (1, 1), (0, 2), (1, 2)],
            ],
            TetrominoType::L => [
                [(2, 0), (0, 1), (1, 1), (2, 1)],
                [(1, 0), (1, 1), (1, 2), (2, 2)],
                [(0, 1), (1, 1), (2, 1), (0, 2)],
                [(0, 0), (1, 0), (1, 1), (1, 2)],
            ],
        };
        base[rotation % 4]
    }

    fn random() -> Self {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..7) {
            0 => TetrominoType::I,
            1 => TetrominoType::O,
            2 => TetrominoType::T,
            3 => TetrominoType::S,
            4 => TetrominoType::Z,
            5 => TetrominoType::J,
            _ => TetrominoType::L,
        }
    }
}

#[derive(Clone)]
struct Tetromino {
    piece_type: TetrominoType,
    x: i32,
    y: i32,
    rotation: usize,
}

impl Tetromino {
    fn new(piece_type: TetrominoType) -> Self {
        Self {
            piece_type,
            x: 3,
            y: 0,
            rotation: 0,
        }
    }

    fn cells(&self) -> [(i32, i32); 4] {
        let base = self.piece_type.cells(self.rotation);
        [
            (base[0].0 + self.x, base[0].1 + self.y),
            (base[1].0 + self.x, base[1].1 + self.y),
            (base[2].0 + self.x, base[2].1 + self.y),
            (base[3].0 + self.x, base[3].1 + self.y),
        ]
    }
}

// ============================================================================
// PARTICLE SYSTEM
// ============================================================================

#[derive(Clone, Copy)]
enum ParticleType {
    Spark,
    Trail,
    Explosion,
}

struct Particle {
    pos: Vec2,
    vel: Vec2,
    life: f32,
    max_life: f32,
    color: [u8; 4],
    size: f32,
    particle_type: ParticleType,
}

impl Particle {
    fn new(pos: Vec2, vel: Vec2, life: f32, color: [u8; 4], particle_type: ParticleType) -> Self {
        Self {
            pos,
            vel,
            life,
            max_life: life,
            color,
            size: match particle_type {
                ParticleType::Spark => 2.0,
                ParticleType::Trail => 4.0,
                ParticleType::Explosion => 5.0,
            },
            particle_type,
        }
    }

    fn update(&mut self, dt: f32) {
        self.pos += self.vel * dt;
        match self.particle_type {
            ParticleType::Spark => {
                self.vel.y += 400.0 * dt;
                self.size *= 0.95;
            },
            ParticleType::Trail => {
                self.vel *= 0.95;
                self.size *= 0.98;
            },
            ParticleType::Explosion => {
                self.vel *= 0.92;
            },
        }
        self.life -= dt;
    }

    fn is_alive(&self) -> bool {
        self.life > 0.0 && self.size > 0.5
    }

    fn alpha(&self) -> f32 {
        (self.life / self.max_life).powf(0.5)
    }
}

// ============================================================================
// SCREEN EFFECTS
// ============================================================================

struct ScreenEffects {
    shake_intensity: f32,
    shake_timer: f32,
    flash_intensity: f32,
    flash_color: [u8; 4],
}

impl ScreenEffects {
    fn new() -> Self {
        Self {
            shake_intensity: 0.0,
            shake_timer: 0.0,
            flash_intensity: 0.0,
            flash_color: [255, 255, 255, 255],
        }
    }

    fn shake(&mut self, intensity: f32, duration: f32) {
        self.shake_intensity = intensity;
        self.shake_timer = duration;
    }

    fn flash(&mut self, color: [u8; 4], intensity: f32) {
        self.flash_color = color;
        self.flash_intensity = intensity;
    }

    fn update(&mut self, dt: f32) {
        if self.shake_timer > 0.0 {
            self.shake_timer -= dt;
            if self.shake_timer <= 0.0 {
                self.shake_intensity = 0.0;
            }
        }
        self.flash_intensity *= 0.85;
    }

    fn get_offset(&self) -> (i32, i32) {
        if self.shake_intensity > 0.0 {
            let mut rng = rand::thread_rng();
            let x = rng.gen_range(-self.shake_intensity..self.shake_intensity) as i32;
            let y = rng.gen_range(-self.shake_intensity..self.shake_intensity) as i32;
            (x, y)
        } else {
            (0, 0)
        }
    }
}

// ============================================================================
// GAME STATE
// ============================================================================

#[derive(Clone, Copy, PartialEq, Eq)]
enum GameState {
    Playing,
    LineClear,
    GameOver,
    Paused,
}

struct TetrisGame {
    grid: [[Option<[u8; 4]>; GRID_COLS]; GRID_ROWS],
    current_piece: Tetromino,
    next_pieces: Vec<TetrominoType>,
    held_piece: Option<TetrominoType>,
    can_hold: bool,

    state: GameState,
    score: u32,
    level: u32,
    lines_cleared: u32,
    combo: u32,
    last_clear_count: u32,

    drop_timer: f32,
    lock_timer: f32,
    is_soft_dropping: bool,

    // DAS (Delayed Auto Shift)
    das_direction: i32,
    das_timer: f32,
    das_charged: bool,

    clearing_lines: Vec<usize>,
    line_clear_timer: f32,

    particles: Vec<Particle>,
    effects: ScreenEffects,

    game_time: f32,
    last_update: Instant,
}

/// Per-call config for `TetrisGame::draw_cell` — keeps the function signature
/// at a reasonable arg count and gives each parameter a name at the call site.
#[derive(Debug, Copy, Clone)]
struct CellDraw {
    col: usize,
    row: usize,
    color: [u8; 4],
    flash: f32,
    shake_x: i32,
    shake_y: i32,
}

impl TetrisGame {
    fn new() -> Self {
        let mut game = Self {
            grid: [[None; GRID_COLS]; GRID_ROWS],
            current_piece: Tetromino::new(TetrominoType::random()),
            next_pieces: Vec::new(),
            held_piece: None,
            can_hold: true,

            state: GameState::Playing,
            score: 0,
            level: 1,
            lines_cleared: 0,
            combo: 0,
            last_clear_count: 0,

            drop_timer: 0.0,
            lock_timer: 0.0,
            is_soft_dropping: false,

            das_direction: 0,
            das_timer: 0.0,
            das_charged: false,

            clearing_lines: Vec::new(),
            line_clear_timer: 0.0,

            particles: Vec::new(),
            effects: ScreenEffects::new(),

            game_time: 0.0,
            last_update: Instant::now(),
        };

        // Fill next piece queue with 5 pieces
        for _ in 0..5 {
            game.next_pieces.push(TetrominoType::random());
        }

        game
    }

    fn reset(&mut self) {
        *self = Self::new();
    }

    fn drop_interval(&self) -> f32 {
        if self.is_soft_dropping {
            SOFT_DROP_INTERVAL
        } else {
            // Exponential speed increase
            (BASE_DROP_INTERVAL * 0.9_f32.powi(self.level as i32 - 1)).max(0.05)
        }
    }

    fn update(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_update).as_secs_f32().min(0.1);
        self.last_update = now;
        self.game_time += dt;

        // Update particles
        for p in &mut self.particles {
            p.update(dt);
        }
        self.particles.retain(|p| p.is_alive());

        // Update screen effects
        self.effects.update(dt);

        match self.state {
            GameState::Playing => self.update_playing(dt),
            GameState::LineClear => self.update_line_clear(dt),
            GameState::GameOver | GameState::Paused => {},
        }
    }

    fn update_playing(&mut self, dt: f32) {
        // Handle DAS
        if self.das_direction != 0 {
            self.das_timer += dt;
            if !self.das_charged && self.das_timer >= DAS_DELAY {
                self.das_charged = true;
                self.das_timer = 0.0;
            }
            if self.das_charged && self.das_timer >= DAS_REPEAT {
                self.das_timer = 0.0;
                self.try_move(self.das_direction, 0);
            }
        }

        self.drop_timer += dt;

        if self.drop_timer >= self.drop_interval() {
            self.drop_timer = 0.0;
            if !self.try_move(0, 1) {
                self.lock_timer += self.drop_interval();
            }
        }

        // Check if piece should lock
        if !self.can_move(0, 1) {
            self.lock_timer += dt;
            if self.lock_timer >= LOCK_DELAY {
                self.lock_piece();
            }
        } else {
            self.lock_timer = 0.0;
        }
    }

    fn update_line_clear(&mut self, dt: f32) {
        self.line_clear_timer += dt;

        if self.line_clear_timer >= LINE_CLEAR_ANIMATION_DURATION {
            // Remove ALL cleared lines at once
            // Sort in reverse order so we remove from bottom to top
            let mut sorted_lines = self.clearing_lines.clone();
            sorted_lines.sort_by(|a, b| b.cmp(a));

            for &row in &sorted_lines {
                // Shift all rows above this one down
                for r in (1..=row).rev() {
                    self.grid[r] = self.grid[r - 1];
                }
                self.grid[0] = [None; GRID_COLS];
            }

            // Update score based on number of lines cleared
            let lines = self.clearing_lines.len() as u32;
            self.last_clear_count = lines;

            let base_score = match lines {
                1 => 100,
                2 => 300,
                3 => 500,
                4 => 800, // TETRIS!
                _ => lines * 200,
            };

            self.score += base_score * self.level * (self.combo + 1);
            self.lines_cleared += lines;
            self.combo += 1;

            // Screen effects based on clear type
            if lines >= 4 {
                self.effects.shake(8.0, 0.3);
                self.effects.flash(palette::CYAN, 0.5);
            } else if lines >= 2 {
                self.effects.shake(4.0, 0.15);
            }

            // Level up every 10 lines
            self.level = (self.lines_cleared / 10) + 1;

            self.clearing_lines.clear();
            self.line_clear_timer = 0.0;
            self.spawn_piece();
            self.state = GameState::Playing;
        }
    }

    fn can_move(&self, dx: i32, dy: i32) -> bool {
        let mut test = self.current_piece.clone();
        test.x += dx;
        test.y += dy;
        self.is_valid_position(&test)
    }

    fn is_valid_position(&self, piece: &Tetromino) -> bool {
        for (x, y) in piece.cells() {
            if x < 0 || x >= GRID_COLS as i32 || y >= GRID_ROWS as i32 {
                return false;
            }
            if y >= 0 && self.grid[y as usize][x as usize].is_some() {
                return false;
            }
        }
        true
    }

    fn try_move(&mut self, dx: i32, dy: i32) -> bool {
        if self.can_move(dx, dy) {
            self.current_piece.x += dx;
            self.current_piece.y += dy;
            true
        } else {
            false
        }
    }

    fn try_rotate(&mut self, clockwise: bool) {
        let old_rotation = self.current_piece.rotation;
        if clockwise {
            self.current_piece.rotation = (self.current_piece.rotation + 1) % 4;
        } else {
            self.current_piece.rotation = (self.current_piece.rotation + 3) % 4;
        }

        // SRS-like wall kick attempts
        let kicks = [
            (0, 0),
            (-1, 0),
            (1, 0),
            (0, -1),
            (-1, -1),
            (1, -1),
            (-2, 0),
            (2, 0),
        ];
        for (dx, dy) in kicks {
            let mut test = self.current_piece.clone();
            test.x += dx;
            test.y += dy;
            if self.is_valid_position(&test) {
                self.current_piece = test;
                return;
            }
        }

        // Revert if no kick worked
        self.current_piece.rotation = old_rotation;
    }

    fn hard_drop(&mut self) {
        let mut drop_distance = 0;
        let start_y = self.current_piece.y;

        while self.try_move(0, 1) {
            drop_distance += 1;
        }

        self.score += drop_distance * 2;

        // Spawn trail particles along drop path
        let color = self.current_piece.piece_type.color();
        for (x, _) in self.current_piece.cells() {
            let px = GRID_OFFSET_X as f32 + x as f32 * CELL_SIZE as f32 + CELL_SIZE as f32 / 2.0;
            for dy in 0..(drop_distance as i32).min(8) {
                let py = GRID_OFFSET_Y as f32
                    + (start_y + dy) as f32 * CELL_SIZE as f32
                    + CELL_SIZE as f32 / 2.0;
                self.spawn_trail_particle(Vec2::new(px, py), color);
            }
        }

        // Screen shake on hard drop
        self.effects.shake(3.0, 0.1);

        self.lock_piece();
    }

    fn ghost_y(&self) -> i32 {
        let mut ghost = self.current_piece.clone();
        while {
            ghost.y += 1;
            self.is_valid_position(&ghost)
        } {}
        ghost.y - 1
    }

    fn lock_piece(&mut self) {
        let color = self.current_piece.piece_type.color();
        for (x, y) in self.current_piece.cells() {
            if y >= 0 && y < GRID_ROWS as i32 && x >= 0 && x < GRID_COLS as i32 {
                self.grid[y as usize][x as usize] = Some(color);

                // Spawn particles at lock position
                let px =
                    GRID_OFFSET_X as f32 + x as f32 * CELL_SIZE as f32 + CELL_SIZE as f32 / 2.0;
                let py =
                    GRID_OFFSET_Y as f32 + y as f32 * CELL_SIZE as f32 + CELL_SIZE as f32 / 2.0;
                self.spawn_lock_particles(Vec2::new(px, py), color);
            }
        }

        self.can_hold = true;
        self.check_lines();
    }

    fn spawn_lock_particles(&mut self, pos: Vec2, color: [u8; 4]) {
        let mut rng = rand::thread_rng();
        for _ in 0..3 {
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let speed = rng.gen_range(40.0..100.0);
            let vel = Vec2::new(angle.cos() * speed, angle.sin() * speed - 30.0);
            self.particles.push(Particle::new(
                pos,
                vel,
                rng.gen_range(0.2..0.4),
                color,
                ParticleType::Spark,
            ));
        }
    }

    fn spawn_trail_particle(&mut self, pos: Vec2, color: [u8; 4]) {
        let mut rng = rand::thread_rng();
        let vel = Vec2::new(rng.gen_range(-10.0..10.0), rng.gen_range(-20.0..0.0));
        self.particles.push(Particle::new(
            pos,
            vel,
            rng.gen_range(0.1..0.2),
            color,
            ParticleType::Trail,
        ));
    }

    fn check_lines(&mut self) {
        self.clearing_lines.clear();

        // Find ALL complete lines
        for row in 0..GRID_ROWS {
            if self.grid[row].iter().all(|c| c.is_some()) {
                self.clearing_lines.push(row);
            }
        }

        if !self.clearing_lines.is_empty() {
            self.state = GameState::LineClear;
            self.line_clear_timer = 0.0;

            // Collect particle spawn data for ALL clearing lines
            let mut particle_data: Vec<(Vec2, [u8; 4])> = Vec::new();
            for &row in &self.clearing_lines {
                for col in 0..GRID_COLS {
                    if let Some(color) = self.grid[row][col] {
                        let px = GRID_OFFSET_X as f32
                            + col as f32 * CELL_SIZE as f32
                            + CELL_SIZE as f32 / 2.0;
                        let py = GRID_OFFSET_Y as f32
                            + row as f32 * CELL_SIZE as f32
                            + CELL_SIZE as f32 / 2.0;
                        particle_data.push((Vec2::new(px, py), color));
                    }
                }
            }

            // Spawn explosion particles for each cell in cleared lines
            for (pos, color) in particle_data {
                self.spawn_line_clear_particles(pos, color);
            }

            // Extra effects for multi-line clears
            let lines = self.clearing_lines.len();
            if lines >= 4 {
                // TETRIS! Extra particles
                for _ in 0..50 {
                    let mut rng = rand::thread_rng();
                    let x = GRID_OFFSET_X as f32
                        + rng.gen_range(0.0..GRID_COLS as f32 * CELL_SIZE as f32);
                    let y = GRID_OFFSET_Y as f32
                        + rng.gen_range(0.0..GRID_ROWS as f32 * CELL_SIZE as f32);
                    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                    let speed = rng.gen_range(100.0..300.0);
                    let vel = Vec2::new(angle.cos() * speed, angle.sin() * speed);
                    self.particles.push(Particle::new(
                        Vec2::new(x, y),
                        vel,
                        rng.gen_range(0.5..1.0),
                        palette::CYAN,
                        ParticleType::Explosion,
                    ));
                }
            }
        } else {
            self.combo = 0;
            self.spawn_piece();
        }
    }

    fn spawn_line_clear_particles(&mut self, pos: Vec2, color: [u8; 4]) {
        let mut rng = rand::thread_rng();
        for _ in 0..8 {
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let speed = rng.gen_range(100.0..250.0);
            let vel = Vec2::new(angle.cos() * speed, angle.sin() * speed);
            self.particles.push(Particle::new(
                pos,
                vel,
                rng.gen_range(0.3..0.6),
                color,
                ParticleType::Explosion,
            ));
        }
    }

    fn spawn_piece(&mut self) {
        let next_type = self.next_pieces.remove(0);
        self.next_pieces.push(TetrominoType::random());
        self.current_piece = Tetromino::new(next_type);
        self.drop_timer = 0.0;
        self.lock_timer = 0.0;

        if !self.is_valid_position(&self.current_piece) {
            self.state = GameState::GameOver;
            self.effects.shake(10.0, 0.5);
        }
    }

    fn hold_piece(&mut self) {
        if !self.can_hold {
            return;
        }

        let current_type = self.current_piece.piece_type;
        if let Some(held) = self.held_piece {
            self.current_piece = Tetromino::new(held);
        } else {
            self.spawn_piece();
        }
        self.held_piece = Some(current_type);
        self.can_hold = false;
        self.drop_timer = 0.0;
        self.lock_timer = 0.0;
    }

    fn start_das(&mut self, direction: i32) {
        if self.das_direction != direction {
            self.das_direction = direction;
            self.das_timer = 0.0;
            self.das_charged = false;
            self.try_move(direction, 0);
        }
    }

    fn stop_das(&mut self, direction: i32) {
        if self.das_direction == direction {
            self.das_direction = 0;
            self.das_charged = false;
        }
    }

    fn handle_input(&mut self, key: KeyCode) {
        match self.state {
            GameState::Playing => match key {
                KeyCode::ArrowLeft | KeyCode::KeyA => {
                    self.start_das(-1);
                },
                KeyCode::ArrowRight | KeyCode::KeyD => {
                    self.start_das(1);
                },
                KeyCode::ArrowDown | KeyCode::KeyS => {
                    self.is_soft_dropping = true;
                },
                KeyCode::ArrowUp | KeyCode::KeyW | KeyCode::KeyX => {
                    self.try_rotate(true);
                },
                KeyCode::KeyZ | KeyCode::ControlLeft => {
                    self.try_rotate(false);
                },
                KeyCode::Space => {
                    self.hard_drop();
                },
                KeyCode::KeyC | KeyCode::ShiftLeft => {
                    self.hold_piece();
                },
                KeyCode::KeyP | KeyCode::Escape => {
                    self.state = GameState::Paused;
                },
                _ => {},
            },
            GameState::Paused => {
                if key == KeyCode::KeyP || key == KeyCode::Escape {
                    self.state = GameState::Playing;
                    self.last_update = Instant::now();
                }
            },
            GameState::GameOver => {
                if key == KeyCode::Enter || key == KeyCode::Space {
                    self.reset();
                }
            },
            _ => {},
        }
    }

    fn handle_key_release(&mut self, key: KeyCode) {
        match key {
            KeyCode::ArrowDown | KeyCode::KeyS => {
                self.is_soft_dropping = false;
            },
            KeyCode::ArrowLeft | KeyCode::KeyA => {
                self.stop_das(-1);
            },
            KeyCode::ArrowRight | KeyCode::KeyD => {
                self.stop_das(1);
            },
            _ => {},
        }
    }

    fn render(&self, frame: &mut [u8]) {
        let (shake_x, shake_y) = self.effects.get_offset();

        // Draw gradient background
        self.draw_gradient_background(frame);

        // Draw decorative elements
        self.draw_scanlines(frame);
        self.draw_grid_glow(frame, shake_x, shake_y);

        // Draw grid background
        self.draw_grid_background(frame, shake_x, shake_y);

        // Draw locked pieces
        for row in 0..GRID_ROWS {
            for col in 0..GRID_COLS {
                if let Some(color) = self.grid[row][col] {
                    let flash = if self.state == GameState::LineClear
                        && self.clearing_lines.contains(&row)
                    {
                        let t = self.line_clear_timer / LINE_CLEAR_ANIMATION_DURATION;
                        (t * 8.0 * std::f32::consts::PI).sin().abs()
                    } else {
                        0.0
                    };

                    self.draw_cell(
                        frame,
                        CellDraw {
                            col,
                            row,
                            color,
                            flash,
                            shake_x,
                            shake_y,
                        },
                    );
                }
            }
        }

        // Draw ghost piece
        if self.state == GameState::Playing {
            let ghost_y = self.ghost_y();
            let ghost_color = self.current_piece.piece_type.glow_color();
            for (x, y) in self
                .current_piece
                .piece_type
                .cells(self.current_piece.rotation)
            {
                let gx = x + self.current_piece.x;
                let gy = y + ghost_y;
                if gy >= 0 {
                    self.draw_ghost_cell(
                        frame,
                        gx as usize,
                        gy as usize,
                        ghost_color,
                        shake_x,
                        shake_y,
                    );
                }
            }
        }

        // Draw current piece with glow
        if self.state == GameState::Playing || self.state == GameState::LineClear {
            let color = self.current_piece.piece_type.color();
            for (x, y) in self.current_piece.cells() {
                if y >= 0 {
                    self.draw_cell(
                        frame,
                        CellDraw {
                            col: x as usize,
                            row: y as usize,
                            color,
                            flash: 0.0,
                            shake_x,
                            shake_y,
                        },
                    );
                }
            }
        }

        // Draw particles
        for p in &self.particles {
            self.draw_particle(frame, p);
        }

        // Draw UI panels
        self.draw_title(frame);
        self.draw_next_panel(frame);
        self.draw_hold_panel(frame);
        self.draw_score_panel(frame);

        // Draw flash effect
        if self.effects.flash_intensity > 0.01 {
            self.apply_flash(frame);
        }

        // Draw game state overlays
        match self.state {
            GameState::Paused => self.draw_paused_overlay(frame),
            GameState::GameOver => self.draw_game_over_overlay(frame),
            _ => {},
        }
    }

    fn draw_gradient_background(&self, frame: &mut [u8]) {
        for y in 0..HEIGHT {
            let t = y as f32 / HEIGHT as f32;
            let r = lerp(palette::BG_TOP[0] as f32, palette::BG_BOTTOM[0] as f32, t) as u8;
            let g = lerp(palette::BG_TOP[1] as f32, palette::BG_BOTTOM[1] as f32, t) as u8;
            let b = lerp(palette::BG_TOP[2] as f32, palette::BG_BOTTOM[2] as f32, t) as u8;

            for x in 0..WIDTH {
                let idx = ((y * WIDTH + x) * 4) as usize;
                frame[idx] = r;
                frame[idx + 1] = g;
                frame[idx + 2] = b;
                frame[idx + 3] = 255;
            }
        }
    }

    fn draw_scanlines(&self, frame: &mut [u8]) {
        // Subtle CRT scanline effect
        for y in (0..HEIGHT).step_by(3) {
            for x in 0..WIDTH {
                let idx = ((y * WIDTH + x) * 4) as usize;
                frame[idx] = frame[idx].saturating_sub(8);
                frame[idx + 1] = frame[idx + 1].saturating_sub(8);
                frame[idx + 2] = frame[idx + 2].saturating_sub(8);
            }
        }
    }

    fn draw_grid_glow(&self, frame: &mut [u8], shake_x: i32, shake_y: i32) {
        let x0 = (GRID_OFFSET_X as i32 + shake_x - 4).max(0);
        let y0 = (GRID_OFFSET_Y as i32 + shake_y - 4).max(0);
        let w = (GRID_COLS as u32 * CELL_SIZE) as i32 + 8;
        let h = (GRID_ROWS as u32 * CELL_SIZE) as i32 + 8;

        // Outer glow
        for y in y0..(y0 + h).min(HEIGHT as i32) {
            for x in x0..(x0 + w).min(WIDTH as i32) {
                let dx = if x < GRID_OFFSET_X as i32 + shake_x {
                    GRID_OFFSET_X as i32 + shake_x - x
                } else if x > GRID_OFFSET_X as i32 + shake_x + (GRID_COLS as i32 * CELL_SIZE as i32)
                {
                    x - (GRID_OFFSET_X as i32 + shake_x + GRID_COLS as i32 * CELL_SIZE as i32)
                } else {
                    0
                };
                let dy = if y < GRID_OFFSET_Y as i32 + shake_y {
                    GRID_OFFSET_Y as i32 + shake_y - y
                } else if y > GRID_OFFSET_Y as i32 + shake_y + (GRID_ROWS as i32 * CELL_SIZE as i32)
                {
                    y - (GRID_OFFSET_Y as i32 + shake_y + GRID_ROWS as i32 * CELL_SIZE as i32)
                } else {
                    0
                };

                if dx > 0 || dy > 0 {
                    let dist = ((dx * dx + dy * dy) as f32).sqrt();
                    if dist < 6.0 {
                        let alpha = (1.0 - dist / 6.0) * 0.3;
                        let idx = ((y as u32 * WIDTH + x as u32) * 4) as usize;
                        if idx + 3 < frame.len() {
                            frame[idx] =
                                (frame[idx] as f32 + palette::GRID_BORDER[0] as f32 * alpha) as u8;
                            frame[idx + 1] = (frame[idx + 1] as f32
                                + palette::GRID_BORDER[1] as f32 * alpha)
                                as u8;
                            frame[idx + 2] = (frame[idx + 2] as f32
                                + palette::GRID_BORDER[2] as f32 * alpha)
                                as u8;
                        }
                    }
                }
            }
        }
    }

    fn draw_grid_background(&self, frame: &mut [u8], shake_x: i32, shake_y: i32) {
        let x0 = GRID_OFFSET_X as i32 + shake_x;
        let y0 = GRID_OFFSET_Y as i32 + shake_y;
        let w = (GRID_COLS as u32 * CELL_SIZE) as i32;
        let h = (GRID_ROWS as u32 * CELL_SIZE) as i32;

        // Fill grid area
        for y in y0.max(0)..(y0 + h).min(HEIGHT as i32) {
            for x in x0.max(0)..(x0 + w).min(WIDTH as i32) {
                let idx = ((y as u32 * WIDTH + x as u32) * 4) as usize;
                if idx + 3 < frame.len() {
                    frame[idx..idx + 4].copy_from_slice(&palette::BG_GRID);
                }
            }
        }

        // Draw grid lines with subtle variation
        for col in 0..=GRID_COLS {
            let x = x0 + col as i32 * CELL_SIZE as i32;
            if x >= 0 && x < WIDTH as i32 {
                for y in y0.max(0)..(y0 + h).min(HEIGHT as i32) {
                    let idx = ((y as u32 * WIDTH + x as u32) * 4) as usize;
                    if idx + 3 < frame.len() {
                        frame[idx..idx + 4].copy_from_slice(&palette::GRID_LINE);
                    }
                }
            }
        }
        for row in 0..=GRID_ROWS {
            let y = y0 + row as i32 * CELL_SIZE as i32;
            if y >= 0 && y < HEIGHT as i32 {
                for x in x0.max(0)..(x0 + w).min(WIDTH as i32) {
                    let idx = ((y as u32 * WIDTH + x as u32) * 4) as usize;
                    if idx + 3 < frame.len() {
                        frame[idx..idx + 4].copy_from_slice(&palette::GRID_LINE);
                    }
                }
            }
        }

        // Draw neon border
        let border_color = palette::GRID_BORDER;
        // Top
        for x in (x0 - 2).max(0)..(x0 + w + 2).min(WIDTH as i32) {
            for thickness in 0..2 {
                let y = y0 - 2 + thickness;
                if y >= 0 && y < HEIGHT as i32 {
                    let idx = ((y as u32 * WIDTH + x as u32) * 4) as usize;
                    if idx + 3 < frame.len() {
                        frame[idx..idx + 4].copy_from_slice(&border_color);
                    }
                }
            }
        }
        // Bottom
        for x in (x0 - 2).max(0)..(x0 + w + 2).min(WIDTH as i32) {
            for thickness in 0..2 {
                let y = y0 + h + thickness;
                if y >= 0 && y < HEIGHT as i32 {
                    let idx = ((y as u32 * WIDTH + x as u32) * 4) as usize;
                    if idx + 3 < frame.len() {
                        frame[idx..idx + 4].copy_from_slice(&border_color);
                    }
                }
            }
        }
        // Left
        for y in (y0 - 2).max(0)..(y0 + h + 2).min(HEIGHT as i32) {
            for thickness in 0..2 {
                let x = x0 - 2 + thickness;
                if x >= 0 && x < WIDTH as i32 {
                    let idx = ((y as u32 * WIDTH + x as u32) * 4) as usize;
                    if idx + 3 < frame.len() {
                        frame[idx..idx + 4].copy_from_slice(&border_color);
                    }
                }
            }
        }
        // Right
        for y in (y0 - 2).max(0)..(y0 + h + 2).min(HEIGHT as i32) {
            for thickness in 0..2 {
                let x = x0 + w + thickness;
                if x >= 0 && x < WIDTH as i32 {
                    let idx = ((y as u32 * WIDTH + x as u32) * 4) as usize;
                    if idx + 3 < frame.len() {
                        frame[idx..idx + 4].copy_from_slice(&border_color);
                    }
                }
            }
        }
    }

    fn draw_cell(&self, frame: &mut [u8], cell: CellDraw) {
        let CellDraw {
            col,
            row,
            color,
            flash,
            shake_x,
            shake_y,
        } = cell;
        let x0 = GRID_OFFSET_X as i32 + col as i32 * CELL_SIZE as i32 + shake_x;
        let y0 = GRID_OFFSET_Y as i32 + row as i32 * CELL_SIZE as i32 + shake_y;
        let size = CELL_SIZE as i32;
        let inset = 1;

        // Draw glow effect first (15% alpha around the cell).
        let glow_size = 3;
        let glow = [color[0], color[1], color[2], 38]; // 0.15 * 255 ≈ 38
        for gy in (y0 - glow_size).max(0)..(y0 + size + glow_size).min(HEIGHT as i32) {
            for gx in (x0 - glow_size).max(0)..(x0 + size + glow_size).min(WIDTH as i32) {
                let in_cell = gx >= x0 && gx < x0 + size && gy >= y0 && gy < y0 + size;
                if !in_cell {
                    blend_pixel(frame, gx, gy, WIDTH, HEIGHT, glow);
                }
            }
        }

        // Draw main cell body
        for y in (y0 + inset).max(0)..(y0 + size - inset).min(HEIGHT as i32) {
            for x in (x0 + inset).max(0)..(x0 + size - inset).min(WIDTH as i32) {
                let idx = ((y as u32 * WIDTH + x as u32) * 4) as usize;
                if idx + 3 < frame.len() {
                    let mut c = color;
                    if flash > 0.0 {
                        c[0] = (c[0] as f32 + (255.0 - c[0] as f32) * flash) as u8;
                        c[1] = (c[1] as f32 + (255.0 - c[1] as f32) * flash) as u8;
                        c[2] = (c[2] as f32 + (255.0 - c[2] as f32) * flash) as u8;
                    }
                    frame[idx..idx + 4].copy_from_slice(&c);
                }
            }
        }

        // Inner highlight (top-left)
        let highlight = [
            color[0].saturating_add(80),
            color[1].saturating_add(80),
            color[2].saturating_add(80),
            255,
        ];
        for x in (x0 + inset + 1).max(0)..(x0 + size - inset - 1).min(WIDTH as i32) {
            let y = y0 + inset + 1;
            if y >= 0 && y < HEIGHT as i32 {
                let idx = ((y as u32 * WIDTH + x as u32) * 4) as usize;
                if idx + 3 < frame.len() {
                    frame[idx..idx + 4].copy_from_slice(&highlight);
                }
            }
        }
        for y in (y0 + inset + 1).max(0)..(y0 + size - inset - 1).min(HEIGHT as i32) {
            let x = x0 + inset + 1;
            if x >= 0 && x < WIDTH as i32 {
                let idx = ((y as u32 * WIDTH + x as u32) * 4) as usize;
                if idx + 3 < frame.len() {
                    frame[idx..idx + 4].copy_from_slice(&highlight);
                }
            }
        }

        // Inner shadow (bottom-right)
        let shadow = [
            color[0].saturating_sub(60),
            color[1].saturating_sub(60),
            color[2].saturating_sub(60),
            255,
        ];
        for x in (x0 + inset + 1).max(0)..(x0 + size - inset - 1).min(WIDTH as i32) {
            let y = y0 + size - inset - 2;
            if y >= 0 && y < HEIGHT as i32 {
                let idx = ((y as u32 * WIDTH + x as u32) * 4) as usize;
                if idx + 3 < frame.len() {
                    frame[idx..idx + 4].copy_from_slice(&shadow);
                }
            }
        }
        for y in (y0 + inset + 1).max(0)..(y0 + size - inset - 1).min(HEIGHT as i32) {
            let x = x0 + size - inset - 2;
            if x >= 0 && x < WIDTH as i32 {
                let idx = ((y as u32 * WIDTH + x as u32) * 4) as usize;
                if idx + 3 < frame.len() {
                    frame[idx..idx + 4].copy_from_slice(&shadow);
                }
            }
        }
    }

    fn draw_ghost_cell(
        &self,
        frame: &mut [u8],
        col: usize,
        row: usize,
        color: [u8; 4],
        shake_x: i32,
        shake_y: i32,
    ) {
        let x0 = GRID_OFFSET_X as i32 + col as i32 * CELL_SIZE as i32 + shake_x;
        let y0 = GRID_OFFSET_Y as i32 + row as i32 * CELL_SIZE as i32 + shake_y;
        let size = CELL_SIZE as i32;
        let inset = 2;

        // Draw semi-transparent fill (multiply source alpha by 0.3).
        let fill_alpha = ((color[3] as f32 * 0.3) as u8).max(1);
        let fill = [color[0], color[1], color[2], fill_alpha];
        for y in (y0 + inset).max(0)..(y0 + size - inset).min(HEIGHT as i32) {
            for x in (x0 + inset).max(0)..(x0 + size - inset).min(WIDTH as i32) {
                blend_pixel(frame, x, y, WIDTH, HEIGHT, fill);
            }
        }

        // Draw outline
        let outline_color = [color[0], color[1], color[2], 180];
        // Top & bottom
        for x in (x0 + inset).max(0)..(x0 + size - inset).min(WIDTH as i32) {
            for &y in &[y0 + inset, y0 + size - inset - 1] {
                blend_pixel(frame, x, y, WIDTH, HEIGHT, outline_color);
            }
        }
        // Left & right
        for y in (y0 + inset).max(0)..(y0 + size - inset).min(HEIGHT as i32) {
            for &x in &[x0 + inset, x0 + size - inset - 1] {
                blend_pixel(frame, x, y, WIDTH, HEIGHT, outline_color);
            }
        }
    }

    fn draw_particle(&self, frame: &mut [u8], p: &Particle) {
        let x = p.pos.x.round() as i32;
        let y = p.pos.y.round() as i32;
        let size = p.size.round() as i32;
        let alpha = p.alpha();

        for dy in -size..=size {
            for dx in -size..=size {
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= size * size {
                    let px = x + dx;
                    let py = y + dy;
                    if px >= 0 && px < WIDTH as i32 && py >= 0 && py < HEIGHT as i32 {
                        let idx = ((py as u32 * WIDTH + px as u32) * 4) as usize;
                        if idx + 3 < frame.len() {
                            let dist_factor = 1.0 - (dist_sq as f32).sqrt() / (size as f32 + 1.0);
                            let a = alpha * dist_factor;
                            frame[idx] =
                                (frame[idx] as f32 * (1.0 - a) + p.color[0] as f32 * a) as u8;
                            frame[idx + 1] =
                                (frame[idx + 1] as f32 * (1.0 - a) + p.color[1] as f32 * a) as u8;
                            frame[idx + 2] =
                                (frame[idx + 2] as f32 * (1.0 - a) + p.color[2] as f32 * a) as u8;
                        }
                    }
                }
            }
        }
    }

    fn apply_flash(&self, frame: &mut [u8]) {
        let intensity = self.effects.flash_intensity;
        let color = self.effects.flash_color;
        for chunk in frame.chunks_exact_mut(4) {
            chunk[0] = (chunk[0] as f32 * (1.0 - intensity) + color[0] as f32 * intensity) as u8;
            chunk[1] = (chunk[1] as f32 * (1.0 - intensity) + color[1] as f32 * intensity) as u8;
            chunk[2] = (chunk[2] as f32 * (1.0 - intensity) + color[2] as f32 * intensity) as u8;
        }
    }

    fn draw_title(&self, frame: &mut [u8]) {
        // Pulsing title
        let pulse = (self.game_time * 2.0).sin() * 0.2 + 0.8;
        let title_color = [
            (palette::TEXT_TITLE[0] as f32 * pulse) as u8,
            (palette::TEXT_TITLE[1] as f32 * pulse) as u8,
            (palette::TEXT_TITLE[2] as f32 * pulse) as u8,
            255,
        ];
        self.draw_text_scaled(frame, "GRAVITA", GRID_OFFSET_X as i32, 15, title_color, 2);
        self.draw_text(
            frame,
            "TETRIS",
            GRID_OFFSET_X as i32 + 100,
            25,
            palette::TEXT_ACCENT,
        );
    }

    fn draw_mini_piece(
        &self,
        frame: &mut [u8],
        piece_type: TetrominoType,
        cx: i32,
        cy: i32,
        scale: i32,
    ) {
        let color = piece_type.color();
        let cells = piece_type.cells(0);

        for (dx, dy) in cells {
            let x0 = cx + dx * scale;
            let y0 = cy + dy * scale;

            // Mini glow (20% alpha around the cell).
            let glow = [color[0], color[1], color[2], 51]; // 0.2 * 255 ≈ 51
            for gy in (y0 - 1).max(0)..(y0 + scale + 1).min(HEIGHT as i32) {
                for gx in (x0 - 1).max(0)..(x0 + scale + 1).min(WIDTH as i32) {
                    let in_cell = gx >= x0 && gx < x0 + scale && gy >= y0 && gy < y0 + scale;
                    if !in_cell {
                        blend_pixel(frame, gx, gy, WIDTH, HEIGHT, glow);
                    }
                }
            }

            // Cell body
            for y in y0.max(0)..(y0 + scale).min(HEIGHT as i32) {
                for x in x0.max(0)..(x0 + scale).min(WIDTH as i32) {
                    let idx = ((y as u32 * WIDTH + x as u32) * 4) as usize;
                    if idx + 3 < frame.len() {
                        frame[idx..idx + 4].copy_from_slice(&color);
                    }
                }
            }
        }
    }

    fn draw_next_panel(&self, frame: &mut [u8]) {
        let panel_x = GRID_OFFSET_X + GRID_COLS as u32 * CELL_SIZE + 25;
        let panel_y = GRID_OFFSET_Y;

        self.draw_text(
            frame,
            "NEXT",
            panel_x as i32,
            panel_y as i32 - 5,
            palette::TEXT_SECONDARY,
        );

        // Draw panel background
        self.draw_panel_bg(frame, panel_x as i32, panel_y as i32 + 10, 90, 180);

        // Draw next pieces
        for (i, &piece_type) in self.next_pieces.iter().take(4).enumerate() {
            let cy = panel_y as i32 + 30 + i as i32 * 42;
            let scale = if i == 0 { 10 } else { 8 };
            self.draw_mini_piece(frame, piece_type, panel_x as i32 + 20, cy, scale);
        }
    }

    fn draw_hold_panel(&self, frame: &mut [u8]) {
        let panel_x = GRID_OFFSET_X + GRID_COLS as u32 * CELL_SIZE + 25;
        let panel_y = GRID_OFFSET_Y + 210;

        self.draw_text(
            frame,
            "HOLD",
            panel_x as i32,
            panel_y as i32 - 5,
            palette::TEXT_SECONDARY,
        );

        // Draw panel background
        self.draw_panel_bg(frame, panel_x as i32, panel_y as i32 + 10, 90, 50);

        if let Some(piece_type) = self.held_piece {
            let color = if self.can_hold {
                piece_type.color()
            } else {
                [0x40, 0x30, 0x50, 0xff]
            };
            let cells = piece_type.cells(0);
            let scale = 8;
            let cx = panel_x as i32 + 20;
            let cy = panel_y as i32 + 25;

            for (dx, dy) in cells {
                let x0 = cx + dx * scale;
                let y0 = cy + dy * scale;

                for y in y0.max(0)..(y0 + scale).min(HEIGHT as i32) {
                    for x in x0.max(0)..(x0 + scale).min(WIDTH as i32) {
                        let idx = ((y as u32 * WIDTH + x as u32) * 4) as usize;
                        if idx + 3 < frame.len() {
                            frame[idx..idx + 4].copy_from_slice(&color);
                        }
                    }
                }
            }
        }
    }

    fn draw_panel_bg(&self, frame: &mut [u8], x: i32, y: i32, w: i32, h: i32) {
        // Darken the background with a 30%-alpha dark-blue tint to mark the panel area.
        // Source color: (0, 0, 20) at alpha 0.3 ≈ 77/255.
        let tint = [0, 0, 20, 77];
        for py in y.max(0)..(y + h).min(HEIGHT as i32) {
            for px in x.max(0)..(x + w).min(WIDTH as i32) {
                blend_pixel(frame, px, py, WIDTH, HEIGHT, tint);
            }
        }
    }

    fn draw_score_panel(&self, frame: &mut [u8]) {
        let panel_x = GRID_OFFSET_X + GRID_COLS as u32 * CELL_SIZE + 25;
        let panel_y = GRID_OFFSET_Y + 290;

        self.draw_text(
            frame,
            "SCORE",
            panel_x as i32,
            panel_y as i32,
            palette::TEXT_SECONDARY,
        );
        self.draw_text(
            frame,
            &format!("{:08}", self.score),
            panel_x as i32,
            panel_y as i32 + 14,
            palette::TEXT_PRIMARY,
        );

        self.draw_text(
            frame,
            "LEVEL",
            panel_x as i32,
            panel_y as i32 + 40,
            palette::TEXT_SECONDARY,
        );
        self.draw_text(
            frame,
            &format!("{}", self.level),
            panel_x as i32,
            panel_y as i32 + 54,
            palette::TEXT_ACCENT,
        );

        self.draw_text(
            frame,
            "LINES",
            panel_x as i32,
            panel_y as i32 + 80,
            palette::TEXT_SECONDARY,
        );
        self.draw_text(
            frame,
            &format!("{}", self.lines_cleared),
            panel_x as i32,
            panel_y as i32 + 94,
            palette::TEXT_PRIMARY,
        );

        // Combo display
        if self.combo > 1 {
            let combo_pulse = (self.game_time * 6.0).sin() * 0.3 + 0.7;
            let combo_color = [
                (palette::ORANGE[0] as f32 * combo_pulse) as u8,
                (palette::ORANGE[1] as f32 * combo_pulse) as u8,
                (palette::ORANGE[2] as f32 * combo_pulse) as u8,
                255,
            ];
            self.draw_text(
                frame,
                &format!("{}x COMBO", self.combo),
                panel_x as i32,
                panel_y as i32 + 130,
                combo_color,
            );
        }

        // Clear type display
        if self.last_clear_count >= 4 && self.state != GameState::LineClear {
            let tetris_pulse = (self.game_time * 4.0).sin() * 0.3 + 0.7;
            let tetris_color = [
                (palette::CYAN[0] as f32 * tetris_pulse) as u8,
                (palette::CYAN[1] as f32 * tetris_pulse) as u8,
                (palette::CYAN[2] as f32 * tetris_pulse) as u8,
                255,
            ];
            self.draw_text(
                frame,
                "TETRIS!",
                panel_x as i32,
                panel_y as i32 + 150,
                tetris_color,
            );
        }
    }

    fn draw_paused_overlay(&self, frame: &mut [u8]) {
        // Darken screen
        for chunk in frame.chunks_exact_mut(4) {
            chunk[0] /= 3;
            chunk[1] /= 3;
            chunk[2] /= 3;
        }

        let cx = WIDTH as i32 / 2;
        let cy = HEIGHT as i32 / 2;

        self.draw_text_centered(frame, "PAUSED", cx, cy - 30, palette::TEXT_TITLE);
        self.draw_text_centered(
            frame,
            "Press P or ESC",
            cx,
            cy + 10,
            palette::TEXT_SECONDARY,
        );
        self.draw_text_centered(frame, "to resume", cx, cy + 24, palette::TEXT_SECONDARY);
    }

    fn draw_game_over_overlay(&self, frame: &mut [u8]) {
        // Darken screen with red/pink tint
        for chunk in frame.chunks_exact_mut(4) {
            chunk[0] = (chunk[0] / 2).saturating_add(40);
            chunk[1] /= 4;
            chunk[2] /= 3;
        }

        let cx = WIDTH as i32 / 2;
        let cy = HEIGHT as i32 / 2;

        self.draw_text_scaled(frame, "GAME", cx - 50, cy - 50, palette::RED, 2);
        self.draw_text_scaled(frame, "OVER", cx - 50, cy - 30, palette::RED, 2);

        self.draw_text_centered(
            frame,
            &format!("Final Score: {}", self.score),
            cx,
            cy + 20,
            palette::TEXT_PRIMARY,
        );
        self.draw_text_centered(
            frame,
            &format!("Level: {}  Lines: {}", self.level, self.lines_cleared),
            cx,
            cy + 40,
            palette::TEXT_SECONDARY,
        );

        let blink = (self.game_time * 3.0).sin() > 0.0;
        if blink {
            self.draw_text_centered(
                frame,
                "Press ENTER to restart",
                cx,
                cy + 80,
                palette::TEXT_ACCENT,
            );
        }
    }

    // Thin wrappers around `gravita_renderer::text` so the call sites below
    // don't need to repeat WIDTH/HEIGHT every time.
    fn draw_text(&self, frame: &mut [u8], text: &str, x: i32, y: i32, color: [u8; 4]) {
        renderer_text::draw_text(frame, text, x, y, color, WIDTH, HEIGHT);
    }

    fn draw_text_scaled(
        &self,
        frame: &mut [u8],
        text: &str,
        x: i32,
        y: i32,
        color: [u8; 4],
        scale: i32,
    ) {
        renderer_text::draw_text_scaled(frame, text, x, y, color, scale, WIDTH, HEIGHT);
    }

    fn draw_text_centered(&self, frame: &mut [u8], text: &str, cx: i32, y: i32, color: [u8; 4]) {
        renderer_text::draw_text_centered(frame, text, cx, y, color, WIDTH, HEIGHT);
    }
}

// Local `lerp` and 5x7 bitmap font moved into `gravita_math::lerp` and
// `gravita_renderer::text` respectively.

// ============================================================================
// APPLICATION HANDLER
// ============================================================================

struct App {
    window: Option<Window>,
    pixels: Option<Pixels<'static>>,
    game: TetrisGame,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            pixels: None,
            game: TetrisGame::new(),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let size = LogicalSize::new(WIDTH, HEIGHT);
            let attrs = Window::default_attributes()
                .with_title("GRAVITA TETRIS")
                .with_inner_size(size)
                .with_min_inner_size(size)
                .with_resizable(false);

            let window = event_loop.create_window(attrs).expect("create window");
            let window_size = window.inner_size();

            let surface = SurfaceTexture::new(window_size.width, window_size.height, &window);
            let pixels = Pixels::new(WIDTH, HEIGHT, surface).expect("create pixels");

            // SAFETY: We're keeping window alive as long as pixels exists
            #[allow(clippy::missing_transmute_annotations)]
            {
                self.pixels = Some(unsafe { std::mem::transmute(pixels) });
            }
            self.window = Some(window);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state,
                        ..
                    },
                ..
            } => {
                if state.is_pressed() {
                    self.game.handle_input(key);
                } else {
                    self.game.handle_key_release(key);
                }
            },
            WindowEvent::RedrawRequested => {
                self.game.update();

                if let Some(pixels) = &mut self.pixels {
                    self.game.render(pixels.frame_mut());
                    if pixels.render().is_err() {
                        event_loop.exit();
                    }
                }

                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            },
            _ => {},
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

// ============================================================================
// MAIN
// ============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}
