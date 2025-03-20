#![cfg_attr(target_arch = "spirv", no_std)]

pub mod push_constants;

use glam::*;

pub const UI_MENU_HEIGHT: u32 = 22;
pub const UI_SIDEBAR_WIDTH: u32 = 164;
pub const DIM: UVec2 = UVec2::splat(192);

#[derive(Clone, Copy, bytemuck::NoUninit)]
#[repr(u32)]
pub enum CellState {
    Off,
    On,
    Dying,
    Spawning,
}
