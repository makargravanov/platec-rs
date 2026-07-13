// SPDX-License-Identifier: LGPL-2.1-or-later
// Derived from Mindwerks Plate Tectonics; see THIRD_PARTY_NOTICES.md.
use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrustKind {
    Oceanic,
    Transitional,
    Continental,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryKind {
    None,
    OceanicBoundary,
    DivergentBoundary,
    TransformBoundary,
    SubductionCandidate,
    ContinentalCollision,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlateVelocity {
    pub x: f32,
    pub y: f32,
}

impl PlateVelocity {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GeologyInput<'a> {
    pub width: u32,
    pub height: u32,
    pub heightmap: &'a [f32],
    pub age_map: &'a [u32],
    pub plates_map: &'a [u32],
    pub plate_velocities: Option<&'a [PlateVelocity]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlatecGeology {
    width: u32,
    height: u32,
    crust_kind: Vec<CrustKind>,
    boundary_kind: Vec<BoundaryKind>,
    boundary_strength: Vec<f32>,
    convergence_strength: Vec<f32>,
    divergence_strength: Vec<f32>,
    transform_strength: Vec<f32>,
    orogenic_strength: Vec<f32>,
    volcanic_arc_strength: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeologyError {
    message: String,
}

impl fmt::Display for GeologyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for GeologyError {}

impl PlatecGeology {
    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    pub fn crust_kind(&self) -> &[CrustKind] {
        &self.crust_kind
    }

    pub fn boundary_kind(&self) -> &[BoundaryKind] {
        &self.boundary_kind
    }

    pub fn boundary_strength(&self) -> &[f32] {
        &self.boundary_strength
    }

    pub fn convergence_strength(&self) -> &[f32] {
        &self.convergence_strength
    }

    pub fn divergence_strength(&self) -> &[f32] {
        &self.divergence_strength
    }

    pub fn transform_strength(&self) -> &[f32] {
        &self.transform_strength
    }

    pub fn orogenic_strength(&self) -> &[f32] {
        &self.orogenic_strength
    }

    pub fn volcanic_arc_strength(&self) -> &[f32] {
        &self.volcanic_arc_strength
    }
}

pub fn analyze_geology(input: GeologyInput<'_>) -> Result<PlatecGeology, GeologyError> {
    let area = checked_area(input.width, input.height)?;
    validate_len("heightmap", input.heightmap.len(), area)?;
    validate_len("age_map", input.age_map.len(), area)?;
    validate_len("plates_map", input.plates_map.len(), area)?;
    validate_plate_velocities(input.plates_map, input.plate_velocities)?;

    let mut crust_kind = Vec::with_capacity(area);
    for height in input.heightmap.iter().copied() {
        crust_kind.push(classify_crust(height));
    }

    let mut geology = PlatecGeology {
        width: input.width,
        height: input.height,
        crust_kind,
        boundary_kind: vec![BoundaryKind::None; area],
        boundary_strength: vec![0.0; area],
        convergence_strength: vec![0.0; area],
        divergence_strength: vec![0.0; area],
        transform_strength: vec![0.0; area],
        orogenic_strength: vec![0.0; area],
        volcanic_arc_strength: vec![0.0; area],
    };

    for y in 0..input.height {
        for x in 0..input.width {
            let index = index_of(input.width, x, y);
            if input.width > 1 && !(input.width == 2 && x == 1) {
                let east = index_of(input.width, (x + 1) % input.width, y);
                record_boundary_pair(
                    &mut geology,
                    input.plates_map,
                    input.plate_velocities,
                    index,
                    east,
                    1.0,
                    0.0,
                );
            }

            if y + 1 < input.height {
                let south = index_of(input.width, x, y + 1);
                record_boundary_pair(
                    &mut geology,
                    input.plates_map,
                    input.plate_velocities,
                    index,
                    south,
                    0.0,
                    1.0,
                );
            }
        }
    }

    Ok(geology)
}

fn checked_area(width: u32, height: u32) -> Result<usize, GeologyError> {
    if width == 0 || height == 0 {
        return Err(GeologyError {
            message: "map dimensions must be non-zero".to_owned(),
        });
    }
    width
        .checked_mul(height)
        .map(|area| area as usize)
        .ok_or_else(|| GeologyError {
            message: "map area overflows u32".to_owned(),
        })
}

fn validate_len(name: &str, actual: usize, expected: usize) -> Result<(), GeologyError> {
    if actual == expected {
        Ok(())
    } else {
        Err(GeologyError {
            message: format!("{name} length {actual} does not match map area {expected}"),
        })
    }
}

fn validate_plate_velocities(
    plates_map: &[u32],
    plate_velocities: Option<&[PlateVelocity]>,
) -> Result<(), GeologyError> {
    let Some(plate_velocities) = plate_velocities else {
        return Ok(());
    };
    let Some(max_plate_id) = plates_map.iter().copied().max() else {
        return Ok(());
    };
    if max_plate_id as usize >= plate_velocities.len() {
        return Err(GeologyError {
            message: format!(
                "plate_velocities length {} does not contain plate id {max_plate_id}",
                plate_velocities.len()
            ),
        });
    }
    Ok(())
}

fn classify_crust(height: f32) -> CrustKind {
    if height < 0.85 {
        CrustKind::Oceanic
    } else if height < 1.05 {
        CrustKind::Transitional
    } else {
        CrustKind::Continental
    }
}

fn record_boundary_pair(
    geology: &mut PlatecGeology,
    plates_map: &[u32],
    plate_velocities: Option<&[PlateVelocity]>,
    a: usize,
    b: usize,
    normal_x: f32,
    normal_y: f32,
) {
    if plates_map[a] == plates_map[b] {
        return;
    }

    let motion = plate_velocities.map(|velocities| {
        classify_motion(
            velocities[plates_map[a] as usize],
            velocities[plates_map[b] as usize],
            normal_x,
            normal_y,
        )
    });
    let kind = classify_boundary(geology.crust_kind[a], geology.crust_kind[b], motion);
    apply_boundary(geology, a, kind);
    apply_boundary(geology, b, kind);

    if let Some(motion) = motion {
        apply_motion_strengths(geology, a, motion);
        apply_motion_strengths(geology, b, motion);
    }

    match kind {
        BoundaryKind::ContinentalCollision => {
            geology.orogenic_strength[a] = geology.orogenic_strength[a].max(1.0);
            geology.orogenic_strength[b] = geology.orogenic_strength[b].max(1.0);
        }
        BoundaryKind::SubductionCandidate => {
            mark_subduction_side(geology, a);
            mark_subduction_side(geology, b);
        }
        BoundaryKind::None
        | BoundaryKind::OceanicBoundary
        | BoundaryKind::DivergentBoundary
        | BoundaryKind::TransformBoundary => {}
    }
}

#[derive(Debug, Clone, Copy)]
struct BoundaryMotion {
    relation: MotionRelation,
    convergence: f32,
    divergence: f32,
    transform: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MotionRelation {
    Convergent,
    Divergent,
    Transform,
    Still,
}

fn classify_motion(
    a: PlateVelocity,
    b: PlateVelocity,
    normal_x: f32,
    normal_y: f32,
) -> BoundaryMotion {
    let relative_x = b.x - a.x;
    let relative_y = b.y - a.y;
    let normal_speed = relative_x * normal_x + relative_y * normal_y;
    let tangent_x = -normal_y;
    let tangent_y = normal_x;
    let tangent_speed = relative_x * tangent_x + relative_y * tangent_y;
    let convergence = (-normal_speed).max(0.0).min(1.0);
    let divergence = normal_speed.max(0.0).min(1.0);
    let transform = tangent_speed.abs().min(1.0);
    let relation = if convergence > 0.05 {
        MotionRelation::Convergent
    } else if divergence > 0.05 {
        MotionRelation::Divergent
    } else if transform > 0.05 {
        MotionRelation::Transform
    } else {
        MotionRelation::Still
    };

    BoundaryMotion {
        relation,
        convergence,
        divergence,
        transform,
    }
}

fn classify_boundary(a: CrustKind, b: CrustKind, motion: Option<BoundaryMotion>) -> BoundaryKind {
    match motion.map(|motion| motion.relation) {
        Some(MotionRelation::Divergent) => return BoundaryKind::DivergentBoundary,
        Some(MotionRelation::Transform) => return BoundaryKind::TransformBoundary,
        Some(MotionRelation::Still) => return BoundaryKind::OceanicBoundary,
        Some(MotionRelation::Convergent) | None => {}
    }

    match (a, b) {
        (CrustKind::Continental, CrustKind::Continental) => BoundaryKind::ContinentalCollision,
        (CrustKind::Oceanic, CrustKind::Continental)
        | (CrustKind::Continental, CrustKind::Oceanic)
        | (CrustKind::Oceanic, CrustKind::Transitional)
        | (CrustKind::Transitional, CrustKind::Oceanic) => BoundaryKind::SubductionCandidate,
        (CrustKind::Oceanic, CrustKind::Oceanic) => BoundaryKind::OceanicBoundary,
        (CrustKind::Transitional, CrustKind::Continental)
        | (CrustKind::Continental, CrustKind::Transitional)
        | (CrustKind::Transitional, CrustKind::Transitional) => BoundaryKind::ContinentalCollision,
    }
}

fn apply_motion_strengths(geology: &mut PlatecGeology, index: usize, motion: BoundaryMotion) {
    geology.convergence_strength[index] =
        geology.convergence_strength[index].max(motion.convergence);
    geology.divergence_strength[index] = geology.divergence_strength[index].max(motion.divergence);
    geology.transform_strength[index] = geology.transform_strength[index].max(motion.transform);
}

fn apply_boundary(geology: &mut PlatecGeology, index: usize, kind: BoundaryKind) {
    geology.boundary_strength[index] = geology.boundary_strength[index].max(1.0);
    if boundary_priority(kind) > boundary_priority(geology.boundary_kind[index]) {
        geology.boundary_kind[index] = kind;
    }
}

fn mark_subduction_side(geology: &mut PlatecGeology, index: usize) {
    if geology.crust_kind[index] != CrustKind::Oceanic {
        geology.orogenic_strength[index] = geology.orogenic_strength[index].max(1.0);
        geology.volcanic_arc_strength[index] = geology.volcanic_arc_strength[index].max(1.0);
    }
}

fn boundary_priority(kind: BoundaryKind) -> u8 {
    match kind {
        BoundaryKind::None => 0,
        BoundaryKind::OceanicBoundary => 1,
        BoundaryKind::DivergentBoundary => 2,
        BoundaryKind::TransformBoundary => 3,
        BoundaryKind::SubductionCandidate => 4,
        BoundaryKind::ContinentalCollision => 5,
    }
}

fn index_of(width: u32, x: u32, y: u32) -> usize {
    (y * width + x) as usize
}
