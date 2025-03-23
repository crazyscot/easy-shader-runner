use crate::{
    bind_group_buffer::{BindGroupBufferType, BufferDescriptor, SSBO},
    Options,
};
use egui_winit::winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta},
    keyboard::{Key, NamedKey},
};
use glam::*;
use shared::push_constants::shader::*;
use shared::*;
use web_time::Instant;

mod ui;

pub struct Controller {
    size: UVec2,
    start: Instant,
    fragment_constants: FragmentConstants,
    compute_constants: ComputeConstants,
    cursor: Vec2,
    prev_cursor: Vec2,
    mouse_button_pressed: u32,
    speed: f32,
    distance: f32,
    last_frame: Instant,
    zoom: f32,
    debug: bool,
    cell_grid: grid::Grid<CellState>,
    transition: bool,
    paused: bool,
    translate: Vec2,
}

impl Controller {
    pub fn new(size: PhysicalSize<u32>, options: &Options) -> Self {
        let now = Instant::now();
        let size = uvec2(size.width - UI_SIDEBAR_WIDTH, size.height - UI_MENU_HEIGHT);

        let debug = options.debug;

        let mut cell_grid = grid::Grid::new(DIM);
        let seed = [
            // Initial configuration
            [0, 1, 0],
            [1, 1, 0],
            [0, 1, 1],
        ];
        {
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
            size,
            start: now,
            fragment_constants: Default::default(),
            compute_constants: Default::default(),
            cursor: Vec2::ZERO,
            prev_cursor: Vec2::ZERO,
            mouse_button_pressed: 0,
            speed: 1.0,
            distance: 0.0,
            last_frame: now,
            zoom: 1.0,
            debug,
            cell_grid,
            transition: false,
            paused: options.debug,
            translate: Vec2::ZERO,
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.size = uvec2(size.width - UI_SIDEBAR_WIDTH, size.height - UI_MENU_HEIGHT);
    }

    pub fn mouse_move(&mut self, position: PhysicalPosition<f64>) {
        self.cursor = vec2(position.x as f32, position.y as f32 - UI_MENU_HEIGHT as f32);
    }

    pub fn mouse_scroll(&mut self, delta: MouseScrollDelta) {
        let val = match delta {
            MouseScrollDelta::LineDelta(_, val) => val * 0.1,
            MouseScrollDelta::PixelDelta(p) => (p.y * 0.005) as f32,
        };
        let prev_zoom = self.zoom;
        self.zoom = (self.zoom + self.zoom * val).clamp(1.0, 100.0);
        let size = self.size.as_vec2();
        let pixel_dif = size / prev_zoom - size / self.zoom;
        self.translate += pixel_dif * self.cursor / size / size;
        self.translate = self
            .translate
            .clamp(Vec2::ZERO, Vec2::ONE * (1.0 - 1.0 / self.zoom));
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
        match key.logical_key {
            Key::Character(c) => match c.chars().next().unwrap() {
                'z' => {}
                'x' => {
                    if key.state.is_pressed() {
                        self.distance += 1.0;
                    }
                }
                _ => {}
            },
            Key::Named(NamedKey::Space) => {
                if key.state.is_pressed() {
                    self.paused = !self.paused;
                }
            }
            _ => {}
        }
    }

    pub fn pre_render(&mut self) {
        self.fragment_constants = FragmentConstants {
            size: self.size.into(),
            time: self.start.elapsed().as_secs_f32(),
            mouse_button_pressed: self.mouse_button_pressed,
            cursor: self.cursor,
            prev_cursor: self.prev_cursor,
            zoom: self.zoom,
            debug: self.debug.into(),
            translate: self.translate,
        };
        self.prev_cursor = self.cursor;
    }

    pub fn pre_update(&mut self) {
        self.compute_constants = ComputeConstants {
            size: self.size.into(),
            time: self.start.elapsed().as_secs_f32(),
            zoom: self.zoom,
            transition: self.transition.into(),
        };
        self.transition = !self.transition;
    }

    pub fn fragment_constants(&self) -> &[u8] {
        bytemuck::bytes_of(&self.fragment_constants)
    }

    pub fn compute_constants(&self) -> &[u8] {
        bytemuck::bytes_of(&self.compute_constants)
    }

    pub fn compute_dimensions(&self) -> UVec2 {
        shared::DIM
    }

    pub fn buffers(&self) -> Vec<BufferDescriptor> {
        vec![BufferDescriptor {
            buffer_type: BindGroupBufferType::SSBO(SSBO {
                data: bytemuck::cast_slice(&self.cell_grid.buffer),
                read_only: false,
            }),
            shader_stages: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
        }]
    }

    pub fn iterations(&mut self) -> u32 {
        let speed = if self.paused { 0.0 } else { self.speed };
        let t = self.last_frame.elapsed().as_secs_f32() * 30.0;
        self.last_frame = Instant::now();
        self.distance += speed * t;
        if self.distance >= 1.0 {
            let iterations = self.distance as u32;
            self.distance = self.distance.fract();
            iterations
        } else {
            0
        }
    }
}
