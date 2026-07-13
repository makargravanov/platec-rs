// SPDX-License-Identifier: LGPL-2.1-or-later
// Derived from Mindwerks Plate Tectonics; see THIRD_PARTY_NOTICES.md.
use super::rectangle::Rectangle;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentData {
    rectangle: Rectangle,
    area: u32,
    coll_count: u32,
}

impl SegmentData {
    pub const fn new(rectangle: Rectangle, area: u32) -> Self {
        Self {
            rectangle,
            area,
            coll_count: 0,
        }
    }

    pub fn enlarge_to_contain(&mut self, x: u32, y: u32) {
        self.rectangle.enlarge_to_contain(x, y);
    }
    pub fn shift(&mut self, dx: u32, dy: u32) {
        self.rectangle.shift(dx, dy);
    }
    pub fn inc_coll_count(&mut self) {
        self.coll_count += 1;
    }
    pub fn inc_area(&mut self) {
        self.area += 1;
    }
    pub fn inc_area_by(&mut self, amount: u32) {
        self.area += amount;
    }
    pub fn mark_non_existent(&mut self) {
        self.area = 0;
    }
    pub const fn left(&self) -> u32 {
        self.rectangle.left()
    }
    pub const fn right(&self) -> u32 {
        self.rectangle.right()
    }
    pub const fn top(&self) -> u32 {
        self.rectangle.top()
    }
    pub const fn bottom(&self) -> u32 {
        self.rectangle.bottom()
    }
    pub const fn area(&self) -> u32 {
        self.area
    }
    pub const fn coll_count(&self) -> u32 {
        self.coll_count
    }
    pub fn is_empty(&self) -> bool {
        self.area == 0
    }
    pub fn set_left(&mut self, value: u32) {
        self.rectangle.set_left(value);
    }
    pub fn set_right(&mut self, value: u32) {
        self.rectangle.set_right(value);
    }
    pub fn set_top(&mut self, value: u32) {
        self.rectangle.set_top(value);
    }
    pub fn set_bottom(&mut self, value: u32) {
        self.rectangle.set_bottom(value);
    }
}
