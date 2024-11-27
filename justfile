set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

dev:
    cargo run

build:
    cargo build --release

shaders:
    cargo run --bin shaders

