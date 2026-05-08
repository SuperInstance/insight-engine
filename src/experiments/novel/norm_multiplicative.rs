//! Experiment: Eisenstein norm multiplicative structure.
//!
//! Eisenstein norms are MULTPLICATIVE: N(αβ) = N(α)·N(β).
//! This experiment probes: what happens when you study the
//! SEMI-GROUP structure? Where are the unexpected factorizations?
//!
//! PROBES:
//! - Are there norm values with MORE factorizations than expected?
//! - Do factorization counts follow a predictable distribution?
//! - Where does multiplicativity create "resonances" (clustering)?

use crate::types::*;
use crate::types::{ExploredRegion, Pattern};
use crate::surprise::{eis_norm, z_score};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NormMultiplicativeParams {
    pub max_norm: i64,
    /// Check for "resonances": norm values with unexpectedly many factorizations
    pub resonance_threshold: f64,
    /// Also check: do Eisenstein norms have the same distribution as ℤ norms?
    pub compare_with_integers: bool,
}

/// A resonance: a norm value with anomalous factorization count.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Resonance {
    pub norm_value: i64,
    pub factorization_count: usize,
    pub expected_count: f64,
    pub z_score: f64,
    /// The actual factor pairs
    pub factor_pairs: Vec<(i64, i64)>,
}

/// A gap: a norm value that SHOULD appear but doesn't.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NormGap {
    pub value: i64,
    /// How many Eisenstein integers have this norm (should be > 0)
    pub representation_count: usize,
    /// Factorizations that should produce this norm but don't
    pub missing_from: Vec<(i64, i64)>,
}

pub fn run(params: &NormMultiplicativeParams, state: &mut EngineState) -> Observations {
    // Step 1: Build norm frequency table
    let mut norm_counts: std::collections::HashMap<i64, usize> = std::collections::HashMap::new();
    let mut norm_sources: std::collections::HashMap<i64, Vec<(i32, i32)>> = std::collections::HashMap::new();

    let radius = (params.max_norm as f64).sqrt() as i32 + 2;
    for a in -radius..=radius {
        for b in -radius..=radius {
            let n = eis_norm(a, b);
            if n > 0 && n <= params.max_norm {
                *norm_counts.entry(n).or_insert(0) += 1;
                norm_sources.entry(n).or_default().push((a, b));
            }
        }
    }

    // Step 2: For each representable norm, count factorizations within the lattice
    let mut resonances: Vec<Resonance> = Vec::new();
    let mut factorization_counts: std::collections::HashMap<i64, usize> = std::collections::HashMap::new();
    let mut factor_pairs_map: std::collections::HashMap<i64, Vec<(i64, i64)>> = std::collections::HashMap::new();

    let representable_norms: Vec<i64> = norm_counts.keys().copied().collect();
    for &n in &representable_norms {
        let mut count = 0;
        let mut pairs = Vec::new();
        // Check all pairs (a, b) where a * b = n and both are representable
        for &a in &representable_norms {
            if a > 0 && n % a == 0 {
                let b = n / a;
                if norm_counts.contains_key(&b) {
                    count += 1;
                    pairs.push((a, b));
                }
            }
        }
        factorization_counts.insert(n, count);
        factor_pairs_map.insert(n, pairs);
    }

    // Step 3: Compute expected factorization count (logarithmic baseline)
    let counts_vec: Vec<f64> = factorization_counts.values().map(|&c| c as f64).collect();
    let mean_count = counts_vec.iter().sum::<f64>() / counts_vec.len().max(1) as f64;
    let std_count = {
        let variance = counts_vec.iter()
            .map(|c| (c - mean_count).powi(2))
            .sum::<f64>() / counts_vec.len().max(1) as f64;
        variance.sqrt().max(0.1)
    };

    for (&n, &count) in &factorization_counts {
        let z = z_score(count as f64, mean_count, std_count);
        if z.abs() > params.resonance_threshold {
            resonances.push(Resonance {
                norm_value: n,
                factorization_count: count,
                expected_count: mean_count,
                z_score: z,
                factor_pairs: factor_pairs_map.get(&n).cloned().unwrap_or_default(),
            });
        }
    }

    // Step 4: Check for norm gaps — values that are products of representable norms
    // but are NOT themselves representable
    let mut gaps: Vec<NormGap> = Vec::new();
    if params.compare_with_integers {
        for &a in &representable_norms {
            for &b in &representable_norms {
                let product = a * b;
                if product > 0 && product <= params.max_norm && !norm_counts.contains_key(&product) {
                    // This norm SHOULD be representable by multiplicativity, but isn't found
                    gaps.push(NormGap {
                        value: product,
                        representation_count: 0,
                        missing_from: vec![(a, b)],
                    });
                }
            }
        }
    }

    // Step 5: Analyze clustering — do resonances cluster at certain residue classes?
    let mut anomaly_texts = Vec::new();
    let mut secondary = Vec::new();

    if !resonances.is_empty() {
        let max_res = resonances.iter().max_by(|a, b| a.z_score.abs().partial_cmp(&b.z_score.abs()).unwrap()).unwrap();
        anomaly_texts.push(format!(
            "Norm resonance at N={} (z={:.2}, {} factorizations, expected {:.1})",
            max_res.norm_value, max_res.z_score, max_res.factorization_count, max_res.expected_count
        ));

        // Check mod 6 clustering (Eisenstein integers have natural mod-3 structure)
        let mut mod3_counts = [0usize; 3];
        for r in &resonances {
            mod3_counts[(r.norm_value % 3).rem_euclid(3) as usize] += 1;
        }
        let dominant_mod = mod3_counts.iter().enumerate()
            .max_by_key(|(_, &c)| c).map(|(i, _)| i).unwrap_or(0);
        anomaly_texts.push(format!(
            "Resonance mod-3 clustering: [{}, {}, {}] → dominant mod {}",
            mod3_counts[0], mod3_counts[1], mod3_counts[2], dominant_mod
        ));
    }

    if !gaps.is_empty() {
        anomaly_texts.push(format!(
            "Found {} norm multiplicativity gaps (product representable but not found in lattice scan)",
            gaps.len()
        ));
    }

    // Primary metric: ratio of resonances to total representable norms
    let resonance_ratio = resonances.len() as f64 / representable_norms.len().max(1) as f64;
    let gap_ratio = gaps.len() as f64 / (representable_norms.len().pow(2) as f64).max(1.0);

    secondary.push(("representable_norms".into(), representable_norms.len() as f64));
    secondary.push(("resonance_count".into(), resonances.len() as f64));
    secondary.push(("resonance_ratio".into(), resonance_ratio));
    secondary.push(("gap_count".into(), gaps.len() as f64));
    secondary.push(("gap_ratio".into(), gap_ratio));
    secondary.push(("mean_factorizations".into(), mean_count));
    secondary.push(("max_resonance_z".into(), resonances.iter().map(|r| r.z_score.abs()).fold(0.0f64, f64::max)));

    let signature = format!("norm_mult_{}", params.max_norm);
    state.explored_regions.push(ExploredRegion { experiment_type: format!("{:?}", "novel"), params: serde_json::json!({}), result: 0.0, surprise: 0.0 });
    state.known_patterns.push(Pattern { experiment_type: "NormMultiplicative".into(), parameter_signature: signature.clone(), result_range: (resonance_ratio, resonance_ratio), observation_count: 1 });

    Observations {
        primary: resonance_ratio,
        secondary,
        success: resonances.is_empty(), // no resonances = multiplicativity is "clean"
        convergence_steps: representable_norms.len() as u32,
        anomalies: anomaly_texts,
    }
}
