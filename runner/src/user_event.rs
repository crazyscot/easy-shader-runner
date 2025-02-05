use crate::app::Graphics;
use crate::shader::CompiledShaderModule;

pub enum UserEvent {
    NewModule(CompiledShaderModule),
    SetVSync(bool),
    CreateWindow(Graphics),
}
