use sdl3_sys::scancode::SDL_Scancode;

/// about 60fps
const STEP_RATE_IN_MILLISECONDS: u64 = 16;
const BLOCK_SIZE_IN_PIXELS: i32 = 24;

pub const WINDOW_WIDTH: i32 = BLOCK_SIZE_IN_PIXELS * GAME_WIDTH as i32;
pub const WINDOW_HEIGHT: i32 = BLOCK_SIZE_IN_PIXELS * GAME_HEIGHT as i32;

const GAME_WIDTH: i8 = 24;
const GAME_HEIGHT: i8 = 18;

pub struct GameState {
    accumulated_ticks: u64,
    last_step: u64,
    keys_down: Vec<SDL_Scancode>,
    keys_just_pressed: Vec<SDL_Scancode>,

    pub use_wire_frame_mode: bool,
    pub use_small_viewport: bool,
    pub use_scissor_rect: bool,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            accumulated_ticks: 0,
            last_step: 0,
            keys_down: Default::default(),
            keys_just_pressed: Default::default(),

            use_wire_frame_mode: false,
            use_small_viewport: false,
            use_scissor_rect: false,
        }
    }

    pub fn key_pressed(&mut self, scan_code: SDL_Scancode) {
        if !self.keys_down.contains(&scan_code) {
            self.keys_down.push(scan_code);
            self.keys_just_pressed.push(scan_code);
        }
    }

    pub fn key_released(&mut self, scan_code: SDL_Scancode) {
        self.keys_down.retain(|k| *k != scan_code);
        self.keys_just_pressed.retain(|k| *k != scan_code);
    }

    pub fn step(&mut self, ticks: u64) {
        let new_ticks = ticks - self.last_step;
        self.accumulated_ticks += new_ticks;
        self.last_step = ticks;

        while self.accumulated_ticks >= STEP_RATE_IN_MILLISECONDS {
            self.accumulated_ticks -= STEP_RATE_IN_MILLISECONDS;
            self.fixed_step();

            self.keys_just_pressed.clear();
        }
    }

    fn fixed_step(&mut self) {
        if self.keys_just_pressed.contains(&SDL_Scancode::LEFT) {
            self.use_wire_frame_mode = !self.use_wire_frame_mode;
        }
        if self.keys_just_pressed.contains(&SDL_Scancode::DOWN) {
            self.use_small_viewport = !self.use_small_viewport;
        }
        if self.keys_just_pressed.contains(&SDL_Scancode::RIGHT) {
            self.use_scissor_rect = !self.use_scissor_rect;
        }
    }
}
