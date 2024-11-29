use std::process::Command;

const SHADERCROSS: &'static str = "./bin/shadercross";

const SHADERS_SOURCE_DIR: &'static str = "./content/shaders/source";
const SHADERS_COMPILED_DIR: &'static str = "./content/shaders/compiled";

pub fn main() {
    let shader_source_dir = std::fs::read_dir(SHADERS_SOURCE_DIR).unwrap();

    for entry in shader_source_dir {
        let entry = entry.unwrap();

        let in_path = entry.path().display().to_string();

        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();

        for out_format in ["spv", "json"] {
            let out_file_name = file_name.replace("hlsl", out_format);
            let out_path = format!("{SHADERS_COMPILED_DIR}/{out_file_name}");

            Command::new(SHADERCROSS)
                .arg(&in_path)
                .arg("--output")
                .arg(&out_path)
                .output()
                .expect(&format!("failed to compile shader: {in_path}"));
        }
    }
}
