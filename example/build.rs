fn main() {
    #[cfg(any(not(feature = "watch"), target_arch = "wasm32"))]
    builder_launcher::build("shader/shader");
}
