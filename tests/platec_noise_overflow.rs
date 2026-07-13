use platec_rs::{Lithosphere, PlatecConfig};

#[test]
fn lithosphere_accepts_seed_whose_noise_seed_square_exceeds_i64() {
    let result = std::panic::catch_unwind(|| {
        Lithosphere::new(PlatecConfig {
            seed: 424_242,
            width: 448,
            height: 256,
            ..PlatecConfig::default()
        })
    });

    assert!(
        result.is_ok(),
        "platec noise seed arithmetic must not rely on integer overflow"
    );
    assert!(result.unwrap().is_ok());
}
