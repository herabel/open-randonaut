// Integration test for entropy quality
// This test generates a large amount of random bytes using the project's
// `generate_random_bytes` function and runs a chi‑square goodness‑of‑fit
// test against the uniform distribution of byte values (0..255).
// A p‑value > 0.01 indicates that the observed distribution does not
// significantly deviate from uniform at the 99 % confidence level.

use randonautics::entropy::generate_random_bytes;
use statrs::distribution::{ChiSquared, ContinuousCDF};

#[test]
fn chi_square_byte_distribution() {
    // Number of random bytes to sample – 1 000 000 gives a good balance
    // between statistical power and test runtime (< 1 s on modern HW).
    const SAMPLE_SIZE: usize = 1_000_000;

    // Get raw entropy from the library (uses OS entropy + optional RDRAND/RDSEED).
    let bytes = generate_random_bytes(SAMPLE_SIZE);

    // Count occurrences of each possible byte value.
    let mut counts = [0usize; 256];
    for &b in &bytes {
        counts[b as usize] += 1;
    }

    // Expected count per bucket under perfect uniformity.
    let expected = SAMPLE_SIZE as f64 / 256.0;
    let mut chi2 = 0.0_f64;
    for &c in &counts {
        let diff = c as f64 - expected;
        chi2 += diff * diff / expected;
    }

    // Degrees of freedom = number of categories - 1 = 255.
    let chi = ChiSquared::new(255.0).expect("valid chi‑squared distribution");
    // Upper‑tail probability: probability of observing a chi2 >= observed.
    let p_value = 1.0 - chi.cdf(chi2);

    println!("Chi‑squared = {chi2:.3}, p‑value = {p_value:.6}");
    // A very low p‑value (< 0.01) would indicate non‑uniformity.
    assert!(p_value > 0.01, "Byte distribution deviates (p = {p_value:.6})");
}
