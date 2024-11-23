/// about 60fps
const STEP_RATE_IN_MILLISECONDS: u64 = 16;
const BLOCK_SIZE_IN_PIXELS: i32 = 24;
/// about half a block per frame
const MOVE_SPEED: f32 = 12.0;

pub const WINDOW_WIDTH: i32 = BLOCK_SIZE_IN_PIXELS * GAME_WIDTH as i32;
pub const WINDOW_HEIGHT: i32 = BLOCK_SIZE_IN_PIXELS * GAME_HEIGHT as i32;

const GAME_WIDTH: i8 = 24;
const GAME_HEIGHT: i8 = 18;

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum KeyCode {
    Esc,
    Q,
    R,
    Right,
    Up,
    Left,
    Down,
}

pub enum GameEvent {
    Quit,
    KeyDown(KeyCode),
}

pub struct GameState {
    player_x: f32,
    player_y: f32,
    accumulated_ticks: u64,
    last_step: u64,
    keys_down: Vec<KeyCode>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            player_x: BLOCK_SIZE_IN_PIXELS as f32 * (GAME_WIDTH as f32 / 2.0),
            player_y: BLOCK_SIZE_IN_PIXELS as f32 * (GAME_HEIGHT as f32 / 2.0),
            accumulated_ticks: 0,
            last_step: 0,
            keys_down: Default::default(),
        }
    }

    pub fn step(&mut self, ticks: u64) {
        let new_ticks = ticks - self.last_step;
        self.accumulated_ticks += new_ticks;
        self.last_step = ticks;

        while self.accumulated_ticks >= STEP_RATE_IN_MILLISECONDS {
            self.accumulated_ticks -= STEP_RATE_IN_MILLISECONDS;
            self.fixed_step();
        }
    }

    fn fixed_step(&mut self) {
        if self.keys_down.contains(&KeyCode::Up) {
            self.player_y -= MOVE_SPEED;
        }
        if self.keys_down.contains(&KeyCode::Down) {
            self.player_y += MOVE_SPEED;
        }
        if self.keys_down.contains(&KeyCode::Left) {
            self.player_x -= MOVE_SPEED;
        }
        if self.keys_down.contains(&KeyCode::Right) {
            self.player_x += MOVE_SPEED;
        }
    }

    pub fn key_pressed(&mut self, key_code: KeyCode) {
        if !self.keys_down.contains(&key_code) {
            self.keys_down.push(key_code);
        }
    }

    pub fn key_released(&mut self, key_code: KeyCode) {
        self.keys_down.retain(|k| *k != key_code)
    }

    pub fn player_rect(&self) -> GameRect {
        GameRect {
            x: self.player_x,
            y: self.player_y,
            w: BLOCK_SIZE_IN_PIXELS as f32,
            h: BLOCK_SIZE_IN_PIXELS as f32,
        }
    }
}

pub struct GameRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}
