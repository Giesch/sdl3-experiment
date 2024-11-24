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

use std::ffi::{c_char, CStr, CString};
use std::mem::zeroed;
use std::ptr::{null, null_mut};
use std::sync::Mutex;

use sdl3_main::{app_event, app_init, app_iterate, app_quit, AppResult};
use sdl3_sys::everything::*;

use sdl3_experiment::GameState;

const BLOCK_SIZE_IN_PIXELS: i32 = 24;
const SDL_WINDOW_WIDTH: i32 = BLOCK_SIZE_IN_PIXELS * GAME_WIDTH as i32;
const SDL_WINDOW_HEIGHT: i32 = BLOCK_SIZE_IN_PIXELS * GAME_HEIGHT as i32;

const GAME_WIDTH: i8 = 24;
const GAME_HEIGHT: i8 = 18;

struct AppState {
    window: *mut SDL_Window,
    device: *mut SDL_GPUDevice,
    fill_pipeline: *mut SDL_GPUGraphicsPipeline,
    line_pipeline: *mut SDL_GPUGraphicsPipeline,
    game_state: GameState,
}

impl Drop for AppState {
    fn drop(&mut self) {
        unsafe {
            if !self.fill_pipeline.is_null() {
                SDL_ReleaseGPUGraphicsPipeline(self.device, self.fill_pipeline);
            }
            if !self.line_pipeline.is_null() {
                SDL_ReleaseGPUGraphicsPipeline(self.device, self.line_pipeline);
            }

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
            let mut color_target_info: SDL_GPUColorTargetInfo = zeroed();
            color_target_info.texture = swapchain_texture;
            color_target_info.clear_color = SDL_FColor {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            };
            color_target_info.load_op = SDL_GPU_LOADOP_CLEAR;
            color_target_info.store_op = SDL_GPU_STOREOP_STORE;

            let num_color_targets = 1;
            let depth_stencil_target_info = null_mut();
            let render_pass = SDL_BeginGPURenderPass(
                command_buffer,
                &color_target_info,
                num_color_targets,
                depth_stencil_target_info,
            );

            let pipeline = if app.game_state.use_wire_frame_mode {
                app.line_pipeline
            } else {
                app.fill_pipeline
            };
            SDL_BindGPUGraphicsPipeline(render_pass, pipeline);

            if app.game_state.use_small_viewport {
                let small_viewport = SDL_GPUViewport {
                    x: 160.0,
                    y: 120.0,
                    w: 320.0,
                    h: 240.0,
                    min_depth: 0.1,
                    max_depth: 1.0,
                };
                SDL_SetGPUViewport(render_pass, &small_viewport);
            }

            if app.game_state.use_scissor_rect {
                let scissor = SDL_Rect {
                    x: 320,
                    y: 240,
                    w: 320,
                    h: 240,
                };
                SDL_SetGPUScissor(render_pass, &scissor);
            }

            SDL_DrawGPUPrimitives(render_pass, 3, 1, 0, 0);
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
            c"GPU Window".as_ptr(),
            SDL_WINDOW_WIDTH,
            SDL_WINDOW_HEIGHT,
            SDL_WINDOW_VULKAN,
        );
        if window.is_null() {
            dbg_sdl_error("SDL_CreateWindow failed");
            return None;
        }

        let format_flags = SDL_GPU_SHADERFORMAT_SPIRV;
        let device = SDL_CreateGPUDevice(format_flags, true, null());
        if device.is_null() {
            dbg_sdl_error("SDL_CreateGPUDevice failed");
            return None;
        }
        if !SDL_ClaimWindowForGPUDevice(device, window) {
            dbg_sdl_error("SDL_ClaimWindowForGPUDevice failed");
            return None;
        }

        let vert_shader = load_shader(
            device,
            LoadShaderRequest {
                file_name: "RawTriangle.vert",
                shader_stage: SDL_GPU_SHADERSTAGE_VERTEX,
                sampler_count: 0,
                uniform_buffer_count: 0,
                storage_buffer_count: 0,
                storage_texture_count: 0,
            },
        );
        if vert_shader.is_null() {
            dbg_sdl_error("failed to load vert shader");
            return None;
        }

        let frag_shader = load_shader(
            device,
            LoadShaderRequest {
                file_name: "SolidColor.frag",
                shader_stage: SDL_GPU_SHADERSTAGE_FRAGMENT,
                sampler_count: 0,
                uniform_buffer_count: 0,
                storage_buffer_count: 0,
                storage_texture_count: 0,
            },
        );
        if frag_shader.is_null() {
            dbg_sdl_error("failed to load frag shader");
            return None;
        }

        let mut pipeline_create_info = SDL_GPUGraphicsPipelineCreateInfo {
            vertex_shader: vert_shader,
            fragment_shader: frag_shader,
            primitive_type: SDL_GPU_PRIMITIVETYPE_TRIANGLELIST,
            target_info: SDL_GPUGraphicsPipelineTargetInfo {
                num_color_targets: 1,
                color_target_descriptions: [SDL_GPUColorTargetDescription {
                    format: SDL_GetGPUSwapchainTextureFormat(device, window),
                    ..zeroed()
                }]
                .as_ptr(),
                ..zeroed()
            },
            ..zeroed()
        };

        pipeline_create_info.rasterizer_state.fill_mode = SDL_GPU_FILLMODE_FILL;
        let fill_pipeline = SDL_CreateGPUGraphicsPipeline(device, &pipeline_create_info);
        if fill_pipeline.is_null() {
            dbg_sdl_error("failed to create fill pipeline");
            return None;
        }

        pipeline_create_info.rasterizer_state.fill_mode = SDL_GPU_FILLMODE_LINE;
        let line_pipeline = SDL_CreateGPUGraphicsPipeline(device, &pipeline_create_info);
        if line_pipeline.is_null() {
            dbg_sdl_error("failed to create line pipeline");
            return None;
        }

        SDL_ReleaseGPUShader(device, vert_shader);
        SDL_ReleaseGPUShader(device, frag_shader);

        let app = AppState {
            window,
            device,
            fill_pipeline,
            line_pipeline,
            game_state: GameState::new(),
        };

        println!("Press Left to toggle wireframe mode");
        println!("Press Down to toggle small viewport");
        println!("Press Right to toggle scissor rect");

        Some(Box::new(Mutex::new(app)))
    }
}

struct LoadShaderRequest {
    file_name: &'static str,
    shader_stage: SDL_GPUShaderStage,
    sampler_count: u32,
    uniform_buffer_count: u32,
    storage_buffer_count: u32,
    storage_texture_count: u32,
}

unsafe fn load_shader(device: *mut SDL_GPUDevice, req: LoadShaderRequest) -> *mut SDL_GPUShader {
    let full_path = format!("./content/shaders/compiled/{}.spv", req.file_name);

    let full_path = CString::new(full_path).unwrap();
    let mut code_size = 0;

    let loaded_code = SDL_LoadFile(full_path.as_ptr(), &mut code_size);
    if loaded_code.is_null() || code_size == 0 {
        dbg_sdl_error(&format!("failed to load shader: {}", req.file_name));
        return null_mut();
    }

    let shader_info = SDL_GPUShaderCreateInfo {
        code: loaded_code as *const u8,
        code_size,
        entrypoint: c"main".as_ptr(),
        format: SDL_GPU_SHADERFORMAT_SPIRV,
        stage: req.shader_stage,
        num_samplers: req.sampler_count,
        num_storage_buffers: req.storage_buffer_count,
        num_uniform_buffers: req.uniform_buffer_count,
        num_storage_textures: req.storage_texture_count,
        props: 0,
    };
    let shader = SDL_CreateGPUShader(device, &shader_info);
    if shader.is_null() {
        dbg_sdl_error(&format!("failed to create shader: {}", req.file_name));
        SDL_free(loaded_code);
        return null_mut();
    }

    SDL_free(loaded_code);

    shader
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
