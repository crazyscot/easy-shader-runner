use crate::shader::compile_shader;
use egui_winit::winit::event_loop::EventLoop;
use structopt::StructOpt;

mod app;
mod bind_group_buffer;
mod context;
mod controller;
mod fps_counter;
mod render_pass;
mod shader;
mod ui;
mod user_event;

const TITLE: &str = "runner";

#[derive(StructOpt, Clone, Copy)]
#[structopt(name = TITLE)]
pub struct Options {
    /// Starts in debug mode and with speed set to 0
    #[structopt(short, long)]
    debug: bool,
}

pub fn main() {
    let options = Options::from_args();

    env_logger::init();

    let event_loop = EventLoop::with_user_event().build().unwrap();

    // Build the shader before we pop open a window, since it might take a while.
    let shader_path = compile_shader(
        #[cfg(feature = "watch")]
        event_loop.create_proxy(),
    );

    let mut app = app::App::new(event_loop.create_proxy(), shader_path, options);
    event_loop.run_app(&mut app).unwrap()
}
