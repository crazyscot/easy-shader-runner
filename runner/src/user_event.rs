use crate::app::Graphics;

pub enum UserEvent {
    #[cfg(not(target_arch = "wasm32"))]
    NewModule(std::path::PathBuf),
    #[cfg(not(target_arch = "wasm32"))]
    SetVSync(bool),
    CreateWindow(Graphics),
}
