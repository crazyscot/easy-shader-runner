# Template
A template for an application using fragment and compute shaders with egui

Uses
[rust-gpu](https://github.com/Rust-GPU/rust-gpu),
[wgpu](https://github.com/gfx-rs/wgpu), and
[egui](https://github.com/emilk/egui)

## Try it out
```bash
nix run github:abel465/rust-gpu-template
```

## Development
With shader hot reloading
```bash
git clone https://github.com/abel465/rust-gpu-template.git
cd rust-gpu-template/
nix develop
cargo run --release
```
