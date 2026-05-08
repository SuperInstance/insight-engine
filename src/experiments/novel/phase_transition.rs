//! Experiment: Phase transition detection in Eisenstein constraint satisfaction.
//!
//! THE KEY QUESTION: At what density/constraint-count does an Eisenstein
//! constraint system become UNSATISFIABLE? Is the transition sharp (like
//! 3-SAT at α≈4.267) or smooth? Does the hex lattice geometry shift it?
//!
//! This is potentially PUBLISHABLE — nobody has studied phase transitions
//! specifically on Eisenstein/hex constraint systems.

use crate::types::*;
use crate::types::{ExploredRegion, Pattern};
use crate::surprise::z_score;
use crate::sbm::IsingModel;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PhaseTransitionParams {
    pub n_variables: usize,
    /// Sweep density from 0 to this value in steps
    pub max_density: f64,
    pub density_steps: usize,
    /// Number of random constraint systems to sample per density
    pub samples_per_density: usize,
    /// SBM solver parameters
    pub max_iterations: u32,
    pub kerr_coefficient: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PhaseTransitionPoint {
    pub density: f64,
    pub satisfaction_rate: f64,
    pub mean_convergence_steps: f64,
    /// Sharpness: how quickly does satisfaction_rate drop?
    pub sharpness: f64,
}

pub fn run(params: &PhaseTransitionParams, state: &mut EngineState) -> Observations {
    let mut transition_curve: Vec<PhaseTransitionPoint> = Vec::new();
    let mut anomalies = Vec::new();
    let mut secondary = Vec::new();

    let mut prev_sat_rate = 1.0;
    let mut max_sharpness = 0.0;
    let mut critical_density = 0.0f64;

    for step in 0..params.density_steps {
        let density = (step as f64 / params.density_steps as f64) * params.max_density;

        let mut sat_count = 0u32;
        let mut total_steps = 0u64;

        for _ in 0..params.samples_per_density {
            // Generate random constraint system at this density
            let n_binary = ((params.n_variables as f64 * density) * 0.6) as usize;
            let n_unary = ((params.n_variables as f64 * density) * 0.4) as usize;

            let mut model = IsingModel::new(params.n_variables);

            // Add random binary constraints
            for _ in 0..n_binary {
                let i = rand::random::<usize>() % params.n_variables;
                let j = rand::random::<usize>() % params.n_variables;
                if i != j {
                    let coupling = if rand::random::<bool>() { -1.0 } else { 1.0 };
                    model.add_coupling(i, j, coupling);
                }
            }

            // Add random local fields (unary constraints)
            for _ in 0..n_unary {
                let i = rand::random::<usize>() % params.n_variables;
                model.add_field(i, rand::random::<f64>() * 2.0 - 1.0);
            }

            // Solve
            let result = model.solve(params.max_iterations, params.kerr_coefficient);
            if result.satisfied {
                sat_count += 1;
            }
            total_steps += result.steps as u64;
        }

        let sat_rate = sat_count as f64 / params.samples_per_density as f64;
        let mean_steps = total_steps as f64 / params.samples_per_density as f64;

        // Sharpness = absolute change in satisfaction rate
        let sharpness = (sat_rate - prev_sat_rate).abs();
        if sharpness > max_sharpness {
            max_sharpness = sharpness;
            critical_density = density;
        }

        transition_curve.push(PhaseTransitionPoint {
            density,
            satisfaction_rate: sat_rate,
            mean_convergence_steps: mean_steps,
            sharpness,
        });

        prev_sat_rate = sat_rate;
    }

    // Detect anomalies in the transition curve
    let sat_rates: Vec<f64> = transition_curve.iter().map(|p| p.satisfaction_rate).collect();
    let mean_rate = sat_rates.iter().sum::<f64>() / sat_rates.len().max(1) as f64;

    // Look for non-monotonicity (satisfaction goes UP after going down)
    for i in 1..transition_curve.len() {
        if transition_curve[i].satisfaction_rate > transition_curve[i-1].satisfaction_rate + 0.05
            && transition_curve[i-1].satisfaction_rate < 0.5 {
            anomalies.push(format!(
                "Non-monotonic satisfaction at density {:.3}: {:.2} → {:.2} (went UP after declining)",
                transition_curve[i].density,
                transition_curve[i-1].satisfaction_rate,
                transition_curve[i].satisfaction_rate
            ));
        }
    }

    // Look for unusually sharp transitions
    if max_sharpness > 0.3 {
        anomalies.push(format!(
            "SHARP phase transition at density {:.3} (Δsat={:.3})",
            critical_density, max_sharpness
        ));
    }

    // Compare with 3-SAT critical point (α ≈ 4.267)
    // Our "density" maps to constraint/variable ratio
    let sat_alpha = 4.267;
    if critical_density > 0.0 {
        let deviation = (critical_density - sat_alpha / params.n_variables as f64).abs();
        secondary.push(("critical_density".into(), critical_density));
        secondary.push(("sat_alpha_deviation".into(), deviation));
        secondary.push(("max_sharpness".into(), max_sharpness));

        if deviation < 0.01 {
            anomalies.push(format!(
                "Critical density {:.3} matches 3-SAT threshold (α≈4.267/n={:.3})",
                critical_density, sat_alpha / params.n_variables as f64
            ));
        }
    }

    secondary.push(("critical_density".into(), critical_density));
    secondary.push(("max_sharpness".into(), max_sharpness));
    secondary.push(("transition_points".into(), transition_curve.len() as f64));

    let signature = format!("phase_n{}_d{}_s{}", params.n_variables,
        params.max_density, params.density_steps);
    state.explored_regions.push(ExploredRegion { experiment_type: format!("{:?}", "novel"), params: serde_json::json!({}), result: 0.0, surprise: 0.0 });
    state.known_patterns.push(Pattern { experiment_type: "PhaseTransition".into(), parameter_signature: signature.clone(), result_range: (critical_density, critical_density), observation_count: 1 });

    Observations {
        primary: critical_density,
        secondary,
        success: max_sharpness > 0.3, // Found a real phase transition
        convergence_steps: transition_curve.len() as u32,
        anomalies,
    }
}
