//! Entropy Tester Binary
//! Generates random bytes using the project's entropy module and runs statistical tests.
//!
//! Tests performed:
//! 1. **Shannon entropy** (bits per byte) – measures average information content.
//! 2. **Chi‑square test** for uniform byte distribution (256 bins).
//! 3. **Kolmogorov‑Smirnov test** – not available in `statrs` crate, omitted.
//!
//! Usage:
//! ```bash
//! cargo run --bin entropy_tester [NUM_BYTES]
//! ```
//! If `NUM_BYTES` is omitted, 1 000 000 bytes are generated.
//!
//! The binary is placed under `src/bin/entropy_tester.rs` so Cargo will treat it as
//! an additional executable target.

use std::env;
use randonautics::entropy::generate_random_bytes;
use statrs::distribution::{ChiSquared, ContinuousCDF};


fn shannon_entropy(data: &[u8]) -> f64 {
    // Compute frequency of each byte value (0..255).
    let mut counts = [0usize; 256];
    for &b in data {
        counts[b as usize] += 1;
    }
    let len = data.len() as f64;
    let mut entropy = 0.0_f64;
    for &c in &counts {
        if c == 0 { continue; }
        let p = c as f64 / len;
        entropy -= p * p.log2();
    }
    entropy
}

fn chi_square_test(data: &[u8]) -> (f64, f64) {
    // Expected count for uniform distribution.
    let n = data.len() as f64;
    let expected = n / 256.0;
    let mut chi2 = 0.0_f64;
    let mut counts = [0usize; 256];
    for &b in data {
        counts[b as usize] += 1;
    }
    for &c in &counts {
        let diff = c as f64 - expected;
        chi2 += diff * diff / expected;
    }
    // Degrees of freedom = bins-1 = 255.
    let chi_dist = ChiSquared::new(255.0).expect("valid chi‑square distribution");
    let p_val = 1.0 - chi_dist.cdf(chi2);
    (chi2, p_val)
}

// KS test removed – Kolmogorov distribution not available in this crate

fn main() {
    // Parse optional byte count.
    let args: Vec<String> = env::args().collect();
    let byte_count: usize = if args.len() > 1 {
        args[1].parse().unwrap_or(1_000_000)
    } else {
        1_000_000
    };
    println!("Generating {byte_count} random bytes using generate_random_bytes...");
    let data = generate_random_bytes(byte_count);

    // Shannon entropy.
    let entropy = shannon_entropy(&data);
    println!("Shannon entropy: {entropy:.4} bits/byte (max 8.0)\n");

    // Chi‑square test.
    let (chi2, chi_p) = chi_square_test(&data);
    println!("Chi‑square statistic: {chi2:.4}, p‑value: {chi_p:.6}");
    if chi_p < 0.05 {
        println!("⚠️  Distribution deviates from uniform (p < 0.05).\n");
    } else {
        println!("✅  No significant deviation from uniform (p >= 0.05).\n");
    }

// KS test omitted
}
