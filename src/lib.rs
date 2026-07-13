// SPDX-License-Identifier: LGPL-2.1-or-later
// Derived from Mindwerks Plate Tectonics; see THIRD_PARTY_NOTICES.md.
mod bounds;
pub mod geology;
mod geometry;
mod lithosphere;
mod mass;
mod matrix;
mod movement;
mod plate;
mod plate_functions;
mod random;
mod rectangle;
mod segment_data;
mod segments;
mod simplexnoise;
pub mod terrain_classification;
pub mod terrain_refinement;

pub use lithosphere::{Lithosphere, PlatecConfig, PlatecError};
pub use random::SimpleRandom;

pub const PLATEC_ABI_VERSION: u32 = 1;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PlatecMapRequest {
    pub abi_version: u32,
    pub seed: u32,
    pub width: u32,
    pub height: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct PlatecMapOutput {
    pub heightmap: *mut f32,
    pub relief_kind: *mut u8,
    pub len: usize,
    pub steps: *mut u32,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatecMapStatus {
    Ok = 0,
    NullPointer = 1,
    AbiMismatch = 2,
    InvalidDimensions = 3,
    InvalidOutputLength = 4,
    GenerationFailed = 5,
}

/// Runs the entire Platec pipeline once and writes only into caller-owned buffers.
///
/// # Safety
///
/// `request` and `output` must point to valid values. The two output arrays must be
/// writable for exactly `width * height` elements, and `steps` must be writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn platec_generate_map_once(
    request: *const PlatecMapRequest,
    output: *mut PlatecMapOutput,
) -> PlatecMapStatus {
    if request.is_null() || output.is_null() {
        return PlatecMapStatus::NullPointer;
    }

    // SAFETY: null pointers are rejected above; caller upholds validity requirements.
    let request = unsafe { *request };
    // SAFETY: null pointers are rejected above; caller upholds validity requirements.
    let output = unsafe { &mut *output };
    if request.abi_version != PLATEC_ABI_VERSION {
        return PlatecMapStatus::AbiMismatch;
    }
    let Some(area) = request
        .width
        .checked_mul(request.height)
        .map(|area| area as usize)
    else {
        return PlatecMapStatus::InvalidDimensions;
    };
    if request.width < 5 || request.height < 5 {
        return PlatecMapStatus::InvalidDimensions;
    }
    if output.len != area {
        return PlatecMapStatus::InvalidOutputLength;
    }
    if output.heightmap.is_null() || output.relief_kind.is_null() || output.steps.is_null() {
        return PlatecMapStatus::NullPointer;
    }

    let Ok((heightmap, relief_kind, steps)) = generate_map(request.seed, request.width, request.height)
    else {
        return PlatecMapStatus::GenerationFailed;
    };
    // SAFETY: the caller supplied writable buffers with exactly `area` elements.
    unsafe {
        core::ptr::copy_nonoverlapping(heightmap.as_ptr(), output.heightmap, area);
        core::ptr::copy_nonoverlapping(relief_kind.as_ptr(), output.relief_kind, area);
        *output.steps = steps;
    }
    PlatecMapStatus::Ok
}

fn generate_map(seed: u32, width: u32, height: u32) -> Result<(Vec<f32>, Vec<u8>, u32), ()> {
    let config = PlatecConfig {
        seed,
        width,
        height,
        ..PlatecConfig::default()
    };
    let mut lithosphere = Lithosphere::new(config).map_err(|_| ())?;
    let mut steps = 0;
    while !lithosphere.is_finished() {
        lithosphere.step();
        steps += 1;
    }
    let geology = geology::analyze_geology(geology::GeologyInput {
        width,
        height,
        heightmap: lithosphere.heightmap(),
        age_map: lithosphere.age_map(),
        plates_map: lithosphere.plates_map(),
        plate_velocities: Some(lithosphere.plate_velocities()),
    })
    .map_err(|_| ())?;
    let refined = terrain_refinement::refine_terrain(terrain_refinement::TerrainRefinementInput {
        width,
        height,
        heightmap: lithosphere.heightmap(),
        geology: &geology,
    })
    .map_err(|_| ())?;
    let classification = terrain_classification::classify_terrain(
        terrain_classification::TerrainClassificationInput {
            width,
            height,
            heightmap: refined.heightmap(),
            ocean_level: config.sea_level,
        },
    )
    .map_err(|_| ())?;
    let relief_kind = classification
        .relief_kind()
        .iter()
        .copied()
        .map(relief_kind_code)
        .collect();

    Ok((refined.heightmap().to_vec(), relief_kind, steps))
}

fn relief_kind_code(kind: terrain_classification::ReliefKind) -> u8 {
    match kind {
        terrain_classification::ReliefKind::Ocean => 0,
        terrain_classification::ReliefKind::Valley => 1,
        terrain_classification::ReliefKind::Plain => 2,
        terrain_classification::ReliefKind::Hill => 3,
        terrain_classification::ReliefKind::Mountain => 4,
        terrain_classification::ReliefKind::HighMountain => 5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_shot_ffi_fills_caller_owned_map_buffers() {
        let width = 448;
        let height = 256;
        let mut heightmap = vec![0.0; width * height];
        let mut relief_kind = vec![u8::MAX; width * height];
        let mut steps = 0;
        let request = PlatecMapRequest {
            abi_version: PLATEC_ABI_VERSION,
            seed: 17,
            width: width as u32,
            height: height as u32,
        };
        let mut output = PlatecMapOutput {
            heightmap: heightmap.as_mut_ptr(),
            relief_kind: relief_kind.as_mut_ptr(),
            len: heightmap.len(),
            steps: &mut steps,
        };

        let status = unsafe { platec_generate_map_once(&request, &mut output) };

        assert_eq!(status, PlatecMapStatus::Ok);
        assert!(steps > 0);
        assert!(heightmap.iter().all(|height| height.is_finite()));
        assert!(relief_kind.iter().all(|kind| *kind <= 5));
    }

    #[test]
    fn small_map_reaches_each_generation_stage() {
        let config = PlatecConfig {
            seed: 17,
            width: 448,
            height: 256,
            ..PlatecConfig::default()
        };
        let mut lithosphere = Lithosphere::new(config).unwrap();
        while !lithosphere.is_finished() {
            lithosphere.step();
        }
        let geology = geology::analyze_geology(geology::GeologyInput {
            width: config.width,
            height: config.height,
            heightmap: lithosphere.heightmap(),
            age_map: lithosphere.age_map(),
            plates_map: lithosphere.plates_map(),
            plate_velocities: Some(lithosphere.plate_velocities()),
        });
        assert!(geology.is_ok(), "geology failed: {geology:?}");
    }
}
