use egui_winit::winit::event_loop::EventLoop;
use std::borrow::Cow;
use std::path::PathBuf;
use user_event::CustomEvent;

pub use context::GraphicsContext;
pub use controller::ControllerTrait;
pub use ui::UiState;

pub use egui_wgpu::wgpu;
pub use egui_winit::egui;
pub use egui_winit::winit;

mod app;
mod context;
mod controller;
mod fps_counter;
mod render_pass;
#[cfg(all(
    any(feature = "runtime-compilation", feature = "hot-reload-shader"),
    not(target_arch = "wasm32")
))]
mod shader;
mod ui;
mod user_event;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    EventLoopError(#[from] egui_winit::winit::error::EventLoopError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("Missing CARGO_MANIFEST_DIR")]
    MissingCargoManifest,
    #[error("Shader directory {0} not found")]
    ShaderDirectoryNotFound(PathBuf),
    #[cfg(all(
        any(feature = "runtime-compilation", feature = "hot-reload-shader"),
        not(target_arch = "wasm32")
    ))]
    #[error(transparent)]
    BuildFailed(spirv_builder::SpirvBuilderError),
    #[error("Build failed")]
    BuildFailedQuietly,
}

/// Run with runtime compilation
///
/// `shader_crate_path` is relative to CARGO_MANIFEST_DIR
#[cfg(all(
    any(feature = "runtime-compilation", feature = "hot-reload-shader"),
    not(target_arch = "wasm32")
))]
pub fn run_with_runtime_compilation<C: ControllerTrait + Send>(
    controller: C,
    // Path of shader crate, relative to CARGO_MANIFEST_DIR
    shader_crate_path: impl AsRef<std::path::Path>,
    title: impl Into<String>,
) {
    run_with_runtime_compilation_2(controller, shader_crate_path, title, true).unwrap();
}

/// Run with runtime compilation
///
/// If `relative_to_manifest` is true, `shader_crate_path` is relative to CARGO_MANIFEST_DIR.
/// If not, it is a standard path (may be absolute or relative).
#[cfg(all(
    any(feature = "runtime-compilation", feature = "hot-reload-shader"),
    not(target_arch = "wasm32")
))]
pub fn run_with_runtime_compilation_2<C: ControllerTrait + Send>(
    controller: C,
    // Path of shader crate, relative to CARGO_MANIFEST_DIR
    shader_crate_path: impl AsRef<std::path::Path>,
    title: impl Into<String>,
    relative_to_manifest: bool,
) -> Result<(), Error> {
    setup_logging();
    let event_loop = EventLoop::with_user_event().build()?;
    // Build the shader before we pop open a window, since it might take a while.
    let shader_path = shader::compile_shader(
        #[cfg(feature = "hot-reload-shader")]
        event_loop.create_proxy(),
        shader_crate_path,
        relative_to_manifest,
    )?;
    let shader_bytes = std::fs::read(shader_path)?;
    start(event_loop, controller, shader_bytes, title.into())
}

pub fn run_with_prebuilt_shader<C: ControllerTrait>(
    controller: C,
    shader_bytes: &'static [u8],
    title: impl Into<String>,
) {
    setup_logging();
    let event_loop = EventLoop::with_user_event().build().unwrap();
    start(event_loop, controller, shader_bytes, title.into()).unwrap();
}

pub fn run_with_prebuilt_shader_2<C: ControllerTrait>(
    controller: C,
    shader_bytes: &'static [u8],
    title: impl Into<String>,
) -> Result<(), Error> {
    setup_logging();
    let event_loop = EventLoop::with_user_event().build()?;
    start(event_loop, controller, shader_bytes, title.into())
}

fn start<C: ControllerTrait>(
    event_loop: EventLoop<CustomEvent<C>>,
    controller: C,
    shader_bytes: impl Into<Cow<'static, [u8]>>,
    title: String,
) -> Result<(), Error> {
    let mut app = app::App::new(
        event_loop.create_proxy(),
        shader_bytes.into(),
        controller,
        title,
    );
    Ok(event_loop.run_app(&mut app)?)
}

pub fn setup_logging() {
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
            unsafe {
                std::env::set_var("RUST_LOG", rust_log);
            }
            let _ = env_logger::try_init();
        }
    }
}
