use crate::app::Graphics;

pub enum UserEvent {
    NewModule(std::path::PathBuf),
    SetVSync(bool),
    CreateWindow(Graphics),
}
