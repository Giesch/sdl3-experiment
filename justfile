set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

[linux]
dev:
    __NV_PRIME_RENDER_OFFLOAD=1 \
    cargo run

build:
    cargo build --release
