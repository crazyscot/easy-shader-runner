#![no_std]

use push_constants::shader::*;
use shared::*;
use spirv_std::glam::*;
use spirv_std::spirv;

#[spirv(fragment)]
pub fn main_fs(
    #[spirv(frag_coord)] frag_coord: Vec4,
    #[cfg(not(feature = "emulate_constants"))]
    #[spirv(push_constant)]
    constants: &FragmentConstants,
    #[cfg(feature = "emulate_constants")]
    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)]
    constants: &FragmentConstants,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] cell_grid: &mut [CellState],
    output: &mut Vec4,
) {
    let zoom = constants.zoom;
    let size = constants.size.as_vec2();
    let frag_coord = vec2(frag_coord.x, frag_coord.y - UI_MENU_HEIGHT as f32);
    let cursor = ((constants.cursor - size / 2.0 / zoom) / size).clamp(
        Vec2::splat(-1.0 / zoom + 1.0 / zoom),
        Vec2::splat(1.0 - 1.0 / zoom),
    );
    let i = ((frag_coord - 0.5) / size * DIM.as_vec2() / zoom + cursor * DIM.as_vec2()).as_uvec2();

    if constants.mouse_button_pressed & 1 == 1 {
        if constants.cursor.distance_squared(frag_coord) < 0.5 {
            cell_grid[(i.y * DIM.x + i.x) as usize] = CellState::On;
        }
    }

    let val = cell_grid[(i.y * DIM.x + i.x) as usize];
    let col = match val {
        CellState::Off => Vec3::ZERO,
        CellState::On => Vec3::X,
        CellState::Dying => vec3(0.3, 0.05, 0.0),
        CellState::Spawning => vec3(0.35, 0.0, 0.0),
    };
    *output = col.extend(1.0);
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vert_id: i32,
    #[spirv(position, invariant)] out_pos: &mut Vec4,
) {
    let uv = vec2(((vert_id << 1) & 2) as f32, (vert_id & 2) as f32);
    let pos = 2.0 * uv - Vec2::ONE;
    *out_pos = pos.extend(0.0).extend(1.0);
}

#[spirv(compute(threads(16, 16)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] gid: UVec3,
    #[cfg(not(feature = "emulate_constants"))]
    #[spirv(push_constant)]
    constants: &ComputeConstants,
    #[cfg(feature = "emulate_constants")]
    #[spirv(storage_buffer, descriptor_set = 2, binding = 0)]
    constants: &ComputeConstants,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] cell_grid: &mut [CellState],
) {
    let index = (gid.y * DIM.x + gid.x) as usize;
    let val = cell_grid[index];

    if constants.transition.into() {
        cell_grid[index] = match val {
            CellState::Dying => CellState::Off,
            CellState::Spawning => CellState::On,
            CellState::Off => CellState::Off,
            CellState::On => CellState::On,
        };
        return;
    }

    let mut count = 0;
    for i in -1..=1 {
        for j in -1..=1 {
            if i == 0 && j == 0 {
                continue;
            }
            let y = (gid.y as i32 + i).rem_euclid(DIM.y as i32);
            let x = (gid.x as i32 + j).rem_euclid(DIM.x as i32);

            let val = cell_grid[(y * DIM.x as i32 + x) as usize];
            if matches!(val, CellState::On | CellState::Dying) {
                count += 1
            };
        }
    }

    if matches!(val, CellState::On) && (count < 2 || count > 3) {
        cell_grid[index] = CellState::Dying;
    } else if matches!(val, CellState::Off) && count == 3 {
        cell_grid[index] = CellState::Spawning;
    }
}
