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
use std::ptr::null_mut;
use std::sync::Mutex;

use sdl3_main::{app_event, app_init, app_iterate, app_quit, AppResult};

use sdl3_sys::gpu::SDL_AcquireGPUCommandBuffer;
use sdl3_sys::gpu::SDL_AcquireGPUSwapchainTexture;
use sdl3_sys::gpu::SDL_BeginGPURenderPass;
use sdl3_sys::gpu::SDL_ClaimWindowForGPUDevice;
use sdl3_sys::gpu::SDL_DestroyGPUDevice;
use sdl3_sys::gpu::SDL_EndGPURenderPass;
use sdl3_sys::gpu::SDL_GPUColorTargetInfo;
use sdl3_sys::gpu::SDL_GPUDevice;
use sdl3_sys::gpu::SDL_GPULoadOp;
use sdl3_sys::gpu::SDL_GPUStoreOp;
use sdl3_sys::gpu::SDL_GPUTexture;
use sdl3_sys::gpu::SDL_ReleaseWindowFromGPUDevice;
use sdl3_sys::gpu::SDL_SubmitGPUCommandBuffer;
use sdl3_sys::gpu::{SDL_CreateGPUDevice, SDL_GPU_SHADERFORMAT_SPIRV};

use sdl3_sys::pixels::SDL_FColor;
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
    timer::SDL_GetTicks,
    video::{SDL_DestroyWindow, SDL_Window},
};

use sdl3_experiment::GameState;

const BLOCK_SIZE_IN_PIXELS: i32 = 24;
const SDL_WINDOW_WIDTH: i32 = BLOCK_SIZE_IN_PIXELS * GAME_WIDTH as i32;
const SDL_WINDOW_HEIGHT: i32 = BLOCK_SIZE_IN_PIXELS * GAME_HEIGHT as i32;

const GAME_WIDTH: i8 = 24;
const GAME_HEIGHT: i8 = 18;

struct AppState {
    window: *mut SDL_Window,
    device: *mut SDL_GPUDevice,
    game_state: GameState,
}

impl Drop for AppState {
    fn drop(&mut self) {
        unsafe {
            if !self.device.is_null() && !self.device.is_null() {
                SDL_ReleaseWindowFromGPUDevice(self.device, self.window);
            }
            if !self.window.is_null() {
                SDL_DestroyWindow(self.window);
            }
            if !self.device.is_null() {
                SDL_DestroyGPUDevice(self.device);
            }
        }
    }
}

unsafe impl Send for AppState {}

impl AppState {
    fn new(window: *mut SDL_Window, device: *mut SDL_GPUDevice) -> Self {
        let game_state = GameState::new();

        Self {
            window,
            device,
            game_state,
        }
    }
}

#[app_iterate]
fn app_iterate(app: &mut AppState) -> AppResult {
    unsafe {
        let ticks = SDL_GetTicks();
        app.game_state.step(ticks);

        let command_buffer = SDL_AcquireGPUCommandBuffer(app.device);
        if command_buffer.is_null() {
            dbg_sdl_error("failed to acquire command buffer");
            return AppResult::Failure;
        }

        let mut swapchain_texture: *mut SDL_GPUTexture = null_mut();
        if !SDL_AcquireGPUSwapchainTexture(
            command_buffer,
            app.window,
            &mut swapchain_texture,
            null_mut(),
            null_mut(),
        ) {
            dbg_sdl_error("failed to acquire swapchain texture");
            return AppResult::Failure;
        }

        if !swapchain_texture.is_null() {
            let color_target_info = SDL_GPUColorTargetInfo {
                texture: swapchain_texture,
                clear_color: SDL_FColor {
                    r: 0.3,
                    g: 0.4,
                    b: 0.5,
                    a: 1.0,
                },
                load_op: SDL_GPULoadOp::CLEAR,
                store_op: SDL_GPUStoreOp::STORE,
                // TODO: implement Default?
                cycle: false,
                cycle_resolve_texture: false,
                resolve_texture: null_mut(),
                mip_level: 0,
                layer_or_depth_plane: 0,
                resolve_mip_level: 0,
                resolve_layer: 0,
                padding1: 0,
                padding2: 0,
            };

            let num_color_targets = 1;
            let depth_stencil_target_info = null_mut();
            let render_pass = SDL_BeginGPURenderPass(
                command_buffer,
                &color_target_info,
                num_color_targets,
                depth_stencil_target_info,
            );
            SDL_EndGPURenderPass(render_pass);
        }

        SDL_SubmitGPUCommandBuffer(command_buffer);
    }

    AppResult::Continue
}

const EXTENDED_METADATA: &[(*const c_char, *const c_char)] = &[
    (SDL_PROP_APP_METADATA_URL_STRING, c"giesch.dev".as_ptr()),
    (
        SDL_PROP_APP_METADATA_CREATOR_STRING,
        c"Dan Knutson".as_ptr(),
    ),
    (
        SDL_PROP_APP_METADATA_COPYRIGHT_STRING,
        c"Copyright Dan Knutson 2024".as_ptr(),
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
            dbg_sdl_error("SDL_SetAppMetaData failed");
            return None;
        }

        for (key, value) in EXTENDED_METADATA.iter().copied() {
            if !SDL_SetAppMetadataProperty(key, value) {
                dbg_sdl_error("SDL_SetAppMetadataProperty failed");
                return None;
            }
        }

        if !SDL_Init(SDL_INIT_VIDEO) {
            dbg_sdl_error("SDL_Init failed");
            return None;
        }

        // GPU EXPERIMENTING

        let window = SDL_CreateWindow(
            c"GPU Window?".as_ptr(),
            SDL_WINDOW_WIDTH,
            SDL_WINDOW_HEIGHT,
            SDL_WINDOW_VULKAN,
        );
        if window.is_null() {
            dbg_sdl_error("SDL_CreateWindow failed");
            return None;
        }

        let format_flags = SDL_GPU_SHADERFORMAT_SPIRV;
        let device = SDL_CreateGPUDevice(format_flags, true, std::ptr::null());
        if device.is_null() {
            dbg_sdl_error("SDL_CreateGPUDevice failed");
            return None;
        }
        if !SDL_ClaimWindowForGPUDevice(device, window) {
            dbg_sdl_error("SDL_ClaimWindowForGPUDevice failed");
            return None;
        }

        let app = AppState::new(window, device);

        Some(Box::new(Mutex::new(app)))
    }
}

unsafe fn dbg_sdl_error(msg: &str) {
    println!("{}", msg);
    let error = CStr::from_ptr(SDL_GetError()).to_string_lossy();
    dbg!(&error);
}

#[app_event]
fn app_event(app: &mut AppState, event: &SDL_Event) -> AppResult {
    unsafe {
        match SDL_EventType(event.r#type) {
            SDL_EVENT_QUIT => AppResult::Success,

            SDL_EVENT_KEY_DOWN => {
                app.game_state.key_pressed(event.key.scancode);
                AppResult::Continue
            }

            SDL_EVENT_KEY_UP => {
                app.game_state.key_released(event.key.scancode);
                AppResult::Continue
            }

            _ => AppResult::Continue,
        }
    }
}

#[app_quit]
fn app_quit() {}
