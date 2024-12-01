set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

trangle:
    cargo run --example basic_triangle

shaders:
    cargo run --bin shaders

