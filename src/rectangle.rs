// SPDX-License-Identifier: LGPL-2.1-or-later
// Derived from Mindwerks Plate Tectonics; see THIRD_PARTY_NOTICES.md.
use super::geometry::WorldDimension;

pub const BAD_INDEX: u32 = u32::MAX;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rectangle {
    dimensions: WorldDimension,
    left: u32,
    right: u32,
    top: u32,
    bottom: u32,
}

impl Rectangle {
    pub const fn new(
        dimensions: WorldDimension,
        left: u32,
        right: u32,
        top: u32,
        bottom: u32,
    ) -> Self {
        Self {
            dimensions,
            left,
            right,
            top,
            bottom,
        }
    }

    pub fn get_map_index(&self, px: &mut u32, py: &mut u32) -> u32 {
        let world_width = self.dimensions.width();
        let world_height = self.dimensions.height();
        let mut x = *px % world_width;
        let mut y = *py % world_height;
        let ilft = self.left;
        let itop = self.top;
        let irgt = self.right + if self.right < ilft { world_width } else { 0 };
        let ibtm = self.bottom + if self.bottom < itop { world_height } else { 0 };
        let width = irgt - ilft;

        let x_plus_width = x + world_width;
        let x_ok = (x >= ilft && x < irgt) || (x_plus_width >= ilft && x_plus_width < irgt);
        let y_plus_height = y + world_height;
        let y_ok = (y >= itop && y < ibtm) || (y_plus_height >= itop && y_plus_height < ibtm);

        if x < ilft {
            x += world_width;
        }
        if y < itop {
            y += world_height;
        }
        x -= ilft;
        y -= itop;

        if x_ok && y_ok {
            *px = x;
            *py = y;
            y * width + x
        } else {
            BAD_INDEX
        }
    }

    pub fn enlarge_to_contain(&mut self, x: u32, y: u32) {
        if y < self.top {
            self.top = y;
        } else if y > self.bottom {
            self.bottom = y;
        }
        if x < self.left {
            self.left = x;
        } else if x > self.right {
            self.right = x;
        }
    }

    pub fn shift(&mut self, dx: u32, dy: u32) {
        self.left += dx;
        self.right += dx;
        self.top += dy;
        self.bottom += dy;
    }

    pub const fn left(self) -> u32 {
        self.left
    }
    pub const fn right(self) -> u32 {
        self.right
    }
    pub const fn top(self) -> u32 {
        self.top
    }
    pub const fn bottom(self) -> u32 {
        self.bottom
    }
    pub fn set_left(&mut self, value: u32) {
        self.left = value;
    }
    pub fn set_right(&mut self, value: u32) {
        self.right = value;
    }
    pub fn set_top(&mut self, value: u32) {
        self.top = value;
    }
    pub fn set_bottom(&mut self, value: u32) {
        self.bottom = value;
    }
}
