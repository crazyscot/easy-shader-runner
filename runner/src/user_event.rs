use crate::app::Graphics;
use crate::controller::ControllerTrait;

pub enum UserEvent<C: ControllerTrait> {
    #[cfg(feature = "watch")]
    NewModule(std::path::PathBuf),
    #[cfg(not(target_arch = "wasm32"))]
    SetVSync(bool),
    CreateWindow(Graphics<C>),
}
