use platec_rs::{
    geology::{GeologyInput, PlateVelocity, analyze_geology},
    terrain_refinement::{TerrainRefinementInput, refine_terrain},
};

#[test]
fn refinement_raises_and_spreads_continental_mountain_belts() {
    let heightmap = vec![1.2; 9];
    let geology = analyze_geology(GeologyInput {
        width: 3,
        height: 3,
        heightmap: &heightmap,
        age_map: &[80; 9],
        plates_map: &[0, 0, 0, 1, 1, 1, 1, 1, 1],
        plate_velocities: Some(&[PlateVelocity::new(0.0, 1.0), PlateVelocity::new(0.0, -1.0)]),
    })
    .unwrap();

    let refined = refine_terrain(TerrainRefinementInput {
        width: 3,
        height: 3,
        heightmap: &heightmap,
        geology: &geology,
    })
    .unwrap();

    assert!(refined.heightmap()[1] > 1.7);
    assert!(refined.heightmap()[4] > 1.7);
    assert!(refined.heightmap()[7] > heightmap[7]);
    assert!(refined.mountain_potential()[1] > refined.mountain_potential()[7]);
}

#[test]
fn refinement_marks_oceanic_rifts_without_turning_them_into_mountains() {
    let heightmap = vec![0.4; 6];
    let geology = analyze_geology(GeologyInput {
        width: 3,
        height: 2,
        heightmap: &heightmap,
        age_map: &[10; 6],
        plates_map: &[0, 0, 0, 1, 1, 1],
        plate_velocities: Some(&[PlateVelocity::new(0.0, -1.0), PlateVelocity::new(0.0, 1.0)]),
    })
    .unwrap();

    let refined = refine_terrain(TerrainRefinementInput {
        width: 3,
        height: 2,
        heightmap: &heightmap,
        geology: &geology,
    })
    .unwrap();

    assert!(refined.rift_potential()[1] > 0.9);
    assert!(refined.heightmap()[1] > heightmap[1]);
    assert!(refined.heightmap()[1] < 0.8);
    assert_eq!(refined.mountain_potential()[1], 0.0);
}

#[test]
fn refinement_marks_transform_faults_without_large_uplift() {
    let heightmap = vec![1.2; 6];
    let geology = analyze_geology(GeologyInput {
        width: 3,
        height: 2,
        heightmap: &heightmap,
        age_map: &[80; 6],
        plates_map: &[0, 0, 0, 1, 1, 1],
        plate_velocities: Some(&[PlateVelocity::new(1.0, 0.0), PlateVelocity::new(-1.0, 0.0)]),
    })
    .unwrap();

    let refined = refine_terrain(TerrainRefinementInput {
        width: 3,
        height: 2,
        heightmap: &heightmap,
        geology: &geology,
    })
    .unwrap();

    assert!(refined.fault_potential()[1] > 0.9);
    assert!((refined.heightmap()[1] - heightmap[1]).abs() < 0.05);
    assert_eq!(refined.mountain_potential()[1], 0.0);
}

#[test]
fn refinement_builds_volcanic_arc_on_continental_side_of_subduction() {
    let heightmap = vec![0.4, 0.4, 0.4, 1.2, 1.2, 1.2];
    let geology = analyze_geology(GeologyInput {
        width: 3,
        height: 2,
        heightmap: &heightmap,
        age_map: &[10, 10, 10, 80, 80, 80],
        plates_map: &[0, 0, 0, 1, 1, 1],
        plate_velocities: Some(&[PlateVelocity::new(0.0, 1.0), PlateVelocity::new(0.0, -1.0)]),
    })
    .unwrap();

    let refined = refine_terrain(TerrainRefinementInput {
        width: 3,
        height: 2,
        heightmap: &heightmap,
        geology: &geology,
    })
    .unwrap();

    assert_eq!(refined.volcanic_potential()[1], 0.0);
    assert!(refined.volcanic_potential()[4] > 0.9);
    assert!(refined.heightmap()[4] > heightmap[4]);
}

#[test]
fn refinement_rejects_heightmap_with_wrong_length() {
    let heightmap = vec![1.0; 4];
    let geology = analyze_geology(GeologyInput {
        width: 2,
        height: 2,
        heightmap: &heightmap,
        age_map: &[0; 4],
        plates_map: &[0; 4],
        plate_velocities: None,
    })
    .unwrap();

    let error = refine_terrain(TerrainRefinementInput {
        width: 2,
        height: 2,
        heightmap: &[1.0; 3],
        geology: &geology,
    })
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "heightmap length 3 does not match map area 4"
    );
}
