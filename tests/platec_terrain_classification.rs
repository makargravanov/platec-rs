use platec_rs::terrain_classification::{
    ReliefKind, TerrainClassificationInput, classify_terrain,
};

#[test]
fn sharp_local_peak_becomes_mountain() {
    let mut heightmap = vec![1.0; 25];
    heightmap[12] = 3.0;

    let terrain = classify_terrain(TerrainClassificationInput {
        width: 5,
        height: 5,
        heightmap: &heightmap,
        ocean_level: 0.5,
    })
    .unwrap();

    assert_eq!(terrain.relief_kind()[12], ReliefKind::Mountain);
}

#[test]
fn high_flat_plateau_does_not_become_mountain() {
    let heightmap = vec![3.0; 25];

    let terrain = classify_terrain(TerrainClassificationInput {
        width: 5,
        height: 5,
        heightmap: &heightmap,
        ocean_level: 0.5,
    })
    .unwrap();

    assert!(
        terrain
            .relief_kind()
            .iter()
            .all(|kind| *kind == ReliefKind::Plain)
    );
}

#[test]
fn secondary_local_peak_becomes_hill_not_mountain() {
    let mut heightmap = vec![1.0; 25];
    heightmap[12] = 3.0;
    heightmap[6] = 2.0;

    let terrain = classify_terrain(TerrainClassificationInput {
        width: 5,
        height: 5,
        heightmap: &heightmap,
        ocean_level: 0.5,
    })
    .unwrap();

    assert_eq!(terrain.relief_kind()[12], ReliefKind::Mountain);
    assert_eq!(terrain.relief_kind()[6], ReliefKind::Hill);
}

#[test]
fn water_connected_to_border_is_ocean_and_enclosed_water_is_valley() {
    let heightmap = vec![
        0.2, 1.0, 1.0, 1.0, 1.0, //
        1.0, 1.0, 1.0, 1.0, 1.0, //
        1.0, 1.0, 0.2, 1.0, 1.0, //
        1.0, 1.0, 1.0, 1.0, 1.0, //
        1.0, 1.0, 1.0, 1.0, 1.0, //
    ];

    let terrain = classify_terrain(TerrainClassificationInput {
        width: 5,
        height: 5,
        heightmap: &heightmap,
        ocean_level: 0.5,
    })
    .unwrap();

    assert_eq!(terrain.relief_kind()[0], ReliefKind::Ocean);
    assert_eq!(terrain.relief_kind()[12], ReliefKind::Valley);
}

#[test]
fn below_sea_component_crossing_horizontal_seam_is_enclosed_valley() {
    let heightmap = vec![
        1.0, 1.0, 1.0, 1.0, 1.0, //
        0.2, 1.0, 1.0, 1.0, 0.2, //
        1.0, 1.0, 1.0, 1.0, 1.0, //
    ];

    let terrain = classify_terrain(TerrainClassificationInput {
        width: 5,
        height: 3,
        heightmap: &heightmap,
        ocean_level: 0.5,
    })
    .unwrap();

    assert_eq!(terrain.relief_kind()[5], ReliefKind::Valley);
    assert_eq!(terrain.relief_kind()[9], ReliefKind::Valley);
}

#[test]
fn ocean_connectivity_wraps_across_horizontal_seam() {
    let heightmap = vec![
        1.0, 1.0, 1.0, 1.0, 0.2, //
        0.2, 1.0, 1.0, 1.0, 0.2, //
        0.2, 1.0, 1.0, 1.0, 1.0, //
        1.0, 1.0, 1.0, 1.0, 1.0, //
    ];

    let terrain = classify_terrain(TerrainClassificationInput {
        width: 5,
        height: 4,
        heightmap: &heightmap,
        ocean_level: 0.5,
    })
    .unwrap();

    for index in [4, 5, 9, 10] {
        assert_eq!(terrain.relief_kind()[index], ReliefKind::Ocean);
    }
}
