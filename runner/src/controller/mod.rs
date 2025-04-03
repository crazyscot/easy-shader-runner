use crate::{bind_group_buffer::BufferDescriptor, Options};
use egui_winit::winit::{
    event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta},
    keyboard::{Key, NamedKey},
};
use glam::*;
use shared::push_constants::shader::*;
use shared::*;
use simulation_runner::SimulationRunner;
use web_time::Instant;

mod simulation_runner;
mod ui;

struct Camera {
    zoom: f32,
    translate: Vec2,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            translate: Default::default(),
        }
    }
}

pub struct Controller {
    size: UVec2,
    start: Instant,
    cursor: Vec2,
    prev_cursor: Vec2,
    mouse_button_pressed: u32,
    camera: Camera,
    debug: bool,
    cell_grid: grid::Grid<CellState>,
    transition: bool,
    simulation_runner: SimulationRunner,
}

impl Controller {
    pub fn new(options: &Options) -> Self {
        let now = Instant::now();

        let mut cell_grid = grid::Grid::new(DIM);
        {
            let seed = [
                // Initial configuration
                [0, 1, 0],
                [1, 1, 0],
                [0, 1, 1],
            ];
            let p = DIM / 2;
            for (i, row) in seed.into_iter().enumerate() {
                for (j, val) in row.into_iter().enumerate() {
                    if val != 0 {
                        cell_grid.set(p + uvec2(i as u32, j as u32), CellState::On);
                    }
                }
            }
        }

        Self {
            size: UVec2::ZERO,
            start: now,
            cursor: Vec2::ZERO,
            prev_cursor: Vec2::ZERO,
            mouse_button_pressed: 0,
            camera: Default::default(),
            debug: options.debug,
            cell_grid,
            transition: false,
            simulation_runner: SimulationRunner::new(now, options.debug),
        }
    }

    pub fn resize(&mut self, size: UVec2) {
        self.size = size;
    }

    pub fn mouse_move(&mut self, position: Vec2) {
        self.cursor = position;
    }

    pub fn mouse_scroll(&mut self, delta: MouseScrollDelta) {
        let val = match delta {
            MouseScrollDelta::LineDelta(_, val) => val * 0.1,
            MouseScrollDelta::PixelDelta(p) => (p.y * 0.005) as f32,
        };
        let prev_zoom = self.camera.zoom;
        self.camera.zoom = (prev_zoom * (1.0 + val)).clamp(1.0, 100.0);
        let dif = 1.0 / prev_zoom - 1.0 / self.camera.zoom;
        self.camera.translate += dif * self.cursor / self.size.as_vec2();
        self.camera.translate = self
            .camera
            .translate
            .clamp(Vec2::ZERO, Vec2::splat(1.0 - 1.0 / self.camera.zoom));
    }

    pub fn mouse_input(&mut self, state: ElementState, button: MouseButton) {
        let mask = 1
            << match button {
                MouseButton::Left => 0,
                MouseButton::Middle => 1,
                MouseButton::Right => 2,
                MouseButton::Back => 3,
                MouseButton::Forward => 4,
                MouseButton::Other(i) => 5 + (i as usize),
            };
        match state {
            ElementState::Pressed => self.mouse_button_pressed |= mask,
            ElementState::Released => self.mouse_button_pressed &= !mask,
        }
    }

    pub fn keyboard_input(&mut self, key: KeyEvent) {
        if !key.state.is_pressed() {
            return;
        }
        match key.logical_key {
            Key::Character(c) => match c.chars().next().unwrap() {
                'z' => {}
                'x' => self.simulation_runner.add_iteration(),
                _ => {}
            },
            Key::Named(NamedKey::Space) => {
                self.simulation_runner.paused = !self.simulation_runner.paused;
            }
            _ => {}
        }
    }

    pub fn prepare_render(&mut self, offset: Vec2) -> FragmentConstants {
        let fragment_constants = FragmentConstants {
            size: self.size.into(),
            translate: offset,
            time: self.start.elapsed().as_secs_f32(),
            mouse_button_pressed: self.mouse_button_pressed,
            cursor: self.cursor,
            prev_cursor: self.prev_cursor,
            camera_translate: self.camera.translate,
            camera_zoom: self.camera.zoom,
            debug: self.debug.into(),
        };
        self.prev_cursor = self.cursor;
        fragment_constants
    }

    pub fn update<F: Fn(UVec2, &ComputeConstants)>(&mut self, compute: F, allowed_duration: f32) {
        let start = web_time::Instant::now();
        for _ in 0..self.simulation_runner.iterations() {
            compute(
                shared::DIM,
                &ComputeConstants {
                    size: self.size.into(),
                    time: self.start.elapsed().as_secs_f32(),
                    zoom: self.camera.zoom,
                    transition: self.transition.into(),
                },
            );
            self.transition = !self.transition;
            if start.elapsed().as_secs_f32() > allowed_duration {
                break;
            }
        }
    }

    pub fn buffers(&self) -> Vec<BufferDescriptor> {
        vec![BufferDescriptor {
            data: bytemuck::cast_slice(&self.cell_grid.buffer),
            read_only: false,
            shader_stages: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
        }]
    }
}
