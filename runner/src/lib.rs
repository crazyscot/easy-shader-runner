pub use bind_group_buffer::BufferDescriptor;
pub use controller::ControllerTrait;
use egui_winit::winit::event_loop::EventLoop;
pub use ui::UiState;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::wasm_bindgen::{self, prelude::*};

pub use egui_wgpu::wgpu;
pub use egui_winit::egui;
pub use egui_winit::winit;

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

pub fn start<
    #[cfg(feature = "watch")] C: ControllerTrait + Send,
    #[cfg(not(feature = "watch"))] C: ControllerTrait,
>(
    controller: C,
    shader_crate_path: impl AsRef<std::path::Path>,
) {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            let _ = console_log::init();
        } else {
            let mut rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_owned());
            for loud_crate in ["naga", "wgpu_core", "wgpu_hal"] {
                if !rust_log.contains(&format!("{loud_crate}=")) {
                    rust_log += &format!(",{loud_crate}=warn");
                }
            }
            std::env::set_var("RUST_LOG", rust_log);
            let _ = env_logger::try_init();
        }
    }

    let event_loop = EventLoop::with_user_event().build().unwrap();

    // Build the shader before we pop open a window, since it might take a while.
    #[cfg(not(target_arch = "wasm32"))]
    let shader_path = shader::compile_shader(
        #[cfg(feature = "watch")]
        event_loop.create_proxy(),
        shader_crate_path,
    );

    let mut app = app::App::new(
        event_loop.create_proxy(),
        #[cfg(not(target_arch = "wasm32"))]
        shader_path,
        controller,
    );
    event_loop.run_app(&mut app).unwrap()
}
