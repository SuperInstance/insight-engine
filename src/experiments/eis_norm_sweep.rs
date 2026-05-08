//! Experiment: Eisenstein norm distribution analysis.
//!
//! Probes: What happens to the norm distribution as we sweep across
//! the hex lattice? Are there unexpected gaps, clusters, or symmetries?

use crate::types::*;
use crate::surprise::{eis_norm, eis_in_disk, z_score};
use crate::surprise;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EisNormParams {
    /// Sweep radius
    pub radius: i32,
    /// Sampling density (1 = every point, 2 = every other, etc.)
    pub density: i32,
    /// Center offset
    pub center_a: i32,
    pub center_b: i32,
}

pub fn run(params: &EisNormParams, state: &mut EngineState) -> Observations {
    let mut norms = Vec::new();
    let mut zero_count = 0u32;
    let mut norm1_count = 0u32;
    let mut primes: Vec<i64> = Vec::new();

    for a in (-params.radius..=params.radius).step_by(params.density.max(1) as usize) {
        for b in (-params.radius..=params.radius).step_by(params.density.max(1) as usize) {
            let pa = a + params.center_a;
            let pb = b + params.center_b;
            let norm = eis_norm(pa, pb);
            norms.push(norm);

            match norm {
                0 => zero_count += 1,
                1 => norm1_count += 1,
                n if is_prime(n) => primes.push(n),
                _ => {}
            }
        }
    }

    let total = norms.len() as f64;
    let mean = norms.iter().sum::<i64>() as f64 / total;
    let variance = norms.iter()
        .map(|&n| (n as f64 - mean).powi(2))
        .sum::<f64>() / total;
    let std_dev = variance.sqrt();

    // Novel metric: ratio of prime norms to total norms
    let prime_ratio = primes.len() as f64 / total;

    // Novel metric: norm distribution skewness
    let skewness = if std_dev > 0.0 {
        norms.iter()
            .map(|&n| ((n as f64 - mean) / std_dev).powi(3))
            .sum::<f64>() / total
    } else {
        0.0
    };

    // Calculate surprise: is this distribution what we'd expect?
    // Expected: norm distribution follows N(a,b) = a²-ab+b² which is
    // approximately quadratic → mean ~ r²/3, variance ~ r⁴/45
    let expected_mean = (params.radius as f64).powi(2) / 3.0;
    let surprise_val = surprise::z_score(mean, expected_mean, (params.radius as f64).powi(4) / 45.0).abs();

    // Detect anomalies
    let mut anomalies = Vec::new();
    if prime_ratio > 0.4 {
        anomalies.push(format!("High prime ratio: {:.3}", prime_ratio));
    }
    if skewness.abs() > 1.0 {
        anomalies.push(format!("Strong skewness: {:.3}", skewness));
    }
    // Check if norm1_count is unexpectedly high (the 6 Eisenstein units)
    let expected_norm1 = 6.0; // always exactly 6 units
    if (norm1_count as f64 - expected_norm1).abs() > 0.5 && params.radius > 2 {
        anomalies.push(format!("Norm-1 count anomaly: {} (expected 6)", norm1_count));
    }

    let result = Observations {
        primary: prime_ratio,
        secondary: vec![
            ("mean_norm".into(), mean),
            ("std_dev".into(), std_dev),
            ("skewness".into(), skewness),
            ("prime_ratio".into(), prime_ratio),
            ("norm1_count".into(), norm1_count as f64),
            ("total_points".into(), total),
        ],
        success: true,
        convergence_steps: norms.len() as u32,
        anomalies,
    };

    // Update state
    let sig = format!("eis_norm_r{}_d{}_{}_{}", params.radius, params.density, params.center_a, params.center_b);
    state.update_patterns(ExperimentType::EisensteinNormSweep, sig, prime_ratio);
    state.record_region(ExperimentType::EisensteinNormSweep, serde_json::to_value(params).unwrap(), prime_ratio, surprise_val);

    result
}

fn is_prime(n: i64) -> bool {
    if n < 2 { return false; }
    if n < 4 { return true; }
    if n % 2 == 0 || n % 3 == 0 { return false; }
    let mut i = 5;
    while i * i <= n {
        if n % i == 0 || n % (i + 2) == 0 { return false; }
        i += 6;
    }
    true
}

impl Default for EisNormParams {
    fn default() -> Self {
        EisNormParams { radius: 10, density: 1, center_a: 0, center_b: 0 }
    }
}
