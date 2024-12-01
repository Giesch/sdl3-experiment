use std::ffi::{CStr, CString};
use std::ptr::null_mut;

use sdl3_sys::everything::*;

use serde::Deserialize;

pub unsafe fn load_shader(
    device: *mut SDL_GPUDevice,
    file_name: &'static str,
) -> *mut SDL_GPUShader {
    const COMPILED_SHADERS_DIR: &'static str = "./content/shaders/compiled";

    let full_path = format!("{COMPILED_SHADERS_DIR}/{file_name}.spv");
    let full_path = CString::new(full_path).unwrap();
    let mut code_size = 0;
    let loaded_code = SDL_LoadFile(full_path.as_ptr(), &mut code_size);
    if loaded_code.is_null() {
        dbg_sdl_error(&format!("failed to load shader: {file_name}"));
        return null_mut();
    }

    let json_path = format!("{COMPILED_SHADERS_DIR}/{file_name}.json");
    let Ok(json) = std::fs::read_to_string(&json_path) else {
        println!("failed to find shader json: {json_path}");
        return null_mut();
    };
    let meta = match serde_json::from_str::<ShaderMeta>(&json) {
        Ok(m) => m,
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
        stage,
        code: loaded_code as *const u8,
        code_size,
        entrypoint: c"main".as_ptr(),
        format: SDL_GPU_SHADERFORMAT_SPIRV,
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
