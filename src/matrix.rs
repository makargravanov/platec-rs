// SPDX-License-Identifier: LGPL-2.1-or-later
// Derived from Mindwerks Plate Tectonics; see THIRD_PARTY_NOTICES.md.
#[derive(Debug, Clone, PartialEq)]
pub struct Matrix<T> {
    width: u32,
    height: u32,
    values: Vec<T>,
}

impl<T: Clone> Matrix<T> {
    pub fn new(width: u32, height: u32, value: T) -> Self {
        assert!(
            width != 0 && height != 0,
            "matrix dimensions must be non-zero"
        );
        Self {
            width,
            height,
            values: vec![value; (width * height) as usize],
        }
    }
}

impl<T> Matrix<T> {
    #[inline]
    pub fn from_vec(width: u32, height: u32, values: Vec<T>) -> Self {
        assert!(
            width != 0 && height != 0,
            "matrix dimensions must be non-zero"
        );
        assert_eq!(values.len(), (width * height) as usize);
        Self {
            width,
            height,
            values,
        }
    }

    #[inline]
    pub fn line_index(&self, y: u32) -> usize {
        assert!(y < self.height);
        (y * self.width) as usize
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        &self.values
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.values
    }
}

impl<T> core::ops::Index<usize> for Matrix<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.values[index]
    }
}

impl<T> core::ops::IndexMut<usize> for Matrix<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.values[index]
    }
}

pub type HeightMap = Matrix<f32>;
pub type AgeMap = Matrix<u32>;
pub type IndexMap = Matrix<u32>;
