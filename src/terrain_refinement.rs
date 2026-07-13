// SPDX-License-Identifier: LGPL-2.1-or-later
// Derived from Mindwerks Plate Tectonics; see THIRD_PARTY_NOTICES.md.
use core::fmt;

use super::geology::{CrustKind, PlatecGeology};

#[derive(Debug, Clone, Copy)]
pub struct TerrainRefinementInput<'a> {
    pub width: u32,
    pub height: u32,
    pub heightmap: &'a [f32],
    pub geology: &'a PlatecGeology,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RefinedTerrain {
    heightmap: Vec<f32>,
    mountain_potential: Vec<f32>,
    volcanic_potential: Vec<f32>,
    rift_potential: Vec<f32>,
    fault_potential: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerrainRefinementError {
    message: String,
}

impl fmt::Display for TerrainRefinementError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for TerrainRefinementError {}

impl RefinedTerrain {
    pub fn heightmap(&self) -> &[f32] {
        &self.heightmap
    }

    pub fn mountain_potential(&self) -> &[f32] {
        &self.mountain_potential
    }

    pub fn volcanic_potential(&self) -> &[f32] {
        &self.volcanic_potential
    }

    pub fn rift_potential(&self) -> &[f32] {
        &self.rift_potential
    }

    pub fn fault_potential(&self) -> &[f32] {
        &self.fault_potential
    }
}

pub fn refine_terrain(
    input: TerrainRefinementInput<'_>,
) -> Result<RefinedTerrain, TerrainRefinementError> {
    let area = checked_area(input.width, input.height)?;
    if input.geology.width() != input.width || input.geology.height() != input.height {
        return Err(TerrainRefinementError {
            message: format!(
                "geology dimensions {}x{} do not match terrain dimensions {}x{}",
                input.geology.width(),
                input.geology.height(),
                input.width,
                input.height
            ),
        });
    }
    validate_len("heightmap", input.heightmap.len(), area)?;

    let mut mountain_sources = vec![0.0; area];
    let mut volcanic_sources = vec![0.0; area];
    let mut rift_sources = vec![0.0; area];
    let mut fault_sources = vec![0.0; area];

    for index in 0..area {
        mountain_sources[index] = input.geology.orogenic_strength()[index]
            * input.geology.convergence_strength()[index].max(0.75);
        volcanic_sources[index] = input.geology.volcanic_arc_strength()[index];
        rift_sources[index] = input.geology.divergence_strength()[index];
        fault_sources[index] = input.geology.transform_strength()[index];
    }

    let mountain_potential =
        spread_sources(input.width, input.height, &mountain_sources, 2, |_| true);
    let volcanic_potential =
        spread_sources(input.width, input.height, &volcanic_sources, 2, |index| {
            input.geology.crust_kind()[index] != CrustKind::Oceanic
        });
    let rift_potential = spread_sources(input.width, input.height, &rift_sources, 1, |_| true);
    let fault_potential = spread_sources(input.width, input.height, &fault_sources, 1, |_| true);

    let mut heightmap = input.heightmap.to_vec();
    for index in 0..area {
        heightmap[index] += mountain_potential[index] * 0.75;
        heightmap[index] += volcanic_potential[index] * 0.35;
        heightmap[index] += rift_potential[index] * 0.22;
    }

    Ok(RefinedTerrain {
        heightmap,
        mountain_potential,
        volcanic_potential,
        rift_potential,
        fault_potential,
    })
}

fn checked_area(width: u32, height: u32) -> Result<usize, TerrainRefinementError> {
    if width == 0 || height == 0 {
        return Err(TerrainRefinementError {
            message: "terrain dimensions must be non-zero".to_owned(),
        });
    }
    width
        .checked_mul(height)
        .map(|area| area as usize)
        .ok_or_else(|| TerrainRefinementError {
            message: "terrain area overflows u32".to_owned(),
        })
}

fn validate_len(name: &str, actual: usize, expected: usize) -> Result<(), TerrainRefinementError> {
    if actual == expected {
        Ok(())
    } else {
        Err(TerrainRefinementError {
            message: format!("{name} length {actual} does not match map area {expected}"),
        })
    }
}

fn spread_sources(
    width: u32,
    height: u32,
    sources: &[f32],
    radius: i32,
    allowed: impl Fn(usize) -> bool,
) -> Vec<f32> {
    let mut result = vec![0.0_f32; sources.len()];
    for y in 0..height as i32 {
        for x in 0..width as i32 {
            let source_index = index_of(width, x as u32, y as u32);
            let source = sources[source_index];
            if source <= 0.0 {
                continue;
            }

            for dy in -radius..=radius {
                let target_y = y + dy;
                if target_y < 0 || target_y >= height as i32 {
                    continue;
                }
                for dx in -radius..=radius {
                    let distance = ((dx * dx + dy * dy) as f32).sqrt();
                    if distance > radius as f32 {
                        continue;
                    }
                    let target_x = (x + dx).rem_euclid(width as i32) as u32;
                    let target_index = index_of(width, target_x, target_y as u32);
                    if !allowed(target_index) {
                        continue;
                    }
                    let weight = 1.0 - distance / (radius as f32 + 1.0);
                    result[target_index] = result[target_index].max(source * weight);
                }
            }
        }
    }
    result
}

fn index_of(width: u32, x: u32, y: u32) -> usize {
    (y * width + x) as usize
}
