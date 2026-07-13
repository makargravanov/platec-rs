// SPDX-License-Identifier: LGPL-2.1-or-later
// Derived from Mindwerks Plate Tectonics; see THIRD_PARTY_NOTICES.md.
use super::geometry::WorldDimension;

pub fn calculate_crust(
    x: u32,
    y: u32,
    index: usize,
    dimensions: WorldDimension,
    map: &[f32],
    width: u32,
    height: u32,
) -> ([f32; 4], [usize; 4]) {
    let world_width = dimensions.width();
    let world_height = dimensions.height();
    let width_bit = width == world_width;
    let height_bit = height == world_height;
    let w_valid = x > 0 || width_bit;
    let e_valid = x < width - 1 || width_bit;
    let n_valid = y > 0 || height_bit;
    let s_valid = y < height - 1 || height_bit;

    let x_mod = x % world_width;
    let y_mod = y % world_height;
    let x_minus = if x_mod == 0 {
        world_width - 1
    } else {
        x_mod - 1
    };
    let x_plus = if x_mod + 1 == world_width {
        0
    } else {
        x_mod + 1
    };
    let y_minus = if y_mod == 0 {
        world_height - 1
    } else {
        y_mod - 1
    };
    let y_plus = if y_mod + 1 == world_height {
        0
    } else {
        y_mod + 1
    };

    let w = (y * width + if w_valid { x_minus } else { 0 }) as usize;
    let e = (y * width + if e_valid { x_plus } else { 0 }) as usize;
    let n = ((if n_valid { y_minus } else { 0 }) * width + x) as usize;
    let s = ((if s_valid { y_plus } else { 0 }) * width + x) as usize;
    let current = map[index];

    let crust = [
        if w_valid && map[w] < current {
            map[w]
        } else {
            0.0
        },
        if e_valid && map[e] < current {
            map[e]
        } else {
            0.0
        },
        if n_valid && map[n] < current {
            map[n]
        } else {
            0.0
        },
        if s_valid && map[s] < current {
            map[s]
        } else {
            0.0
        },
    ];

    (crust, [w, e, n, s])
}
