use crate::{
    context::GraphicsContext,
    controller::Controller,
    render_pass::RenderPass,
    ui::{Ui, UiState},
    user_event::UserEvent,
    Options,
};
#[cfg(not(target_arch = "wasm32"))]
use egui_winit::winit::platform::wayland::*;
use egui_winit::winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoopProxy},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};
use std::sync::Arc;

pub struct Graphics {
    rpass: RenderPass,
    ctx: GraphicsContext,
    controller: Controller,
    ui: Ui,
    ui_state: UiState,
    window: Arc<Window>,
}

pub struct Builder {
    event_proxy: EventLoopProxy<UserEvent>,
    #[cfg(not(target_arch = "wasm32"))]
    shader_path: std::path::PathBuf,
    options: Options,
}

pub enum App {
    Builder(Builder),
    Building(#[cfg(target_arch = "wasm32")] Option<PhysicalSize<u32>>),
    Graphics(Graphics),
}

impl App {
    pub fn new(
        event_proxy: EventLoopProxy<UserEvent>,
        #[cfg(not(target_arch = "wasm32"))] shader_path: std::path::PathBuf,
        options: Options,
    ) -> Self {
        Self::Builder(Builder {
            event_proxy,
            #[cfg(not(target_arch = "wasm32"))]
            shader_path,
            options,
        })
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        let Self::Graphics(gfx) = self else {
            #[cfg(target_arch = "wasm32")]
            if let Self::Building(_) = self {
                *self = Self::Building(Some(size));
            }
            return;
        };
        if size.width != 0 && size.height != 0 {
            gfx.ctx.config.width = size.width;
            gfx.ctx.config.height = size.height;
            gfx.ctx.surface.configure(&gfx.ctx.device, &gfx.ctx.config);
            gfx.controller.resize(size);
        }
    }

    pub fn scale_factor_changed(&mut self, scale_factor: f64) {
        let Self::Graphics(gfx) = self else {
            return;
        };
        gfx.controller
            .scale_factor_changed(scale_factor, gfx.window.inner_size());
    }

    pub fn keyboard_input(&mut self, event: KeyEvent) {
        let Self::Graphics(gfx) = self else {
            return;
        };
        gfx.controller.keyboard_input(event);
    }

    pub fn mouse_input(&mut self, state: ElementState, button: MouseButton) {
        let Self::Graphics(gfx) = self else {
            return;
        };
        gfx.controller.mouse_input(state, button);
    }

    pub fn mouse_move(&mut self, position: PhysicalPosition<f64>) {
        let Self::Graphics(gfx) = self else {
            return;
        };
        gfx.controller.mouse_move(position);
    }

    pub fn mouse_scroll(&mut self, delta: MouseScrollDelta) {
        let Self::Graphics(gfx) = self else {
            return;
        };
        gfx.controller.mouse_scroll(delta);
    }

    pub fn update(&mut self) {
        let Self::Graphics(gfx) = self else {
            return;
        };
        let start = web_time::Instant::now();
        for _ in 0..gfx.controller.iterations() {
            gfx.controller.pre_update();
            gfx.rpass.compute(&gfx.ctx, &gfx.controller);
            if start.elapsed().as_secs_f32() > 1.0 / gfx.ui_state.fps as f32 {
                break;
            }
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let Self::Graphics(gfx) = self else {
            return Ok(());
        };
        gfx.window.request_redraw();
        gfx.controller.pre_render();
        gfx.rpass.render(
            &gfx.ctx,
            &gfx.window,
            &mut gfx.ui,
            &mut gfx.ui_state,
            &mut gfx.controller,
        )
    }

    pub fn update_and_render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.update();
        self.render()
    }

    pub fn ui_consumes_event(&mut self, event: &WindowEvent) -> bool {
        let Self::Graphics(gfx) = self else {
            return false;
        };
        gfx.ui.consumes_event(&gfx.window, event)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn new_module(&mut self, shader_path: &std::path::Path) {
        let Self::Graphics(gfx) = self else {
            return;
        };
        gfx.rpass.new_module(&gfx.ctx, shader_path);
        gfx.window.request_redraw();
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_vsync(&mut self, enable: bool) {
        let Self::Graphics(gfx) = self else {
            return;
        };
        gfx.ctx.set_vsync(enable);
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Self::Builder(builder) = std::mem::replace(
            self,
            Self::Building(
                #[cfg(target_arch = "wasm32")]
                None,
            ),
        ) {
            let window_attributes = Window::default_attributes().with_title(crate::TITLE);
            let window_attributes = {
                cfg_if::cfg_if! {
                    if #[cfg(target_arch = "wasm32")] {
                        use egui_winit::winit::platform::web::WindowAttributesExtWebSys;
                        window_attributes
                            .with_prevent_default(false)
                            .with_append(true)
                    } else {
                        window_attributes.with_name(crate::TITLE, "")
                    }
                }
            };
            let window = event_loop.create_window(window_attributes).unwrap();

            cfg_if::cfg_if! {
                if #[cfg(target_arch = "wasm32")] {
                    let size = web_sys::window()
                        .map(|win| {
                            let width = win.inner_width().unwrap().unchecked_into_f64() as u32;
                            let height = win.inner_height().unwrap().unchecked_into_f64() as u32;
                            PhysicalSize { width, height }
                        })
                        .expect("couldn't get window size");
                    wasm_bindgen_futures::spawn_local(create_graphics(builder, size, window));
                } else {
                    futures::executor::block_on(create_graphics(builder, window.inner_size(), window));
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.ui_consumes_event(&event) {
            return;
        }
        match event {
            WindowEvent::RedrawRequested => {
                if let Err(wgpu::SurfaceError::OutOfMemory) = self.update_and_render() {
                    event_loop.exit()
                }
            }
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        logical_key: Key::Named(NamedKey::Escape),
                        ..
                    },
                ..
            } => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => self.keyboard_input(event),
            WindowEvent::Resized(size) => self.resize(size),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale_factor_changed(scale_factor)
            }
            WindowEvent::MouseInput { state, button, .. } => self.mouse_input(state, button),
            WindowEvent::MouseWheel { delta, .. } => self.mouse_scroll(delta),
            WindowEvent::CursorMoved { position, .. } => self.mouse_move(position),
            _ => {}
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::CreateWindow(gfx) => {
                gfx.window.request_redraw();
                cfg_if::cfg_if! {
                    if #[cfg(target_arch = "wasm32")] {
                        if let Self::Building(Some(size)) = std::mem::replace(self, Self::Graphics(gfx)) {
                            self.resize(size);
                        };
                    } else {
                        *self = Self::Graphics(gfx);
                    }
                };
            }
            #[cfg(not(target_arch = "wasm32"))]
            UserEvent::NewModule(shader_path) => self.new_module(&shader_path),
            #[cfg(not(target_arch = "wasm32"))]
            UserEvent::SetVSync(enable) => self.set_vsync(enable),
        }
    }
}

async fn create_graphics(builder: Builder, initial_size: PhysicalSize<u32>, window: Window) {
    let window = Arc::new(window);
    let ctx = GraphicsContext::new(window.clone(), initial_size).await;

    let ui = Ui::new(window.clone(), builder.event_proxy.clone());

    let ui_state = UiState::new();

    let controller = Controller::new(initial_size, window.scale_factor(), &builder.options);

    let rpass = RenderPass::new(
        &ctx,
        #[cfg(not(target_arch = "wasm32"))]
        &builder.shader_path,
        &controller.buffers(),
    );

    let gfx = Graphics {
        rpass,
        ctx,
        controller,
        ui,
        ui_state,
        window,
    };

    builder
        .event_proxy
        .send_event(UserEvent::CreateWindow(gfx))
        .ok();
}
