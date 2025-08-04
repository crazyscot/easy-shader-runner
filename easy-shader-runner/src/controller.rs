use egui_winit::winit::event::{ElementState, KeyEvent, MouseButton, TouchPhase};
use glam::*;

pub trait ControllerTrait: 'static {
    fn resize(&mut self, _size: UVec2);

    fn touch(&mut self, _id: u64, _phase: TouchPhase, _position: DVec2) {}

    fn mouse_move(&mut self, _position: DVec2) {}

    fn mouse_scroll(&mut self, _delta: DVec2) {}

    fn mouse_input(&mut self, _state: ElementState, _button: MouseButton) {}

    fn keyboard_input(&mut self, _key: KeyEvent) {}

    fn prepare_render(&mut self, offset: Vec2) -> impl bytemuck::NoUninit;

    /// Run the compute shader after rendering
    #[cfg(feature = "compute")]
    fn update<
        F: Fn(
            UVec3, // dimensions
            UVec3, // threads (same as declared in compute shader)
            &[u8], // push_constants
        ),
    >(
        &mut self,
        _compute: F,
        _allowed_duration: f32,
    ) {
    }

    fn describe_bind_groups(
        &mut self,
        _queue: &wgpu::Queue,
        _device: &wgpu::Device,
    ) -> (Vec<wgpu::BindGroupLayout>, Vec<wgpu::BindGroup>) {
        (vec![], vec![])
    }

    fn describe_wgpu_features_and_limits(
        &self,
        _supported_features: wgpu::Features,
        _supported_limits: wgpu::Limits,
    ) -> (wgpu::Features, wgpu::Limits) {
        (wgpu::Features::default(), wgpu::Limits::default())
    }

    fn ui(
        &mut self,
        _ctx: &egui::Context,
        _ui_state: &mut crate::ui::UiState,
        _graphics_context: &crate::GraphicsContext,
    ) {
    }
}
