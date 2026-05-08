//! Experiment: Hex disk boundary precision analysis.
//!
//! Probes: At what radius does the Eisenstein disk boundary stop being
//! "clean"? Where do holes appear? What's the boundary fractal dimension?

use crate::types::*;
use crate::surprise::{eis_norm, eis_in_disk};
use crate::surprise;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HexDiskParams {
    pub radius: i32,
    /// Check both inner and outer rings?
    pub scan_rings: bool,
    /// Center
    pub center_a: i32,
    pub center_b: i32,
}

pub fn run(params: &HexDiskParams, state: &mut EngineState) -> Observations {
    
    let r_sq = (params.radius as i64).pow(2);

    // Count points inside disk at each ring
    let mut ring_counts: Vec<(i32, usize)> = Vec::new(); // (ring_radius, count_inside)
    let mut _boundary_points: Vec<(i32,i32)> = Vec::new(); // points on the edge
    let mut holes = Vec::new(); // gaps in the boundary

    for ring in 0..=params.radius * 2 {
        let mut count = 0;
        let r_sq_ring = (ring as i64).pow(2);

        // Sample points on this ring
        for a in -params.radius..=params.radius {
            for b in -params.radius..=params.radius {
                let pa = a + params.center_a;
                let pb = b + params.center_b;
                let norm = eis_norm(pa - params.center_a, pb - params.center_b);
                let ring_dist = (norm as f64).sqrt().round() as i32;
                if ring_dist == ring && eis_in_disk(pa, pb, params.center_a, params.center_b, r_sq) {
                    count += 1;
                }
            }
        }
        ring_counts.push((ring, count));
    }

    // Check for holes: expected ring count is 6*ring for ring > 0
    let mut ring_ratios = Vec::new();
    for &(ring, count) in &ring_counts {
        let expected = if ring == 0 { 1 } else { 6 * ring };
        let ratio = count as f64 / expected.max(1) as f64;
        ring_ratios.push(ratio);
        if ratio < 0.8 && ring > 0 {
            holes.push(format!("Hole at ring {}: {}/{} ({:.1}%)", ring, count, expected, ratio * 100.0));
        }
    }

    // Boundary coverage: how many ring points at radius R are inside the disk?
    let boundary_ring = &ring_counts.last();
    let boundary_coverage = ring_ratios.last().copied().unwrap_or(1.0);

    // Fractal dimension estimate: log(N)/log(r) across rings
    let total_inside: usize = ring_counts.iter().map(|&(_, c)| c).sum();
    let fractal_dim = if params.radius > 0 {
        (total_inside as f64).log2() / (params.radius as f64).log2()
    } else {
        0.0
    };

    // Surprise: expected fractal dim for 2D hex lattice is 2.0
    let surprise_val = surprise::z_score(fractal_dim, 2.0, 0.1).abs();

    let mut anomalies = Vec::new();
    let hole_count = holes.len();
    if !holes.is_empty() {
        anomalies.extend(holes);
    }
    let success = hole_count == 0;
    if fractal_dim < 1.8 || fractal_dim > 2.2 {
        anomalies.push(format!("Unexpected fractal dimension: {:.4}", fractal_dim));
    }
    if boundary_coverage < 0.5 {
        anomalies.push(format!("Low boundary coverage: {:.3}", boundary_coverage));
    }

    let result = Observations {
        primary: fractal_dim,
        secondary: vec![
            ("total_inside".into(), total_inside as f64),
            ("boundary_coverage".into(), boundary_coverage),
            ("hole_count".into(), hole_count as f64),
            ("fractal_dim".into(), fractal_dim),
        ],
        success: success,
        convergence_steps: ring_counts.len() as u32,
        anomalies,
    };

    let sig = format!("hex_disk_r{}_{}_{}", params.radius, params.center_a, params.center_b);
    state.update_patterns(ExperimentType::HexDiskBoundary, sig, fractal_dim);
    state.record_region(ExperimentType::HexDiskBoundary, serde_json::to_value(params).unwrap(), fractal_dim, surprise_val);

    result
}

impl Default for HexDiskParams {
    fn default() -> Self {
        HexDiskParams { radius: 10, scan_rings: true, center_a: 0, center_b: 0 }
    }
}
