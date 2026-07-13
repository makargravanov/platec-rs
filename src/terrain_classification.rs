// SPDX-License-Identifier: LGPL-2.1-or-later
// Derived from Mindwerks Plate Tectonics; see THIRD_PARTY_NOTICES.md.
use core::fmt;
use std::collections::VecDeque;

const RELIEF_WINDOW_RADIUS: i32 = 2;
const HEIGHT_WEIGHT: f32 = 0.3;
const HILL_PERCENTILE: f32 = 0.80;
const MOUNTAIN_PERCENTILE: f32 = 0.96;
const PLATEAU_RELIEF_THRESHOLD: f32 = 0.15;

#[derive(Debug, Clone, Copy)]
pub struct TerrainClassificationInput<'a> {
    pub width: u32,
    pub height: u32,
    pub heightmap: &'a [f32],
    pub ocean_level: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReliefKind {
    Ocean,
    Valley,
    Plain,
    Hill,
    Mountain,
    HighMountain,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TerrainClassification {
    width: u32,
    height: u32,
    relief_kind: Vec<ReliefKind>,
    relief_score: Vec<f32>,
    local_relief: Vec<f32>,
    hill_threshold: f32,
    mountain_threshold: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerrainClassificationError {
    message: String,
}

impl fmt::Display for TerrainClassificationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for TerrainClassificationError {}

impl TerrainClassification {
    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    pub fn relief_kind(&self) -> &[ReliefKind] {
        &self.relief_kind
    }

    pub fn relief_score(&self) -> &[f32] {
        &self.relief_score
    }

    pub fn local_relief(&self) -> &[f32] {
        &self.local_relief
    }

    pub const fn hill_threshold(&self) -> f32 {
        self.hill_threshold
    }

    pub const fn mountain_threshold(&self) -> f32 {
        self.mountain_threshold
    }
}

pub fn classify_terrain(
    input: TerrainClassificationInput<'_>,
) -> Result<TerrainClassification, TerrainClassificationError> {
    let area = checked_area(input.width, input.height)?;
    validate_len("heightmap", input.heightmap.len(), area)?;

    let mut relief_kind = fill_ocean_valley_plain(
        input.heightmap,
        input.width,
        input.height,
        input.ocean_level,
    );
    let local_relief = compute_local_relief(
        input.heightmap,
        &relief_kind,
        input.width,
        input.height,
        RELIEF_WINDOW_RADIUS,
    );
    let mut normalized_height = input.heightmap.to_vec();
    let mut normalized_relief = local_relief.clone();
    normalize_map(&mut normalized_height);
    normalize_map(&mut normalized_relief);

    let mut relief_score = vec![0.0; area];
    let mut land_scores = Vec::with_capacity(area);
    for index in 0..area {
        if relief_kind[index] == ReliefKind::Plain {
            relief_score[index] = HEIGHT_WEIGHT * normalized_height[index]
                + (1.0 - HEIGHT_WEIGHT) * normalized_relief[index];
            land_scores.push(relief_score[index]);
        }
    }

    let hill_threshold = percentile(&mut land_scores.clone(), HILL_PERCENTILE);
    let mountain_threshold = percentile(&mut land_scores, MOUNTAIN_PERCENTILE);

    for index in 0..area {
        if relief_kind[index] != ReliefKind::Plain {
            continue;
        }

        if relief_score[index] >= mountain_threshold {
            if normalized_relief[index] >= PLATEAU_RELIEF_THRESHOLD {
                relief_kind[index] = ReliefKind::Mountain;
            }
        } else if relief_score[index] >= hill_threshold {
            relief_kind[index] = ReliefKind::Hill;
        }
    }

    Ok(TerrainClassification {
        width: input.width,
        height: input.height,
        relief_kind,
        relief_score,
        local_relief,
        hill_threshold,
        mountain_threshold,
    })
}

fn checked_area(width: u32, height: u32) -> Result<usize, TerrainClassificationError> {
    if width == 0 || height == 0 {
        return Err(TerrainClassificationError {
            message: "terrain dimensions must be non-zero".to_owned(),
        });
    }
    width
        .checked_mul(height)
        .map(|area| area as usize)
        .ok_or_else(|| TerrainClassificationError {
            message: "terrain area overflows u32".to_owned(),
        })
}

fn validate_len(
    name: &str,
    actual: usize,
    expected: usize,
) -> Result<(), TerrainClassificationError> {
    if actual == expected {
        Ok(())
    } else {
        Err(TerrainClassificationError {
            message: format!("{name} length {actual} does not match map area {expected}"),
        })
    }
}

fn fill_ocean_valley_plain(
    heights: &[f32],
    width: u32,
    height: u32,
    ocean_level: f32,
) -> Vec<ReliefKind> {
    let area = (width * height) as usize;
    let mut relief_kind = vec![ReliefKind::Plain; area];
    let mut queue = VecDeque::new();

    for x in 0..width {
        seed_ocean(
            heights,
            &mut relief_kind,
            &mut queue,
            width,
            x,
            0,
            ocean_level,
        );
        seed_ocean(
            heights,
            &mut relief_kind,
            &mut queue,
            width,
            x,
            height - 1,
            ocean_level,
        );
    }

    while let Some((x, y)) = queue.pop_front() {
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = wrap_x(x as i32 + dx, width);
                let ny = y as i32 + dy;
                if ny < 0 || ny >= height as i32 {
                    continue;
                }

                let ny = ny as u32;
                let index = index_of(width, nx, ny);
                if relief_kind[index] == ReliefKind::Plain && heights[index] <= ocean_level {
                    relief_kind[index] = ReliefKind::Ocean;
                    queue.push_back((nx, ny));
                }
            }
        }
    }

    for index in 0..area {
        if relief_kind[index] == ReliefKind::Plain && heights[index] <= ocean_level {
            relief_kind[index] = ReliefKind::Valley;
        }
    }

    relief_kind
}

fn seed_ocean(
    heights: &[f32],
    relief_kind: &mut [ReliefKind],
    queue: &mut VecDeque<(u32, u32)>,
    width: u32,
    x: u32,
    y: u32,
    ocean_level: f32,
) {
    let index = index_of(width, x, y);
    if heights[index] <= ocean_level && relief_kind[index] == ReliefKind::Plain {
        relief_kind[index] = ReliefKind::Ocean;
        queue.push_back((x, y));
    }
}

fn compute_local_relief(
    heights: &[f32],
    relief_kind: &[ReliefKind],
    width: u32,
    height: u32,
    radius: i32,
) -> Vec<f32> {
    let mut local_relief = vec![0.0; heights.len()];

    for y in 0..height {
        for x in 0..width {
            let index = index_of(width, x, y);
            if is_water(relief_kind[index]) {
                continue;
            }

            let mut min_height = f32::MAX;
            let mut has_land_neighbor = false;
            for dy in -radius..=radius {
                let ny = y as i32 + dy;
                if ny < 0 || ny >= height as i32 {
                    continue;
                }
                for dx in -radius..=radius {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = wrap_x(x as i32 + dx, width);
                    let neighbor_index = index_of(width, nx, ny as u32);
                    if !is_water(relief_kind[neighbor_index]) {
                        min_height = min_height.min(heights[neighbor_index]);
                        has_land_neighbor = true;
                    }
                }
            }

            if has_land_neighbor {
                local_relief[index] = (heights[index] - min_height).max(0.0);
            }
        }
    }

    local_relief
}

fn normalize_map(values: &mut [f32]) {
    let mut min_value = f32::MAX;
    let mut max_value = f32::MIN;
    for value in values.iter().copied() {
        min_value = min_value.min(value);
        max_value = max_value.max(value);
    }

    let range = if max_value - min_value > 1e-6 {
        max_value - min_value
    } else {
        1.0
    };

    for value in values {
        *value = (*value - min_value) / range;
    }
}

fn percentile(values: &mut [f32], percentile: f32) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(|a, b| a.total_cmp(b));
    if values.len() == 1 {
        return values[0];
    }

    let index = percentile * (values.len() - 1) as f32;
    let lower = index.floor() as usize;
    let fraction = index - lower as f32;
    if lower + 1 < values.len() {
        values[lower] + fraction * (values[lower + 1] - values[lower])
    } else {
        values[lower]
    }
}

fn is_water(kind: ReliefKind) -> bool {
    matches!(kind, ReliefKind::Ocean | ReliefKind::Valley)
}

fn wrap_x(x: i32, width: u32) -> u32 {
    let width = width as i32;
    ((x % width + width) % width) as u32
}

fn index_of(width: u32, x: u32, y: u32) -> usize {
    (y * width + x) as usize
}
