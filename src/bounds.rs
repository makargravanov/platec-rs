// SPDX-License-Identifier: LGPL-2.1-or-later
// Derived from Mindwerks Plate Tectonics; see THIRD_PARTY_NOTICES.md.
use super::geometry::{FloatPoint, WorldDimension};
use super::rectangle::{BAD_INDEX, Rectangle};

#[derive(Debug, Clone, PartialEq)]
pub struct Bounds {
    dimensions: WorldDimension,
    position: FloatPoint,
    width: u32,
    height: u32,
}

impl Bounds {
    pub fn new(dimensions: WorldDimension, position: FloatPoint, width: u32, height: u32) -> Self {
        assert!(width <= dimensions.width() && height <= dimensions.height());
        Self {
            dimensions,
            position,
            width,
            height,
        }
    }

    pub fn index(&self, x: u32, y: u32) -> u32 {
        assert!(x < self.width && y < self.height);
        y * self.width + x
    }

    pub fn area(&self) -> u32 {
        self.width * self.height
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn left(&self) -> u32 {
        self.position.x as u32
    }
    pub fn top(&self) -> u32 {
        self.position.y as u32
    }
    pub fn right_non_inclusive(&self) -> u32 {
        self.left() + self.width - 1
    }
    pub fn bottom_non_inclusive(&self) -> u32 {
        self.top() + self.height - 1
    }

    pub fn is_in_limits(&self, x: f32, y: f32) -> bool {
        if x < 0.0 || y < 0.0 {
            return false;
        }
        (x as u32) < self.width && (y as u32) < self.height
    }

    pub fn shift(&mut self, dx: f32, dy: f32) {
        self.position.shift(dx, dy, self.dimensions);
    }

    pub fn grow(&mut self, dx: u32, dy: u32) {
        self.width += dx;
        self.height += dy;
        assert!(self.width <= self.dimensions.width());
        assert!(self.height <= self.dimensions.height());
    }

    pub fn get_map_index(&self, x: &mut u32, y: &mut u32) -> u32 {
        self.as_rect().get_map_index(x, y)
    }

    pub fn get_valid_map_index(&self, x: &mut u32, y: &mut u32) -> u32 {
        let index = self.get_map_index(x, y);
        assert_ne!(index, BAD_INDEX);
        index
    }

    fn as_rect(&self) -> Rectangle {
        Rectangle::new(
            self.dimensions,
            self.left(),
            self.left() + self.width,
            self.top(),
            self.top() + self.height,
        )
    }
}
