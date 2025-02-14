use super::*;
use bytemuck::NoUninit;

#[derive(Copy, Clone, Debug, Default, NoUninit)]
#[repr(C)]
pub struct FragmentConstants {
    pub size: Size,
    pub cursor: Vec2,
    pub prev_cursor: Vec2,
    pub time: f32,
    pub mouse_button_pressed: u32,
    pub zoom: f32,
    pub debug: Bool,
}

#[derive(Copy, Clone, Debug, Default, NoUninit)]
#[repr(C)]
pub struct ComputeConstants {
    pub size: Size,
    pub time: f32,
    pub zoom: f32,
}
