use platec_rs::geology::{
    BoundaryKind, CrustKind, GeologyInput, PlateVelocity, analyze_geology,
};

#[test]
fn geology_classifies_crust_from_height_and_age() {
    let geology = analyze_geology(GeologyInput {
        width: 4,
        height: 1,
        heightmap: &[0.2, 0.95, 1.05, 1.8],
        age_map: &[100, 5, 40, 80],
        plates_map: &[0, 0, 0, 0],
        plate_velocities: None,
    })
    .unwrap();

    assert_eq!(
        geology.crust_kind(),
        &[
            CrustKind::Oceanic,
            CrustKind::Transitional,
            CrustKind::Continental,
            CrustKind::Continental,
        ]
    );
}

#[test]
fn geology_detects_plate_boundaries_and_classifies_relation_by_crust() {
    let geology = analyze_geology(GeologyInput {
        width: 3,
        height: 1,
        heightmap: &[0.4, 1.6, 1.7],
        age_map: &[90, 80, 70],
        plates_map: &[1, 2, 2],
        plate_velocities: None,
    })
    .unwrap();

    assert_eq!(geology.boundary_strength(), &[1.0, 1.0, 1.0]);
    assert_eq!(
        geology.boundary_kind(),
        &[
            BoundaryKind::SubductionCandidate,
            BoundaryKind::SubductionCandidate,
            BoundaryKind::SubductionCandidate,
        ]
    );
    assert_eq!(geology.orogenic_strength(), &[0.0, 1.0, 1.0]);
    assert_eq!(geology.volcanic_arc_strength(), &[0.0, 1.0, 1.0]);
}

#[test]
fn geology_marks_continental_collision_as_orogenic_without_volcanic_arc() {
    let geology = analyze_geology(GeologyInput {
        width: 2,
        height: 1,
        heightmap: &[1.3, 1.8],
        age_map: &[80, 70],
        plates_map: &[1, 2],
        plate_velocities: None,
    })
    .unwrap();

    assert_eq!(
        geology.boundary_kind(),
        &[
            BoundaryKind::ContinentalCollision,
            BoundaryKind::ContinentalCollision
        ]
    );
    assert_eq!(geology.orogenic_strength(), &[1.0, 1.0]);
    assert_eq!(geology.volcanic_arc_strength(), &[0.0, 0.0]);
}

#[test]
fn geology_uses_plate_velocity_to_detect_divergent_boundaries() {
    let geology = analyze_geology(GeologyInput {
        width: 2,
        height: 1,
        heightmap: &[1.3, 1.4],
        age_map: &[80, 70],
        plates_map: &[0, 1],
        plate_velocities: Some(&[PlateVelocity::new(-1.0, 0.0), PlateVelocity::new(1.0, 0.0)]),
    })
    .unwrap();

    assert_eq!(
        geology.boundary_kind(),
        &[
            BoundaryKind::DivergentBoundary,
            BoundaryKind::DivergentBoundary
        ]
    );
    assert_eq!(geology.convergence_strength(), &[0.0, 0.0]);
    assert_eq!(geology.divergence_strength(), &[1.0, 1.0]);
    assert_eq!(geology.transform_strength(), &[0.0, 0.0]);
    assert_eq!(geology.orogenic_strength(), &[0.0, 0.0]);
}

#[test]
fn geology_uses_plate_velocity_to_detect_transform_boundaries() {
    let geology = analyze_geology(GeologyInput {
        width: 2,
        height: 1,
        heightmap: &[1.3, 1.4],
        age_map: &[80, 70],
        plates_map: &[0, 1],
        plate_velocities: Some(&[PlateVelocity::new(0.0, 1.0), PlateVelocity::new(0.0, -1.0)]),
    })
    .unwrap();

    assert_eq!(
        geology.boundary_kind(),
        &[
            BoundaryKind::TransformBoundary,
            BoundaryKind::TransformBoundary
        ]
    );
    assert_eq!(geology.convergence_strength(), &[0.0, 0.0]);
    assert_eq!(geology.divergence_strength(), &[0.0, 0.0]);
    assert_eq!(geology.transform_strength(), &[1.0, 1.0]);
    assert_eq!(geology.orogenic_strength(), &[0.0, 0.0]);
}

#[test]
fn geology_uses_plate_velocity_to_confirm_convergent_boundaries() {
    let geology = analyze_geology(GeologyInput {
        width: 2,
        height: 1,
        heightmap: &[1.3, 1.4],
        age_map: &[80, 70],
        plates_map: &[0, 1],
        plate_velocities: Some(&[PlateVelocity::new(1.0, 0.0), PlateVelocity::new(-1.0, 0.0)]),
    })
    .unwrap();

    assert_eq!(
        geology.boundary_kind(),
        &[
            BoundaryKind::ContinentalCollision,
            BoundaryKind::ContinentalCollision
        ]
    );
    assert_eq!(geology.convergence_strength(), &[1.0, 1.0]);
    assert_eq!(geology.divergence_strength(), &[0.0, 0.0]);
    assert_eq!(geology.transform_strength(), &[0.0, 0.0]);
    assert_eq!(geology.orogenic_strength(), &[1.0, 1.0]);
}

#[test]
fn geology_rejects_layers_with_wrong_length() {
    let error = analyze_geology(GeologyInput {
        width: 2,
        height: 2,
        heightmap: &[1.0, 1.0, 1.0],
        age_map: &[0, 0, 0, 0],
        plates_map: &[0, 0, 0, 0],
        plate_velocities: None,
    })
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "heightmap length 3 does not match map area 4"
    );
}

#[test]
fn geology_rejects_velocity_tables_missing_plate_ids() {
    let error = analyze_geology(GeologyInput {
        width: 2,
        height: 1,
        heightmap: &[1.0, 1.0],
        age_map: &[0, 0],
        plates_map: &[0, 2],
        plate_velocities: Some(&[PlateVelocity::new(0.0, 0.0), PlateVelocity::new(0.0, 0.0)]),
    })
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "plate_velocities length 2 does not contain plate id 2"
    );
}
