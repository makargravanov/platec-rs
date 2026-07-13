use platec_rs::{Lithosphere, PlatecConfig, SimpleRandom};

#[test]
fn simple_random_matches_original_sequence() {
    let mut random = SimpleRandom::new(3);

    assert!((random.next_f32() - 5.1118433e-05).abs() <= f32::EPSILON);
    assert!((random.next_f32() - 0.53070194).abs() <= f32::EPSILON);
    assert!((random.next_f32() - 0.053402752).abs() <= f32::EPSILON);
    assert_eq!(SimpleRandom::maximum(), u32::MAX);
}

#[test]
fn lithosphere_rejects_too_small_dimensions() {
    let config = PlatecConfig {
        seed: 3,
        width: 4,
        height: 5,
        sea_level: 0.65,
        erosion_period: 60,
        folding_ratio: 0.02,
        aggregation_overlap_abs: 1_000_000,
        aggregation_overlap_rel: 0.33,
        cycle_count: 2,
        plate_count: 10,
    };

    assert!(Lithosphere::new(config).is_err());
}

#[test]
fn lithosphere_initial_heightmap_matches_original_statistics() {
    let lithosphere = Lithosphere::new(PlatecConfig {
        seed: 12_345,
        width: 600,
        height: 400,
        sea_level: 0.65,
        erosion_period: 60,
        folding_ratio: 0.02,
        aggregation_overlap_abs: 1_000_000,
        aggregation_overlap_rel: 0.33,
        cycle_count: 2,
        plate_count: 10,
    })
    .unwrap();

    let stats = HeightStats::from(lithosphere.heightmap());

    assert!((stats.min - 0.1).abs() < 0.000_001);
    assert!((stats.max - 2.0).abs() < 0.000_001);
    assert!((stats.mean - 0.689_232).abs() < 0.000_1);
    assert!((stats.median - 0.1).abs() < 0.000_001);
    assert!((stats.std_dev - 0.779_593).abs() < 0.000_1);
    assert!((stats.q25 - 0.1).abs() < 0.000_001);
    assert!((stats.q75 - 1.638_43).abs() < 0.000_1);
}

#[test]
fn lithosphere_creates_requested_plates_and_assigns_every_tile() {
    let lithosphere = Lithosphere::new(PlatecConfig {
        seed: 3,
        width: 64,
        height: 48,
        sea_level: 0.65,
        erosion_period: 60,
        folding_ratio: 0.02,
        aggregation_overlap_abs: 1_000_000,
        aggregation_overlap_rel: 0.33,
        cycle_count: 2,
        plate_count: 10,
    })
    .unwrap();

    assert_eq!(lithosphere.plate_count(), 10);
    assert!(!lithosphere.is_finished());
    assert!(lithosphere.plates_map().iter().all(|id| *id < 10));
}

#[test]
fn lithosphere_final_heightmap_matches_original_x86_statistics() {
    let mut lithosphere = Lithosphere::new(PlatecConfig {
        seed: 12_345,
        width: 600,
        height: 400,
        sea_level: 0.65,
        erosion_period: 60,
        folding_ratio: 0.02,
        aggregation_overlap_abs: 1_000_000,
        aggregation_overlap_rel: 0.33,
        cycle_count: 2,
        plate_count: 10,
    })
    .unwrap();

    while !lithosphere.is_finished() {
        lithosphere.step();
    }

    let stats = HeightStats::from(lithosphere.heightmap());

    assert!((stats.min - 0.042_391_6).abs() < 0.01);
    assert!((stats.max - 17.8405).abs() < 2.7);
    assert!((stats.mean - 0.624_06).abs() < 0.01);
    assert!((stats.median - 0.114_578).abs() < 0.01);
    assert!((stats.std_dev - 0.945_673).abs() < 0.02);
    assert!((stats.q25 - 0.098_344_5).abs() < 0.01);
    assert!((stats.q75 - 0.924_061).abs() < 0.02);
}

#[derive(Debug)]
struct HeightStats {
    min: f32,
    max: f32,
    mean: f32,
    median: f32,
    std_dev: f32,
    q25: f32,
    q75: f32,
}

impl HeightStats {
    fn from(values: &[f32]) -> Self {
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.total_cmp(b));
        let sum = values.iter().map(|value| *value as f64).sum::<f64>();
        let mean = (sum / values.len() as f64) as f32;
        let variance = values
            .iter()
            .map(|value| {
                let diff = *value as f64 - mean as f64;
                diff * diff
            })
            .sum::<f64>()
            / values.len() as f64;

        Self {
            min: sorted[0],
            max: sorted[sorted.len() - 1],
            mean,
            median: sorted[sorted.len() / 2],
            std_dev: variance.sqrt() as f32,
            q25: sorted[sorted.len() / 4],
            q75: sorted[(sorted.len() * 3) / 4],
        }
    }
}
