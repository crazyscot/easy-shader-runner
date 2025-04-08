# Template
A template for a [wgpu](https://github.com/gfx-rs/wgpu) application with [egui](https://github.com/emilk/egui) and [rust-gpu](https://github.com/Rust-GPU/rust-gpu) shaders

## Try it out
```bash
nix run github:abel465/rust-gpu-template
```

## Development
#### Set up environment
```bash
git clone https://github.com/abel465/rust-gpu-template.git
cd rust-gpu-template/
nix develop
```

### Native
```bash
cargo run --bin cellular_automata
```

### Wasm
```bash
cd wasm-app
npm install
wasm-pack build ../examples/cellular_automata/ --out-dir ../../wasm-app/pkg
npm run dev
```
