// This is a Rust port of examples/game/01-snake from SDL.
//
// While it uses some Rust concepts, it's not intended to be idiomatic Rust,
// but rather a close translation of the original.
//
// Like the original example, this code is public domain.
//
// original description:
//
// Logic implementation of the Snake game. It is designed to efficiently
// represent the state of the game in memory.
//
// This code is public domain. Feel free to use it for any purpose!

use core::ffi::c_char;
use core::ffi::CStr;
use std::sync::Mutex;

use sdl3_experiment::{GameState, KeyCode};
use sdl3_main::{app_event, app_init, app_iterate, app_quit, AppResult};

use sdl3_sys::gpu::SDL_ClaimWindowForGPUDevice;
use sdl3_sys::gpu::SDL_GPUSupportsProperties;
use sdl3_sys::gpu::{SDL_CreateGPUDevice, SDL_GPU_SHADERFORMAT_SPIRV};

use sdl3_sys::render::SDL_CreateWindowAndRenderer;
use sdl3_sys::video::SDL_CreateWindow;
use sdl3_sys::video::SDL_WINDOW_VULKAN;
// You can `use sdl3_sys::everything::*` if you don't want to specify everything explicitly
use sdl3_sys::{
    error::SDL_GetError,
    events::{SDL_Event, SDL_EventType, SDL_EVENT_KEY_DOWN, SDL_EVENT_KEY_UP, SDL_EVENT_QUIT},
    init::{
        SDL_Init, SDL_SetAppMetadata, SDL_SetAppMetadataProperty, SDL_INIT_VIDEO,
        SDL_PROP_APP_METADATA_COPYRIGHT_STRING, SDL_PROP_APP_METADATA_CREATOR_STRING,
        SDL_PROP_APP_METADATA_TYPE_STRING, SDL_PROP_APP_METADATA_URL_STRING,
    },
    pixels::SDL_ALPHA_OPAQUE,
    rect::SDL_FRect,
    render::{
        SDL_DestroyRenderer, SDL_RenderClear, SDL_RenderFillRect, SDL_RenderPresent, SDL_Renderer,
        SDL_SetRenderDrawColor,
    },
    scancode::{
        SDL_Scancode, SDL_SCANCODE_DOWN, SDL_SCANCODE_ESCAPE, SDL_SCANCODE_LEFT, SDL_SCANCODE_Q,
        SDL_SCANCODE_R, SDL_SCANCODE_RIGHT, SDL_SCANCODE_UP,
    },
    timer::SDL_GetTicks,
    video::{SDL_DestroyWindow, SDL_Window},
};

const BLOCK_SIZE_IN_PIXELS: i32 = 24;
const SDL_WINDOW_WIDTH: i32 = BLOCK_SIZE_IN_PIXELS * GAME_WIDTH as i32;
const SDL_WINDOW_HEIGHT: i32 = BLOCK_SIZE_IN_PIXELS * GAME_HEIGHT as i32;

const GAME_WIDTH: i8 = 24;
const GAME_HEIGHT: i8 = 18;

struct AppState {
    window: *mut SDL_Window,
    renderer: *mut SDL_Renderer,
    game_state: GameState,
}

impl Drop for AppState {
    fn drop(&mut self) {
        unsafe {
            if !self.renderer.is_null() {
                SDL_DestroyRenderer(self.renderer);
            }
            if !self.window.is_null() {
                SDL_DestroyWindow(self.window);
            }
        }
    }
}

unsafe impl Send for AppState {}

impl AppState {
    fn new() -> Self {
        Self {
            window: core::ptr::null_mut(),
            renderer: core::ptr::null_mut(),
            game_state: GameState::new(),
        }
    }
}

fn translate_scan_code(scan_code: SDL_Scancode) -> Option<KeyCode> {
    match scan_code {
        SDL_SCANCODE_ESCAPE => Some(KeyCode::Esc),
        SDL_SCANCODE_Q => Some(KeyCode::Q),
        SDL_SCANCODE_R => Some(KeyCode::R),
        SDL_SCANCODE_RIGHT => Some(KeyCode::Right),
        SDL_SCANCODE_UP => Some(KeyCode::Up),
        SDL_SCANCODE_LEFT => Some(KeyCode::Left),
        SDL_SCANCODE_DOWN => Some(KeyCode::Down),
        _ => None,
    }
}

#[app_iterate]
fn app_iterate(app: &mut AppState) -> AppResult {
    unsafe {
        let ticks = SDL_GetTicks();
        app.game_state.step(ticks);

        SDL_SetRenderDrawColor(app.renderer, 0, 0, 0, SDL_ALPHA_OPAQUE);
        SDL_RenderClear(app.renderer);

        SDL_SetRenderDrawColor(app.renderer, 0, 128, 0, SDL_ALPHA_OPAQUE);
        let player_rect = app.game_state.player_rect();
        let player_rect = SDL_FRect {
            x: player_rect.x,
            y: player_rect.y,
            w: player_rect.w,
            h: player_rect.h,
        };
        SDL_RenderFillRect(app.renderer, &player_rect);

        SDL_RenderPresent(app.renderer);
    }

    AppResult::Continue
}

const EXTENDED_METADATA: &[(*const c_char, *const c_char)] = &[
    (SDL_PROP_APP_METADATA_URL_STRING, c"TODO url".as_ptr()),
    (
        SDL_PROP_APP_METADATA_CREATOR_STRING,
        c"TODO creator".as_ptr(),
    ),
    (
        SDL_PROP_APP_METADATA_COPYRIGHT_STRING,
        c"TODO copyright".as_ptr(),
    ),
    (SDL_PROP_APP_METADATA_TYPE_STRING, c"game".as_ptr()),
];

#[app_init]
fn app_init() -> Option<Box<Mutex<AppState>>> {
    unsafe {
        if !SDL_SetAppMetadata(
            c"Example Rust SDL3 game".as_ptr(),
            c"0.0".as_ptr(),
            c"dev.giesch.Example".as_ptr(),
        ) {
            return dbg_sdl_error();
        }

        for (key, value) in EXTENDED_METADATA.iter().copied() {
            if !SDL_SetAppMetadataProperty(key, value) {
                return dbg_sdl_error();
            }
        }

        if !SDL_Init(SDL_INIT_VIDEO) {
            return dbg_sdl_error();
        }

        let mut app = AppState::new();
        if !SDL_CreateWindowAndRenderer(
            c"SDL3 Experiment".as_ptr(),
            SDL_WINDOW_WIDTH,
            SDL_WINDOW_HEIGHT,
            0,
            &mut app.window,
            &mut app.renderer,
        ) {
            return dbg_sdl_error();
        }

        // GPU EXPERIMENTING

        // let window = SDL_CreateWindow(
        //     c"GPU Window?".as_ptr(),
        //     SDL_WINDOW_WIDTH,
        //     SDL_WINDOW_HEIGHT,
        //     SDL_WINDOW_VULKAN,
        // );
        // if window.is_null() {
        //     return dbg_sdl_error();
        // }

        // let format_flags = SDL_GPU_SHADERFORMAT_SPIRV;
        // let device = SDL_CreateGPUDevice(format_flags, true, std::ptr::null());
        // if device.is_null() {
        //     return dbg_sdl_error();
        // }
        // if !SDL_ClaimWindowForGPUDevice(device, window) {
        //     return dbg_sdl_error();
        // }

        Some(Box::new(Mutex::new(app)))
    }
}

unsafe fn dbg_sdl_error() -> Option<Box<Mutex<AppState>>> {
    let error = CStr::from_ptr(SDL_GetError()).to_string_lossy();
    dbg!(&error);
    None
}

#[app_event]
fn app_event(app: &mut AppState, event: &SDL_Event) -> AppResult {
    unsafe {
        match SDL_EventType(event.r#type) {
            SDL_EVENT_QUIT => AppResult::Success,

            SDL_EVENT_KEY_DOWN => {
                if let Some(key_code) = translate_scan_code(event.key.scancode) {
                    app.game_state.key_pressed(key_code);
                }
                AppResult::Continue
            }

            SDL_EVENT_KEY_UP => {
                if let Some(key_code) = translate_scan_code(event.key.scancode) {
                    app.game_state.key_released(key_code);
                }
                AppResult::Continue
            }

            _ => AppResult::Continue,
        }
    }
}

#[app_quit]
fn app_quit() {}
