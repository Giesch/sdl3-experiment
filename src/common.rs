use std::ffi::{CStr, CString, c_char};
use std::ptr::{null, null_mut};

use sdl3_sys::everything::*;

use serde::Deserialize;

/// Load a precompiled shader based on file name.
/// Relies on the structure of the content directory, json metadata, and the file name suffix.
pub unsafe fn load_shader(
    device: *mut SDL_GPUDevice,
    shader_name: &'static str,
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

    let full_path = format!("{COMPILED_SHADERS_DIR}/spv/{shader_name}.spv");
    let full_path = CString::new(full_path).unwrap();
    let mut code_size = 0;
    let loaded_code = SDL_LoadFile(full_path.as_ptr(), &mut code_size);
    if loaded_code.is_null() {
        dbg_sdl_error(&format!("failed to load shader: {shader_name}"));
        return null_mut();
    }

    let json_path = format!("{COMPILED_SHADERS_DIR}/json/{shader_name}.json");
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

    let stage = if shader_name.ends_with(".vert") {
        SDL_GPUShaderStage::VERTEX
    } else if shader_name.ends_with(".frag") {
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
        props: SDL_PropertiesID::default(),
    };
    let shader = SDL_CreateGPUShader(device, &shader_info);
    if shader.is_null() {
        dbg_sdl_error(&format!("failed to create shader: {}", shader_name));
        SDL_free(loaded_code);
        return null_mut();
    }

    SDL_free(loaded_code);

    shader
}

pub unsafe fn dbg_sdl_error(msg: &str) {
    #[cfg(debug_assertions)]
    {
        println!("{}", msg);
        let error = CStr::from_ptr(SDL_GetError()).to_string_lossy();
        println!("{}", &error);
    }
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

pub unsafe fn load_bmp(file_name: &str) -> *mut SDL_Surface {
    const IMAGES_DIR: &'static str = "./content/images";

    let full_path = format!("{IMAGES_DIR}/{file_name}");
    let full_path = CString::new(full_path).unwrap();

    let mut result = SDL_LoadBMP(full_path.as_ptr());
    if result.is_null() {
        return result;
    }

    // NOTE this is only the '4 channels' path of the original example
    let format = SDL_PixelFormat::ARGB8888;
    if (*result).format != format {
        let next = SDL_ConvertSurface(result, format);
        SDL_DestroySurface(result);
        result = next;
    }

    result
}

pub struct Matrix4x4 {
    pub m11: f32,
    pub m12: f32,
    pub m13: f32,
    pub m14: f32,

    pub m21: f32,
    pub m22: f32,
    pub m23: f32,
    pub m24: f32,

    pub m31: f32,
    pub m32: f32,
    pub m33: f32,
    pub m34: f32,

    pub m41: f32,
    pub m42: f32,
    pub m43: f32,
    pub m44: f32,
}

impl Matrix4x4 {
    pub const fn create_orthographic_off_center(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        z_near_plane: f32,
        z_far_plane: f32,
    ) -> Self {
        Matrix4x4 {
            m11: 2.0 / (right - left),
            m12: 0.0,
            m13: 0.0,
            m14: 0.0,

            m21: 0.0,
            m22: 2.0 / (top - bottom),
            m23: 0.0,
            m24: 0.0,

            m31: 0.0,
            m32: 0.0,
            m33: 1.0 / (z_near_plane - z_far_plane),
            m34: 0.0,

            m41: (left + right) / (left - right),
            m42: (top + bottom) / (bottom - top),
            m43: z_near_plane / (z_near_plane - z_far_plane),
            m44: 1.0,
        }
    }
}
