// Frogger clone in Rust using raylib
// First draft - ha1tch

use raylib::prelude::*;
use rand::Rng;

// =============================================================================
// Constants
// =============================================================================

const SCREEN_WIDTH: i32 = 640;
const SCREEN_HEIGHT: i32 = 480;
const CELL_SIZE: i32 = 32;
const GRID_COLS: i32 = SCREEN_WIDTH / CELL_SIZE;  // 20
const GRID_ROWS: i32 = SCREEN_HEIGHT / CELL_SIZE; // 15

const FROG_START_ROW: i32 = GRID_ROWS - 1;        // Bottom row
const FROG_START_COL: i32 = GRID_COLS / 2;        // Center

const INITIAL_LIVES: i32 = 3;
const GOAL_COUNT: usize = 5;

// Lane configuration: row index -> lane type
// Row 0: Goal area (lily pads)
// Rows 1-5: River (logs/turtles)
// Row 6: Safe zone (grass)
// Rows 7-12: Road (cars/trucks)
// Row 13: Safe zone (grass)
// Row 14: Start area

const RIVER_START: i32 = 1;
const RIVER_END: i32 = 5;
const ROAD_START: i32 = 7;
const ROAD_END: i32 = 12;

// =============================================================================
// Types
// =============================================================================

#[derive(Clone, Copy, PartialEq)]
enum LaneType {
    Safe,
    Road,
    River,
    Goal,
}

#[derive(Clone, Copy)]
enum Direction {
    Left,
    Right,
}

impl Direction {
    fn multiplier(&self) -> f32 {
        match self {
            Direction::Left => -1.0,
            Direction::Right => 1.0,
        }
    }
}

#[derive(Clone)]
struct MovingObject {
    x: f32,
    y: i32,           // Row (grid-based)
    width: i32,       // In cells
    speed: f32,
    direction: Direction,
    color: Color,
}

impl MovingObject {
    fn new(x: f32, row: i32, width: i32, speed: f32, direction: Direction, color: Color) -> Self {
        Self {
            x,
            y: row,
            width,
            speed,
            direction,
            color,
        }
    }

    fn update(&mut self, dt: f32) {
        self.x += self.speed * self.direction.multiplier() * dt;

        // Wrap around screen
        let screen_width = SCREEN_WIDTH as f32;
        let obj_width = (self.width * CELL_SIZE) as f32;

        match self.direction {
            Direction::Right => {
                if self.x > screen_width {
                    self.x = -obj_width;
                }
            }
            Direction::Left => {
                if self.x + obj_width < 0.0 {
                    self.x = screen_width;
                }
            }
        }
    }

    fn get_rect(&self) -> Rectangle {
        Rectangle {
            x: self.x,
            y: (self.y * CELL_SIZE) as f32,
            width: (self.width * CELL_SIZE) as f32,
            height: CELL_SIZE as f32,
        }
    }
}

struct Frog {
    x: i32,  // Grid column
    y: i32,  // Grid row
    riding_offset: f32,  // X offset when on a log
    is_riding: bool,
}

impl Frog {
    fn new() -> Self {
        Self {
            x: FROG_START_COL,
            y: FROG_START_ROW,
            riding_offset: 0.0,
            is_riding: false,
        }
    }

    fn reset(&mut self) {
        self.x = FROG_START_COL;
        self.y = FROG_START_ROW;
        self.riding_offset = 0.0;
        self.is_riding = false;
    }

    fn get_pixel_x(&self) -> f32 {
        (self.x * CELL_SIZE) as f32 + self.riding_offset
    }

    fn get_pixel_y(&self) -> f32 {
        (self.y * CELL_SIZE) as f32
    }

    fn get_rect(&self) -> Rectangle {
        Rectangle {
            x: self.get_pixel_x(),
            y: self.get_pixel_y(),
            width: CELL_SIZE as f32,
            height: CELL_SIZE as f32,
        }
    }

    fn move_up(&mut self) {
        if self.y > 0 {
            self.y -= 1;
            self.riding_offset = 0.0;
        }
    }

    fn move_down(&mut self) {
        if self.y < GRID_ROWS - 1 {
            self.y += 1;
            self.riding_offset = 0.0;
        }
    }

    fn move_left(&mut self) {
        let new_x = self.get_pixel_x() - CELL_SIZE as f32;
        if new_x >= 0.0 {
            if self.is_riding {
                self.riding_offset -= CELL_SIZE as f32;
            } else {
                self.x -= 1;
            }
        }
    }

    fn move_right(&mut self) {
        let new_x = self.get_pixel_x() + CELL_SIZE as f32;
        if new_x < (SCREEN_WIDTH - CELL_SIZE) as f32 {
            if self.is_riding {
                self.riding_offset += CELL_SIZE as f32;
            } else {
                self.x += 1;
            }
        }
    }
}

struct GoalSlot {
    x: i32,      // Grid column
    occupied: bool,
}

struct GameState {
    frog: Frog,
    cars: Vec<MovingObject>,
    logs: Vec<MovingObject>,
    goals: Vec<GoalSlot>,
    lives: i32,
    score: i32,
    game_over: bool,
    won: bool,
}

impl GameState {
    fn new() -> Self {
        let mut state = Self {
            frog: Frog::new(),
            cars: Vec::new(),
            logs: Vec::new(),
            goals: Vec::new(),
            lives: INITIAL_LIVES,
            score: 0,
            game_over: false,
            won: false,
        };
        state.init_lanes();
        state.init_goals();
        state
    }

    fn init_goals(&mut self) {
        // Create 5 goal slots evenly spaced at the top
        self.goals.clear();
        let spacing = GRID_COLS / (GOAL_COUNT as i32 + 1);
        for i in 0..GOAL_COUNT {
            self.goals.push(GoalSlot {
                x: spacing * (i as i32 + 1),
                occupied: false,
            });
        }
    }

    fn init_lanes(&mut self) {
        let mut rng = rand::thread_rng();

        self.cars.clear();
        self.logs.clear();

        // Road lanes (rows 7-12)
        let car_configs: [(i32, i32, f32, Direction, Color); 6] = [
            (7, 2, 80.0, Direction::Left, Color::RED),
            (8, 3, 100.0, Direction::Right, Color::YELLOW),
            (9, 2, 70.0, Direction::Left, Color::BLUE),
            (10, 4, 120.0, Direction::Right, Color::PURPLE),
            (11, 2, 90.0, Direction::Left, Color::ORANGE),
            (12, 3, 60.0, Direction::Right, Color::MAROON),
        ];

        for (row, width, speed, dir, color) in car_configs {
            // Add 2-3 vehicles per lane
            let count = rng.gen_range(2..=3);
            let spacing = SCREEN_WIDTH as f32 / count as f32;
            for i in 0..count {
                let x = i as f32 * spacing + rng.gen_range(-20.0..20.0);
                self.cars.push(MovingObject::new(x, row, width, speed, dir, color));
            }
        }

        // River lanes (rows 1-5)
        let log_configs: [(i32, i32, f32, Direction); 5] = [
            (1, 4, 50.0, Direction::Right),
            (2, 3, 70.0, Direction::Left),
            (3, 5, 40.0, Direction::Right),
            (4, 3, 60.0, Direction::Left),
            (5, 4, 55.0, Direction::Right),
        ];

        for (row, width, speed, dir) in log_configs {
            let count = rng.gen_range(2..=3);
            let spacing = SCREEN_WIDTH as f32 / count as f32;
            for i in 0..count {
                let x = i as f32 * spacing + rng.gen_range(-30.0..30.0);
                self.logs.push(MovingObject::new(x, row, width, speed, dir, Color::SADDLEBROWN));
            }
        }
    }

    fn reset(&mut self) {
        self.frog.reset();
        self.lives = INITIAL_LIVES;
        self.score = 0;
        self.game_over = false;
        self.won = false;
        self.init_lanes();
        self.init_goals();
    }

    fn kill_frog(&mut self) {
        self.lives -= 1;
        if self.lives <= 0 {
            self.game_over = true;
        } else {
            self.frog.reset();
        }
    }

    fn check_goal_reached(&mut self) {
        if self.frog.y == 0 {
            let frog_center = self.frog.get_pixel_x() + (CELL_SIZE / 2) as f32;

            for goal in &mut self.goals {
                if !goal.occupied {
                    let goal_center = (goal.x * CELL_SIZE + CELL_SIZE / 2) as f32;
                    let distance = (frog_center - goal_center).abs();

                    if distance < (CELL_SIZE as f32 * 0.8) {
                        goal.occupied = true;
                        self.score += 100;
                        self.frog.reset();

                        // Check win condition
                        if self.goals.iter().all(|g| g.occupied) {
                            self.won = true;
                            self.game_over = true;
                        }
                        return;
                    }
                }
            }

            // Missed the goal slot - die
            self.kill_frog();
        }
    }

    fn update(&mut self, dt: f32) {
        if self.game_over {
            return;
        }

        // Update moving objects
        for car in &mut self.cars {
            car.update(dt);
        }
        for log in &mut self.logs {
            log.update(dt);
        }

        let frog_rect = self.frog.get_rect();
        let frog_row = self.frog.y;

        // Check road collisions
        if frog_row >= ROAD_START && frog_row <= ROAD_END {
            for car in &self.cars {
                if car.y == frog_row {
                    let car_rect = car.get_rect();
                    if check_collision_recs(frog_rect, car_rect) {
                        self.kill_frog();
                        return;
                    }
                }
            }
        }

        // Check river logic
        if frog_row >= RIVER_START && frog_row <= RIVER_END {
            let mut on_log = false;

            for log in &self.logs {
                if log.y == frog_row {
                    let log_rect = log.get_rect();
                    if check_collision_recs(frog_rect, log_rect) {
                        on_log = true;
                        self.frog.is_riding = true;
                        // Move frog with log
                        self.frog.riding_offset += log.speed * log.direction.multiplier() * dt;
                        break;
                    }
                }
            }

            if !on_log {
                // Fell in water
                self.kill_frog();
                return;
            }

            // Check if frog drifted off screen
            let frog_x = self.frog.get_pixel_x();
            if frog_x < 0.0 || frog_x > (SCREEN_WIDTH - CELL_SIZE) as f32 {
                self.kill_frog();
                return;
            }
        } else {
            self.frog.is_riding = false;
            self.frog.riding_offset = 0.0;
        }

        // Check if reached goal row
        self.check_goal_reached();
    }
}

// =============================================================================
// Helper: Rectangle collision
// =============================================================================

fn check_collision_recs(r1: Rectangle, r2: Rectangle) -> bool {
    r1.x < r2.x + r2.width
        && r1.x + r1.width > r2.x
        && r1.y < r2.y + r2.height
        && r1.y + r1.height > r2.y
}

// =============================================================================
// Rendering
// =============================================================================

fn get_lane_type(row: i32) -> LaneType {
    if row == 0 {
        LaneType::Goal
    } else if row >= RIVER_START && row <= RIVER_END {
        LaneType::River
    } else if row >= ROAD_START && row <= ROAD_END {
        LaneType::Road
    } else {
        LaneType::Safe
    }
}

fn draw_background(d: &mut RaylibDrawHandle) {
    for row in 0..GRID_ROWS {
        let lane_type = get_lane_type(row);
        let color = match lane_type {
            LaneType::Safe => Color::new(34, 139, 34, 255),   // Forest green
            LaneType::Road => Color::new(50, 50, 50, 255),    // Dark grey
            LaneType::River => Color::new(30, 144, 255, 255), // Dodger blue
            LaneType::Goal => Color::new(0, 100, 0, 255),     // Dark green
        };

        d.draw_rectangle(
            0,
            row * CELL_SIZE,
            SCREEN_WIDTH,
            CELL_SIZE,
            color,
        );

        // Draw road markings
        if lane_type == LaneType::Road {
            for col in 0..GRID_COLS {
                if col % 2 == 0 {
                    d.draw_rectangle(
                        col * CELL_SIZE + CELL_SIZE / 4,
                        row * CELL_SIZE + CELL_SIZE / 2 - 2,
                        CELL_SIZE / 2,
                        4,
                        Color::YELLOW,
                    );
                }
            }
        }
    }
}

fn draw_goals(d: &mut RaylibDrawHandle, goals: &[GoalSlot]) {
    for goal in goals {
        let x = goal.x * CELL_SIZE;
        let y = 0;

        // Draw lily pad
        d.draw_circle(
            x + CELL_SIZE / 2,
            y + CELL_SIZE / 2,
            (CELL_SIZE / 2 - 2) as f32,
            Color::new(50, 205, 50, 255), // Lime green
        );

        if goal.occupied {
            // Draw a small frog icon
            d.draw_circle(
                x + CELL_SIZE / 2,
                y + CELL_SIZE / 2,
                (CELL_SIZE / 4) as f32,
                Color::DARKGREEN,
            );
        }
    }
}

fn draw_logs(d: &mut RaylibDrawHandle, logs: &[MovingObject]) {
    for log in logs {
        let rect = log.get_rect();

        // Main log body
        d.draw_rectangle_rec(rect, log.color);

        // Log texture lines
        let line_count = log.width;
        for i in 0..line_count {
            let lx = rect.x as i32 + i * CELL_SIZE + CELL_SIZE / 2;
            d.draw_line(
                lx,
                rect.y as i32 + 4,
                lx,
                rect.y as i32 + CELL_SIZE - 4,
                Color::new(101, 67, 33, 255),
            );
        }

        // Rounded ends
        d.draw_circle(
            rect.x as i32 + 4,
            rect.y as i32 + CELL_SIZE / 2,
            4.0,
            log.color,
        );
        d.draw_circle(
            (rect.x + rect.width) as i32 - 4,
            rect.y as i32 + CELL_SIZE / 2,
            4.0,
            log.color,
        );
    }
}

fn draw_cars(d: &mut RaylibDrawHandle, cars: &[MovingObject]) {
    for car in cars {
        let rect = car.get_rect();

        // Car body
        d.draw_rectangle_rec(rect, car.color);

        // Wheels
        let wheel_y1 = rect.y as i32 + 2;
        let wheel_y2 = rect.y as i32 + CELL_SIZE - 6;

        d.draw_rectangle(rect.x as i32 + 4, wheel_y1, 8, 4, Color::BLACK);
        d.draw_rectangle(rect.x as i32 + 4, wheel_y2, 8, 4, Color::BLACK);

        let back_x = (rect.x + rect.width) as i32 - 12;
        d.draw_rectangle(back_x, wheel_y1, 8, 4, Color::BLACK);
        d.draw_rectangle(back_x, wheel_y2, 8, 4, Color::BLACK);

        // Windows
        let window_x = rect.x as i32 + (rect.width as i32) / 3;
        let window_w = rect.width as i32 / 3;
        d.draw_rectangle(
            window_x,
            rect.y as i32 + 6,
            window_w,
            CELL_SIZE - 12,
            Color::LIGHTBLUE,
        );
    }
}

fn draw_frog(d: &mut RaylibDrawHandle, frog: &Frog) {
    let x = frog.get_pixel_x() as i32;
    let y = frog.get_pixel_y() as i32;
    let center_x = x + CELL_SIZE / 2;
    let center_y = y + CELL_SIZE / 2;

    // Body
    d.draw_circle(center_x, center_y, (CELL_SIZE / 2 - 4) as f32, Color::GREEN);

    // Eyes
    d.draw_circle(center_x - 6, center_y - 6, 4.0, Color::WHITE);
    d.draw_circle(center_x + 6, center_y - 6, 4.0, Color::WHITE);
    d.draw_circle(center_x - 6, center_y - 6, 2.0, Color::BLACK);
    d.draw_circle(center_x + 6, center_y - 6, 2.0, Color::BLACK);

    // Legs (simple)
    d.draw_rectangle(x + 2, y + CELL_SIZE - 8, 6, 6, Color::DARKGREEN);
    d.draw_rectangle(x + CELL_SIZE - 8, y + CELL_SIZE - 8, 6, 6, Color::DARKGREEN);
}

fn draw_hud(d: &mut RaylibDrawHandle, lives: i32, score: i32) {
    // Lives
    d.draw_text("Lives:", 10, SCREEN_HEIGHT - 28, 20, Color::WHITE);
    for i in 0..lives {
        d.draw_circle(80 + i * 25, SCREEN_HEIGHT - 18, 8.0, Color::GREEN);
    }

    // Score
    let score_text = format!("Score: {}", score);
    d.draw_text(&score_text, SCREEN_WIDTH - 150, SCREEN_HEIGHT - 28, 20, Color::WHITE);
}

fn draw_game_over(d: &mut RaylibDrawHandle, won: bool) {
    let overlay = Color::new(0, 0, 0, 180);
    d.draw_rectangle(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, overlay);

    let message = if won { "YOU WIN!" } else { "GAME OVER" };
    let text_width = measure_text(message, 40);
    d.draw_text(
        message,
        SCREEN_WIDTH / 2 - text_width / 2,
        SCREEN_HEIGHT / 2 - 40,
        40,
        if won { Color::GOLD } else { Color::RED },
    );

    let restart = "Press SPACE to restart";
    let restart_width = measure_text(restart, 20);
    d.draw_text(
        restart,
        SCREEN_WIDTH / 2 - restart_width / 2,
        SCREEN_HEIGHT / 2 + 20,
        20,
        Color::WHITE,
    );
}

// =============================================================================
// Main
// =============================================================================

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Frogger - Rust/raylib")
        .vsync()
        .build();

    rl.set_target_fps(60);

    let mut game = GameState::new();

    while !rl.window_should_close() {
        // Input
        if game.game_over {
            if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
                game.reset();
            }
        } else {
            if rl.is_key_pressed(KeyboardKey::KEY_UP) || rl.is_key_pressed(KeyboardKey::KEY_W) {
                game.frog.move_up();
            }
            if rl.is_key_pressed(KeyboardKey::KEY_DOWN) || rl.is_key_pressed(KeyboardKey::KEY_S) {
                game.frog.move_down();
            }
            if rl.is_key_pressed(KeyboardKey::KEY_LEFT) || rl.is_key_pressed(KeyboardKey::KEY_A) {
                game.frog.move_left();
            }
            if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) || rl.is_key_pressed(KeyboardKey::KEY_D) {
                game.frog.move_right();
            }
        }

        // Update
        let dt = rl.get_frame_time();
        game.update(dt);

        // Draw
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);

        draw_background(&mut d);
        draw_goals(&mut d, &game.goals);
        draw_logs(&mut d, &game.logs);
        draw_cars(&mut d, &game.cars);
        draw_frog(&mut d, &game.frog);
        draw_hud(&mut d, game.lives, game.score);

        if game.game_over {
            draw_game_over(&mut d, game.won);
        }
    }
}
