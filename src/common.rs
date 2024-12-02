use std::ffi::{c_char, CStr, CString};
use std::ptr::{null, null_mut};

use sdl3_sys::everything::*;

use serde::Deserialize;

/// Load a precompiled shader based on file name.
/// Relies on the structure of the content directory and the file name suffix.
pub unsafe fn load_shader(
    device: *mut SDL_GPUDevice,
    file_name: &'static str,
) -> *mut SDL_GPUShader {
    const COMPILED_SHADERS_DIR: &'static str = "./content/shaders/compiled";

    let backend_formats = SDL_GetGPUShaderFormats(device);
    let (format, entrypoint) = if backend_formats & SDL_GPU_SHADERFORMAT_SPIRV != 0 {
        (SDL_GPU_SHADERFORMAT_SPIRV, c"main".as_ptr())
    } else if backend_formats & SDL_GPU_SHADERFORMAT_MSL != 0 {
        (SDL_GPU_SHADERFORMAT_MSL, c"main0".as_ptr())
    } else if backend_formats & SDL_GPU_SHADERFORMAT_DXIL != 0 {
        (SDL_GPU_SHADERFORMAT_DXIL, c"main".as_ptr())
    } else {
        println!("unrecognized backend shader format");
        return null_mut();
    };

    let full_path = format!("{COMPILED_SHADERS_DIR}/spv/{file_name}.spv");
    let full_path = CString::new(full_path).unwrap();
    let mut code_size = 0;
    let loaded_code = SDL_LoadFile(full_path.as_ptr(), &mut code_size);
    if loaded_code.is_null() {
        dbg_sdl_error(&format!("failed to load shader: {file_name}"));
        return null_mut();
    }

    let json_path = format!("{COMPILED_SHADERS_DIR}/json/{file_name}.json");
    let Ok(json) = std::fs::read_to_string(&json_path) else {
        println!("failed to find shader json: {json_path}");
        return null_mut();
    };
    let meta = match serde_json::from_str::<ShaderMeta>(&json) {
        Ok(meta) => meta,
        Err(e) => {
            println!("invalid shader json: {e} {json_path}");
            return null_mut();
        }
    };

    let stage = if file_name.ends_with(".vert") {
        SDL_GPUShaderStage::VERTEX
    } else if file_name.ends_with(".frag") {
        SDL_GPUShaderStage::FRAGMENT
    } else {
        panic!("expected a file name ending in '.frag' or '.vert'")
    };

    let shader_info = SDL_GPUShaderCreateInfo {
        code: loaded_code as *const u8,
        stage,
        entrypoint,
        format,
        code_size,
        num_samplers: meta.samplers,
        num_storage_buffers: meta.storage_buffers,
        num_uniform_buffers: meta.uniform_buffers,
        num_storage_textures: meta.storage_textures,
        props: 0,
    };
    let shader = SDL_CreateGPUShader(device, &shader_info);
    if shader.is_null() {
        dbg_sdl_error(&format!("failed to create shader: {}", file_name));
        SDL_free(loaded_code);
        return null_mut();
    }

    SDL_free(loaded_code);

    shader
}

pub unsafe fn dbg_sdl_error(msg: &str) {
    println!("{}", msg);
    let error = CStr::from_ptr(SDL_GetError()).to_string_lossy();
    dbg!(&error);
}

/// JSON format for resource counts emitted by shadercross
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ShaderMeta {
    samplers: u32,
    storage_textures: u32,
    storage_buffers: u32,
    uniform_buffers: u32,
}

pub fn init_gpu_window(
    window_title: *const c_char,
    window_flags: SDL_WindowFlags,
) -> Option<(*mut SDL_Window, *mut SDL_GPUDevice)> {
    unsafe {
        if !SDL_Init(SDL_INIT_VIDEO) {
            dbg_sdl_error("SDL_Init failed");
            return None;
        }

        let window = SDL_CreateWindow(window_title, 640, 480, window_flags);
        if window.is_null() {
            dbg_sdl_error("SDL_CreateWindow failed");
            return None;
        }

        let format_flags =
            SDL_GPU_SHADERFORMAT_SPIRV | SDL_GPU_SHADERFORMAT_DXIL | SDL_GPU_SHADERFORMAT_MSL;
        let device = SDL_CreateGPUDevice(format_flags, true, null());
        if device.is_null() {
            dbg_sdl_error("SDL_CreateGPUDevice failed");
            return None;
        }
        if !SDL_ClaimWindowForGPUDevice(device, window) {
            dbg_sdl_error("SDL_ClaimWindowForGPUDevice failed");
            return None;
        }

        Some((window, device))
    }
}

pub unsafe fn deinit_gpu_window(device: *mut SDL_GPUDevice, window: *mut SDL_Window) {
    if !device.is_null() && !device.is_null() {
        SDL_ReleaseWindowFromGPUDevice(device, window);
    }
    if !window.is_null() {
        SDL_DestroyWindow(window);
    }
    if !device.is_null() {
        SDL_DestroyGPUDevice(device);
    }
}
