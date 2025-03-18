use egui_winit::winit::event_loop::EventLoop;
use structopt::StructOpt;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::wasm_bindgen::{self, prelude::*};

mod app;
mod bind_group_buffer;
mod context;
mod controller;
mod fps_counter;
mod render_pass;
#[cfg(not(target_arch = "wasm32"))]
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

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn main() {
    let options = Options::from_args();

    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Debug).expect("could not initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::with_user_event().build().unwrap();

    // Build the shader before we pop open a window, since it might take a while.
    #[cfg(not(target_arch = "wasm32"))]
    let shader_path = shader::compile_shader(
        #[cfg(feature = "watch")]
        event_loop.create_proxy(),
    );

    let mut app = app::App::new(
        event_loop.create_proxy(),
        #[cfg(not(target_arch = "wasm32"))]
        shader_path,
        options,
    );
    event_loop.run_app(&mut app).unwrap()
}
