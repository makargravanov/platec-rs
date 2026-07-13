// SPDX-License-Identifier: LGPL-2.1-or-later
// Derived from Mindwerks Plate Tectonics; see THIRD_PARTY_NOTICES.md.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimpleRandom {
    cong: u32,
}

impl SimpleRandom {
    pub const fn new(seed: u32) -> Self {
        Self { cong: seed }
    }

    #[inline]
    pub fn next_u32(&mut self) -> u32 {
        self.cong = 69_069_u32.wrapping_mul(self.cong).wrapping_add(12_345);
        self.cong
    }

    #[inline]
    pub fn next_f64(&mut self) -> f64 {
        self.next_u32() as f64 / Self::maximum() as f64
    }

    #[inline]
    pub fn next_f32(&mut self) -> f32 {
        self.next_u32() as f32 / Self::maximum() as f32
    }

    #[inline]
    pub fn next_f32_signed(&mut self) -> f32 {
        self.next_f32() - 0.5
    }

    #[inline]
    pub const fn maximum() -> u32 {
        u32::MAX
    }
}
