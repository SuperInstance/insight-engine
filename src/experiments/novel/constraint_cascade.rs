//! Experiment: Constraint cascade / domino effect.
//!
//! When one constraint is violated, how many others fall?
//! This measures the FRAGILITY of constraint systems.
//!
//! KEY QUESTION: Are Eisenstein constraint systems MORE fragile
//! or LESS fragile than random ones? Does hex geometry make
//! cascades propagate differently?

use crate::types::*;
use crate::types::{ExploredRegion, Pattern};
use crate::sbm::IsingModel;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConstraintCascadeParams {
    pub n_variables: usize,
    pub n_constraints: usize,
    /// How many constraints to break initially
    pub initial_breaks: usize,
    /// Propagation rule
    pub propagation: PropagationRule,
    /// Maximum cascade steps
    pub max_cascade_steps: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum PropagationRule {
    /// Break if >50% of neighbors are broken
    MajorityVote,
    /// Break if ANY neighbor is broken (domino)
    AnyNeighbor,
    /// Break probabilistically proportional to broken neighbors
    Probabilistic(f64),
    /// Eisenstein-weighted: closer constraints in norm space have more influence
    EisensteinWeighted,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CascadeResult {
    pub total_broken: usize,
    pub cascade_steps: u32,
    pub survived: bool,
    /// Avalanche sizes at each step
    pub avalanche_sizes: Vec<usize>,
}

pub fn run(params: &ConstraintCascadeParams, state: &mut EngineState) -> Observations {
    let n = params.n_variables;
    let mut anomalies = Vec::new();
    let mut secondary = Vec::new();

    // Build random constraint graph
    let mut broken = vec![false; n];
    let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); n];

    for _ in 0..params.n_constraints {
        let i = rand::random::<usize>() % n;
        let j = rand::random::<usize>() % n;
        if i != j && !adjacency[i].contains(&j) {
            adjacency[i].push(j);
            adjacency[j].push(i);
        }
    }

    // Break initial constraints
    for _ in 0..params.initial_breaks.min(n) {
        let idx = rand::random::<usize>() % n;
        broken[idx] = true;
    }

    let mut avalanche_sizes: Vec<usize> = Vec::new();
    let mut total_broken = broken.iter().filter(|&&b| b).count();
    let mut step = 0u32;

    for _ in 0..params.max_cascade_steps {
        let mut new_breaks = Vec::new();

        for i in 0..n {
            if broken[i] { continue; }

            let neighbors = &adjacency[i];
            if neighbors.is_empty() { continue; }

            let broken_neighbors = neighbors.iter().filter(|&&j| broken[j]).count();
            let ratio = broken_neighbors as f64 / neighbors.len() as f64;

            let should_break = match params.propagation {
                PropagationRule::MajorityVote => ratio > 0.5,
                PropagationRule::AnyNeighbor => broken_neighbors > 0,
                PropagationRule::Probabilistic(threshold) => {
                    rand::random::<f64>() < ratio * threshold
                }
                PropagationRule::EisensteinWeighted => {
                    // Weight by inverse Eisenstein norm of difference
                    let weight_sum: f64 = neighbors.iter().map(|&j| {
                        let norm = crate::surprise::eis_norm(i as i32, j as i32).max(1) as f64;
                        if broken[j] { 1.0 / norm } else { 0.0 }
                    }).sum();
                    weight_sum > 0.3 // Threshold
                }
            };

            if should_break {
                new_breaks.push(i);
            }
        }

        if new_breaks.is_empty() {
            break;
        }

        for idx in &new_breaks {
            broken[*idx] = true;
        }
        total_broken += new_breaks.len();
        avalanche_sizes.push(new_breaks.len());
        step += 1;
    }

    let cascade_fraction = total_broken as f64 / n as f64;
    let survived = cascade_fraction < 0.5;

    // Anomaly: look for power-law avalanche sizes
    if avalanche_sizes.len() > 3 {
        let sizes: Vec<f64> = avalanche_sizes.iter().map(|&s| s as f64).collect();
        let mean_size = sizes.iter().sum::<f64>() / sizes.len() as f64;
        let max_size = sizes.iter().cloned().fold(0.0f64, f64::max);

        if max_size > mean_size * 5.0 {
            anomalies.push(format!(
                "Avalanche SIZE HETEROGENEITY: max={:.0}, mean={:.1} — possible criticality",
                max_size, mean_size
            ));
        }

        // Check for SOC (self-organized criticality) signature
        let large_avalanches = sizes.iter().filter(|&&s| s > mean_size * 2.0).count();
        if large_avalanches > 0 && large_avalanches < sizes.len() / 3 {
            anomalies.push(format!(
                "SOC signature: {}/{} large avalanches amid small ones",
                large_avalanches, sizes.len()
            ));
        }
    }

    if cascade_fraction > 0.9 {
        anomalies.push(format!(
            "TOTAL COLLAPSE: {:.1}% broken — system is critically fragile",
            cascade_fraction * 100.0
        ));
    } else if cascade_fraction < 0.2 && params.initial_breaks > 1 {
        anomalies.push(format!(
            "RESILIENT: only {:.1}% broken after {} initial breaks — constraint system absorbs shocks",
            cascade_fraction * 100.0, params.initial_breaks
        ));
    }

    secondary.push(("cascade_fraction".into(), cascade_fraction));
    secondary.push(("total_broken".into(), total_broken as f64));
    secondary.push(("cascade_steps".into(), step as f64));
    secondary.push(("largest_avalanche".into(), avalanche_sizes.iter().copied().max().unwrap_or(0) as f64));
    secondary.push(("avalanche_count".into(), avalanche_sizes.len() as f64));

    if !avalanche_sizes.is_empty() {
        let mean_av = avalanche_sizes.iter().sum::<usize>() as f64 / avalanche_sizes.len() as f64;
        secondary.push(("mean_avalanche_size".into(), mean_av));
    }

    let signature = format!("cascade_n{}_c{}_b{}_{:?}",
        n, params.n_constraints, params.initial_breaks, params.propagation);
    state.explored_regions.push(ExploredRegion { experiment_type: format!("{:?}", "novel"), params: serde_json::json!({}), result: 0.0, surprise: 0.0 });
    state.known_patterns.push(Pattern { experiment_type: "SymmetryBreaking".into(), parameter_signature: signature.clone(), result_range: (cascade_fraction, cascade_fraction), observation_count: 1 });

    Observations {
        primary: cascade_fraction,
        secondary,
        success: survived,
        convergence_steps: step,
        anomalies,
    }
}
