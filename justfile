set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# run the basic triangle example
triangle:
    cargo run --example basic_triangle

# run the sprite batch example
sprites:
    cargo run --example pull_sprite_batch

# compile all shaders
[linux]
shaders:
    cargo run --bin shaders

