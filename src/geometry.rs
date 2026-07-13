// SPDX-License-Identifier: LGPL-2.1-or-later
// Derived from Mindwerks Plate Tectonics; see THIRD_PARTY_NOTICES.md.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldDimension {
    width: u32,
    height: u32,
}

impl WorldDimension {
    pub fn new(width: u32, height: u32) -> Self {
        assert!(width != 0 && height != 0);
        Self { width, height }
    }

    #[inline]
    pub const fn width(self) -> u32 {
        self.width
    }

    #[inline]
    pub const fn height(self) -> u32 {
        self.height
    }

    #[inline]
    pub const fn area(self) -> u32 {
        self.width * self.height
    }

    #[inline]
    pub fn x_mod(self, x: u32) -> u32 {
        x.wrapping_add(self.width) % self.width
    }

    #[inline]
    pub fn y_mod(self, y: u32) -> u32 {
        y.wrapping_add(self.height) % self.height
    }

    #[inline]
    pub fn index_of(self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }

    #[inline]
    pub fn line_index(self, y: u32) -> usize {
        assert!(y < self.height);
        self.index_of(0, y)
    }

    #[inline]
    pub fn y_from_index(self, index: u32) -> u32 {
        index / self.width
    }

    #[inline]
    pub fn x_from_index(self, index: u32) -> u32 {
        index - self.y_from_index(index) * self.width
    }

    #[inline]
    pub fn normalized_index_of(self, x: u32, y: u32) -> usize {
        self.index_of(self.x_mod(x), self.y_mod(y))
    }

    #[inline]
    pub fn x_cap(self, x: u32) -> u32 {
        if x < self.width { x } else { self.width - 1 }
    }

    #[inline]
    pub fn y_cap(self, y: u32) -> u32 {
        if y < self.height { y } else { self.height - 1 }
    }

    #[inline]
    pub fn normalize(self, x: &mut u32, y: &mut u32) {
        *x %= self.width;
        *y %= self.height;
    }

    #[inline]
    pub fn contains_float(self, point: FloatPoint) -> bool {
        point.x >= 0.0
            && point.x < self.width as f32
            && point.y >= 0.0
            && point.y < self.height as f32
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatPoint {
    pub x: f32,
    pub y: f32,
}

impl FloatPoint {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn shift(&mut self, dx: f32, dy: f32, dimensions: WorldDimension) {
        self.x += dx;
        if self.x <= 0.0 {
            self.x += dimensions.width() as f32;
        }
        if self.x >= dimensions.width() as f32 {
            self.x -= dimensions.width() as f32;
        }

        self.y += dy;
        if self.y <= 0.0 {
            self.y += dimensions.height() as f32;
        }
        if self.y >= dimensions.height() as f32 {
            self.y -= dimensions.height() as f32;
        }

        assert!(dimensions.contains_float(*self));
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatVector {
    pub x: f32,
    pub y: f32,
}

impl FloatVector {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }
}

impl core::ops::Sub for FloatVector {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl core::ops::Mul<f32> for FloatVector {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntPoint {
    pub x: i32,
    pub y: i32,
}

impl IntPoint {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl From<FloatPoint> for IntPoint {
    fn from(value: FloatPoint) -> Self {
        Self::new(value.x as i32, value.y as i32)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntVector {
    pub x: i32,
    pub y: i32,
}

impl IntVector {
    pub fn length(self) -> f32 {
        ((self.x * self.x + self.y * self.y) as f32).sqrt()
    }
}

impl core::ops::Sub for IntPoint {
    type Output = IntVector;

    fn sub(self, rhs: Self) -> Self::Output {
        IntVector {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
