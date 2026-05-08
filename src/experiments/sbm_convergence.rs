//! Experiment: Simulated Bifurcation convergence analysis.
//!
//! Probes: How does constraint density, variable count, and coupling
//! strength affect SBM convergence? Where are the phase transitions?

use crate::types::*;
use crate::sbm::{IsingModel, ConstraintGraph, BinaryConstraint};
use crate::eisenstein::{Constraint, Priority, Check};
use crate::surprise;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SbmParams {
    pub n_vars: usize,
    pub density: f64,
    pub coupling_strength: f64,
    pub max_iterations: usize,
    pub kerr_coefficient: f64,
}

pub fn run(params: &SbmParams, state: &mut EngineState) -> Observations {
    let graph = generate_random_graph(params.n_vars, params.density);
    let model = IsingModel::from_graph(&graph, params.coupling_strength, params.kerr_coefficient);

    let (spins, convergence_iters) = model.solve_with_tracking(params.max_iterations);

    // Evaluate solution quality
    let energy = model.energy(&spins);
    let violations = count_violations(&graph, &spins);
    let satisfaction_rate = 1.0 - (violations as f64 / graph.binary_constraints.len().max(1) as f64);

    // Phase transition detection
    let polarization = spins.iter().map(|&s| s as f64).sum::<f64>().abs() / spins.len() as f64;

    // Calculate surprise: energy should scale with N and coupling
    let expected_energy = -(params.n_vars as f64 * params.density * params.coupling_strength);
    let surprise_val = surprise::relative_error(energy, expected_energy).min(1.0);

    let mut anomalies = Vec::new();
    if violations == 0 && params.n_vars > 32 {
        anomalies.push("Zero violations at scale > 32".into());
    }
    if convergence_iters < 100 && params.n_vars > 64 {
        anomalies.push(format!("Fast convergence at scale {}: {} iters", params.n_vars, convergence_iters));
    }
    if polarization > 0.95 {
        anomalies.push(format!("Near-complete polarization: {:.3}", polarization));
    }

    let result = Observations {
        primary: satisfaction_rate,
        secondary: vec![
            ("energy".into(), energy),
            ("violations".into(), violations as f64),
            ("convergence_iters".into(), convergence_iters as f64),
            ("polarization".into(), polarization),
        ],
        success: violations == 0,
        convergence_steps: convergence_iters as u32,
        anomalies,
    };

    let sig = format!("sbm_n{}_d{:.2}_c{:.2}_k{:.2}", params.n_vars, params.density, params.coupling_strength, params.kerr_coefficient);
    state.update_patterns(ExperimentType::SbmConvergence, sig, satisfaction_rate);
    state.record_region(ExperimentType::SbmConvergence, serde_json::to_value(params).unwrap(), satisfaction_rate, surprise_val);

    result
}

fn generate_random_graph(n_vars: usize, density: f64) -> ConstraintGraph {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let mut constraints = Vec::new();
    for i in 0..n_vars {
        constraints.push(Constraint {
            name: format!("v{}", i),
            priority: Priority::Hard,
            checks: vec![Check::Range { start: -10.0, end: 10.0 }],
        });
    }

    // Add binary constraints based on density
    for i in 0..n_vars {
        for j in (i+1)..n_vars {
            if rng.gen::<f64>() < density {
                let check = if rng.gen::<f64>() < 0.5 {
                    Check::Equal(format!("v{}", j))
                } else {
                    Check::NotEqual(format!("v{}", j))
                };
                constraints[i].checks.push(check);
            }
        }
    }

    ConstraintGraph::build(&constraints).unwrap_or(ConstraintGraph {
        n_vars: 0,
        var_names: vec![],
        initial_domains: vec![],
        binary_constraints: vec![],
        adjacency: vec![],
    })
}

fn count_violations(graph: &ConstraintGraph, spins: &[i8]) -> usize {
    let mut violations = 0;
    for bc in &graph.binary_constraints {
        match bc {
            BinaryConstraint::Equal(i, j) => {
                if spins.get(*i).copied().unwrap_or(0) != spins.get(*j).copied().unwrap_or(0) {
                    violations += 1;
                }
            }
            BinaryConstraint::NotEqual(i, j) => {
                if spins.get(*i).copied().unwrap_or(0) == spins.get(*j).copied().unwrap_or(0) {
                    violations += 1;
                }
            }
            BinaryConstraint::Imply { from, to } => {
                let f = spins.get(*from).copied().unwrap_or(0);
                let t = spins.get(*to).copied().unwrap_or(0);
                if f != 0 && f != t { violations += 1; }
            }
        }
    }
    violations
}

impl Default for SbmParams {
    fn default() -> Self {
        SbmParams { n_vars: 32, density: 0.3, coupling_strength: 1.0, max_iterations: 2000, kerr_coefficient: 0.5 }
    }
}
