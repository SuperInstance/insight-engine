//! Experiment: Symmetry breaking cascades in Eisenstein constraint systems.
//!
//! Start with a maximally symmetric configuration (all spins +1, centered).
//! Apply a single perturbation and watch HOW symmetry breaks.
//!
//! KEY INSIGHT: The PATTERN of symmetry breaking reveals the constraint
//! system's hidden structure. Do violations propagate along hex lattice
//! directions? Do they form fractal patterns? Do they respect Eisenstein
//! norm boundaries?

use crate::types::*;
use crate::types::{ExploredRegion, Pattern};
use crate::surprise::eis_norm;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SymmetryBreakingParams {
    pub lattice_radius: i32,
    /// Which site to perturb first (0 = center)
    pub perturbation_site: usize,
    /// How many cascade steps to simulate
    pub cascade_depth: u32,
    /// Temperature for Boltzmann acceptance
    pub temperature: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CascadeStep {
    pub step: u32,
    pub broken_count: usize,
    /// Which hex directions the violations spread along
    pub spread_directions: Vec<(i32, i32)>,
    /// Eisenstein norm of the frontier (furthest violation)
    pub frontier_norm: i64,
    /// Is the frontier fractal? (entropy of spread pattern)
    pub pattern_entropy: f64,
}

pub fn run(params: &SymmetryBreakingParams, state: &mut EngineState) -> Observations {
    // Build hex lattice sites
    let mut sites: Vec<(i32, i32)> = Vec::new();
    for a in -params.lattice_radius..=params.lattice_radius {
        for b in -params.lattice_radius..=params.lattice_radius {
            if eis_norm(a, b) <= (params.lattice_radius as i64).pow(2) {
                sites.push((a, b));
            }
        }
    }

    let n = sites.len();
    if n == 0 {
        return Observations {
            primary: 0.0, secondary: vec![], success: false,
            convergence_steps: 0, anomalies: vec!["Empty lattice".into()],
        };
    }

    // Start all spins +1
    let mut spins: Vec<i8> = vec![1; n];

    // Apply perturbation
    let perturb_idx = params.perturbation_site.min(n - 1);
    spins[perturb_idx] = -1;

    // Simulate cascade: at each step, find the site with highest "frustration"
    // (most neighbors with opposite spin) and flip it
    let hex_neighbors = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, -1), (-1, 1)];
    let mut cascade_history: Vec<CascadeStep> = Vec::new();
    let mut anomalies = Vec::new();
    let mut secondary = Vec::new();

    for step in 0..params.cascade_depth {
        // Compute frustration at each site
        let mut frustrations = vec![0.0f64; n];
        for (i, &(a, b)) in sites.iter().enumerate() {
            for &(da, db) in &hex_neighbors {
                let (na, nb) = (a + da, b + db);
                // Find neighbor index
                if let Some(j) = sites.iter().position(|&(sa, sb)| sa == na && sb == nb) {
                    if spins[i] != spins[j] {
                        frustrations[i] += 1.0;
                    }
                }
            }
        }

        // Find most frustrated site
        let (flip_idx, &max_frust) = frustrations.iter().enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap();

        if max_frust < 1.0 {
            // Cascade stopped — all sites satisfied
            cascade_history.push(CascadeStep {
                step,
                broken_count: spins.iter().filter(|&&s| s == -1).count(),
                spread_directions: vec![],
                frontier_norm: 0,
                pattern_entropy: 0.0,
            });
            break;
        }

        // Flip with Boltzmann probability
        let delta_e = -max_frust * 2.0; // Approximate energy change
        let prob = if params.temperature > 0.0 {
            (delta_e / params.temperature).exp().min(1.0)
        } else {
            1.0
        };

        if rand::random::<f64>() < prob {
            spins[flip_idx] *= -1;
        }

        // Analyze the pattern of violations
        let broken: Vec<(i32, i32)> = sites.iter().zip(spins.iter())
            .filter(|(_, &s)| s == -1)
            .map(|(&(a, b), _)| (a, b))
            .collect();

        // Find frontier (max norm of broken sites)
        let frontier_norm = broken.iter().map(|&(a, b)| eis_norm(a, b)).max().unwrap_or(0);

        // Compute spread directions (which hex directions have violations along them)
        let mut spread_dirs: Vec<(i32, i32)> = Vec::new();
        for &(a, b) in &broken {
            for &(da, db) in &hex_neighbors {
                let (na, nb) = (a + da, b + db);
                if broken.contains(&(na, nb)) {
                    if !spread_dirs.contains(&(da, db)) {
                        spread_dirs.push((da, db));
                    }
                }
            }
        }

        // Pattern entropy: how evenly spread is the violation pattern?
        let n_sectors = 6;
        let mut sector_counts = vec![0usize; n_sectors];
        for &(a, b) in &broken {
            let angle = (b as f64).atan2(a as f64);
            let sector = ((angle + std::f64::consts::PI) / (2.0 * std::f64::consts::PI) * n_sectors as f64) as usize % n_sectors;
            sector_counts[sector] += 1;
        }
        let total = broken.len().max(1) as f64;
        let entropy: f64 = -sector_counts.iter()
            .filter(|&&c| c > 0)
            .map(|&c| {
                let p = c as f64 / total;
                p * p.ln()
            })
            .sum::<f64>();
        let max_entropy = (n_sectors as f64).ln();
        let normalized_entropy = if max_entropy > 0.0 { entropy / max_entropy } else { 0.0 };

        cascade_history.push(CascadeStep {
            step,
            broken_count: broken.len(),
            spread_directions: spread_dirs.clone(),
            frontier_norm,
            pattern_entropy: normalized_entropy,
        });

        // Check for anomalous patterns
        if spread_dirs.len() == 1 {
            anomalies.push(format!(
                "Step {}: Cascade propagating along SINGLE direction {:?} (anisotropic!)",
                step, spread_dirs[0]
            ));
        }

        if normalized_entropy < 0.3 && broken.len() > 3 {
            anomalies.push(format!(
                "Step {}: Low entropy ({:.2}) — violations are sector-clustered",
                step, normalized_entropy
            ));
        }

        // Check for Eisenstein-norm-aligned propagation
        if frontier_norm > 0 {
            let broken_at_frontier = broken.iter().filter(|&&(a, b)| eis_norm(a, b) == frontier_norm).count();
            let expected_at_frontier = broken.len() as f64 * 0.2; // Rough estimate
            if broken_at_frontier as f64 > expected_at_frontier * 3.0 {
                anomalies.push(format!(
                    "Step {}: Violations clustering at norm boundary (norm={}, {} sites)",
                    step, frontier_norm, broken_at_frontier
                ));
            }
        }
    }

    // Primary metric: final cascade extent
    let final_broken = spins.iter().filter(|&&s| s == -1).count();
    let cascade_fraction = final_broken as f64 / n as f64;

    // Was the cascade contained or did it spread to the whole lattice?
    let contained = cascade_fraction < 0.5;

    secondary.push(("final_broken_fraction".into(), cascade_fraction));
    secondary.push(("cascade_depth_reached".into(), cascade_history.len() as f64));
    secondary.push(("frontier_max_norm".into(), cascade_history.iter().map(|s| s.frontier_norm as f64).fold(0.0, f64::max)));
    secondary.push(("mean_entropy".into(), cascade_history.iter().map(|s| s.pattern_entropy).sum::<f64>() / cascade_history.len().max(1) as f64));

    if !contained {
        anomalies.push(format!(
            "Cascade UNCONTAINED: {:.1}% of lattice broken",
            cascade_fraction * 100.0
        ));
    }

    let signature = format!("symmetry_r{}_p{}_c{}", params.lattice_radius,
        params.perturbation_site, params.cascade_depth);
    state.explored_regions.push(ExploredRegion { experiment_type: format!("{:?}", "novel"), params: serde_json::json!({}), result: 0.0, surprise: 0.0 });
    state.known_patterns.push(Pattern { experiment_type: "SymmetryBreaking".into(), parameter_signature: signature.clone(), result_range: (cascade_fraction, cascade_fraction), observation_count: 1 });

    Observations {
        primary: cascade_fraction,
        secondary,
        success: contained,
        convergence_steps: cascade_history.len() as u32,
        anomalies,
    }
}
