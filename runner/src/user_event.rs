use crate::app::Graphics;
use crate::controller::ControllerTrait;

pub enum CustomEvent<C: ControllerTrait> {
    #[cfg(feature = "watch")]
    NewModule(std::path::PathBuf),
    CreateWindow(Graphics<C>),
    UserEvent(UserEvent),
}

pub enum UserEvent {
    #[cfg(not(target_arch = "wasm32"))]
    SetVSync(bool),
}
