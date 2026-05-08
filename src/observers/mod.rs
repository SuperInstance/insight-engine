//! Observer: extracts insights from experiment results.
//!
//! An insight is a surprise × novelty product above a threshold.
//! The observer detects:
//! - Anomalous measurements
//! - Unexpected patterns
//! - Phase transitions
//! - Cross-domain connections

use crate::types::*;

/// Extract an insight from experiment results.
pub fn extract(
    experiment: &Experiment,
    observations: &Observations,
    state: &EngineState,
) -> Insight {
    // Calculate surprise from anomalies
    let anomaly_surprise = observations.anomalies.len() as f64 / 5.0; // 5 anomalies = max surprise
    let convergence_surprise = if observations.success && observations.convergence_steps < 100 {
        0.3 // surprisingly fast convergence
    } else if !observations.success {
        0.1 // failure is not surprising
    } else {
        0.0
    };

    let surprise = (anomaly_surprise + convergence_surprise).min(1.0);
    let novelty = state.calculate_novelty(&experiment.experiment_type, observations.primary);
    let quality = surprise * novelty;

    // Generate insight text
    let insight_text = generate_insight_text(experiment, observations, surprise, novelty);

    Insight {
        id: state.total_insights + 1,
        iteration: state.iteration,
        experiment_type: format!("{:?}", experiment.experiment_type),
        parameters: experiment.parameters.clone(),
        observations: observations.clone(),
        insight_text,
        surprise,
        novelty,
        quality,
        parent_id: experiment.parent_insight_id,
        generation: experiment.generation,
    }
}

fn generate_insight_text(
    experiment: &Experiment,
    obs: &Observations,
    surprise: f64,
    novelty: f64,
) -> String {
    let mut parts = Vec::new();

    // What happened?
    if obs.success {
        parts.push(format!("{:?} converged in {} steps", experiment.experiment_type, obs.convergence_steps));
    } else {
        parts.push(format!("{:?} did NOT converge after {} steps", experiment.experiment_type, obs.convergence_steps));
    }

    // Key metrics
    for (name, value) in &obs.secondary {
        if value.abs() > 0.001 {
            parts.push(format!("{}={:.4}", name, value));
        }
    }

    // Anomalies are the interesting bits
    if !obs.anomalies.is_empty() {
        parts.push("ANOMALIES:".into());
        for a in &obs.anomalies {
            parts.push(format!("  → {}", a));
        }
    }

    // Surprise/novelty summary
    if surprise > 0.7 {
        parts.push(format!("⚠️ HIGH SURPRISE ({:.2})", surprise));
    }
    if novelty > 0.7 {
        parts.push(format!("✨ HIGH NOVELTY ({:.2})", novelty));
    }
    if surprise > 0.7 && novelty > 0.7 {
        parts.push("🔥 INSIGHT CANDIDATE — breed this".into());
    }

    parts.join("\n")
}
