// SPDX-License-Identifier: LGPL-2.1-or-later
// Derived from Mindwerks Plate Tectonics; see THIRD_PARTY_NOTICES.md.
use super::geometry::FloatPoint;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mass {
    mass: f32,
    cx: f32,
    cy: f32,
}

impl Mass {
    pub const fn new(mass: f32, cx: f32, cy: f32) -> Self {
        Self { mass, cx, cy }
    }

    pub fn inc_mass(&mut self, delta: f32) {
        self.mass += delta;
        if self.mass < 0.0 {
            self.mass = 0.0;
        }
    }

    pub const fn mass(self) -> f32 {
        self.mass
    }
    pub const fn center(self) -> FloatPoint {
        FloatPoint::new(self.cx, self.cy)
    }
    pub fn is_null(self) -> bool {
        self.mass <= 0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MassBuilder {
    mass: f32,
    cx: f32,
    cy: f32,
}

impl MassBuilder {
    pub const fn new() -> Self {
        Self {
            mass: 0.0,
            cx: 0.0,
            cy: 0.0,
        }
    }

    pub fn from_map(map: &[f32], width: u32, height: u32) -> Self {
        let mut builder = Self::new();
        let mut index = 0_usize;
        for y in 0..height {
            for x in 0..width {
                builder.add_point(x, y, map[index]);
                index += 1;
            }
        }
        builder
    }

    pub fn add_point(&mut self, x: u32, y: u32, crust: f32) {
        assert!(crust >= 0.0);
        self.mass += crust;
        self.cx += x as f32 * crust;
        self.cy += y as f32 * crust;
    }

    pub fn build(self) -> Mass {
        if self.mass <= 0.0 {
            Mass::new(0.0, 0.0, 0.0)
        } else {
            let inv_mass = 1.0 / self.mass;
            Mass::new(self.mass, self.cx * inv_mass, self.cy * inv_mass)
        }
    }
}
