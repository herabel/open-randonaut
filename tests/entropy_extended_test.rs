// Extended entropy quality test suite
// This file contains several statistical tests that probe different aspects of the
// randomness produced by `generate_random_bytes`. All tests run as `cargo test`.

use randonautics::entropy::generate_random_bytes;
use statrs::distribution::{ChiSquared, ContinuousCDF};


/// Helper: compute Shannon entropy (bits per byte) of a slice.
fn shannon_entropy(data: &[u8]) -> f64 {
    let mut freq = [0usize; 256];
    for &b in data {
        freq[b as usize] += 1;
    }
    let n = data.len() as f64;
    let mut entropy = 0.0_f64;
    for &c in &freq {
        if c == 0 { continue; }
        let p = c as f64 / n;
        entropy -= p * p.log2();
    }
    entropy
}

/// Helper: convert 8 bytes (little‑endian) into a f64 in [0,1).
fn bytes_to_f64_le(bytes: &[u8]) -> f64 {
    // take first 8 bytes, pad with zeros if needed
    let mut buf = [0u8; 8];
    for (i, &b) in bytes.iter().take(8).enumerate() {
        buf[i] = b;
    }
    // interpret as u64 and divide by 2^64 to obtain a uniform double
    let as_u64 = u64::from_le_bytes(buf);
    (as_u64 as f64) / (u64::MAX as f64 + 1.0)
}

/// 1. Chi‑square test on raw byte distribution (already present, kept for completeness).
#[test]
fn chi_square_byte_distribution() {
    const SAMPLE_SIZE: usize = 1_000_000;
    let bytes = generate_random_bytes(SAMPLE_SIZE);
    let mut counts = [0usize; 256];
    for &b in &bytes { counts[b as usize] += 1; }
    let expected = SAMPLE_SIZE as f64 / 256.0;
    let chi2: f64 = counts.iter().map(|&c| {
        let diff = c as f64 - expected;
        diff * diff / expected
    }).sum();
    let chi = ChiSquared::new(255.0).expect("valid chi‑squared");
    let p = 1.0 - chi.cdf(chi2);
    println!("Chi2 = {chi2:.3}, p = {p:.6}");
    assert!(p > 0.01, "Byte distribution deviates (p = {p:.6})");
}

// KS test omitted – Kolmogorov distribution not available in `statrs` crate.

/// 3. Shannon entropy of the byte stream (bits per byte). Ideal = 8.
#[test]
fn shannon_entropy_check() {
    const SAMPLE_SIZE: usize = 500_000;
    let bytes = generate_random_bytes(SAMPLE_SIZE);
    let entropy = shannon_entropy(&bytes);
    println!("Shannon entropy = {entropy:.4} bits/byte");
    // 7.9 is a common practical threshold for cryptographic RNGs.
    assert!(entropy > 7.9, "Entropy too low ( {entropy:.4} bits/byte )");
}

/// 4. Serial correlation test – correlation between successive bytes.
#[test]
fn serial_correlation_bytes() {
    const SAMPLE_SIZE: usize = 500_000;
    let bytes = generate_random_bytes(SAMPLE_SIZE + 1); // need n+1 for pairwise comparison
    let mut sum_xy: i128 = 0;
    let mut sum_x: i128 = 0;
    let mut sum_y: i128 = 0;
    for i in 0..SAMPLE_SIZE {
        let x = bytes[i] as i128;
        let y = bytes[i + 1] as i128;
        sum_xy += x * y;
        sum_x += x;
        sum_y += y;
    }
    let n = SAMPLE_SIZE as i128;
    // Pearson correlation coefficient
    let numerator = n * sum_xy - sum_x * sum_y;
    let denominator = ((n * sum_x * sum_x - sum_x * sum_x).abs() as f64).sqrt() *
        ((n * sum_y * sum_y - sum_y * sum_y).abs() as f64).sqrt();
    let corr = numerator as f64 / denominator;
    println!("Serial correlation (bytes) = {corr:.6}");
    // For a good RNG correlation should be near 0; we allow a small epsilon.
    assert!(corr.abs() < 0.01, "Byte serial correlation too high ( {corr:.6} )");
}

/// 5. Runs test – counts runs of values above/below the median.
#[test]
fn runs_test_bytes() {
    const SAMPLE_SIZE: usize = 200_000;
    let bytes = generate_random_bytes(SAMPLE_SIZE);
    // median for uniform byte distribution is 127.5; we use 128 as threshold.
    let threshold = 128u8;
    let mut runs = 1usize; // start counting runs
    let mut last_above = bytes[0] >= threshold;
    for &b in &bytes[1..] {
        let above = b >= threshold;
        if above != last_above {
            runs += 1;
            last_above = above;
        }
    }
    // Expected number of runs for a random binary sequence of length n is
    //   E[R] = 2 * n * p * (1-p) + 1, where p = 0.5 → E[R] = n + 1.
    // Expected number of runs for a random binary sequence of length n with p=0.5
    let expected = (SAMPLE_SIZE as f64) / 2.0 + 1.0;
    // Variance for runs test with p=0.5: Var = (2n - 1)*(2n - 2) / (12 * (n - 1))
    let variance = ((SAMPLE_SIZE as f64 / 2.0) * ((SAMPLE_SIZE as f64 / 2.0) - 1.0)) / (2.0 * SAMPLE_SIZE as f64 - 1.0);
    let stddev = variance.sqrt();
    let z = (runs as f64 - expected as f64) / stddev;
    println!("Runs = {runs}, expected ≈ {expected}, z = {z:.3}");
    // |z| should be < 3 for 99.7% confidence.
    assert!(z.abs() < 3.0, "Runs test failed (|z| = {z:.3})");
}
