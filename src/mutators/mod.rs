//! Mutator: breeds new experiments from insights.
//!
//! Strategies:
//! 1. Parameter perturbation — vary one parameter of a high-quality insight
//! 2. Cross-pollination — combine parameters from two different insight types
//! 3. Anomaly drilling — take an anomaly and create a focused experiment
//! 4. Boundary exploration — push parameters to extremes
//! 5. Novel combination — random walk through parameter space

use crate::types::*;
use rand::Rng;

/// Breed a new experiment from the current engine state.
pub fn breed_next(state: &EngineState) -> Experiment {
    let mut rng = rand::thread_rng();

    // If we have high-quality insights, mutate them
    let top_insights: Vec<&Insight> = state.insights.iter()
        .filter(|i| i.quality > 0.5)
        .collect();

    if !top_insights.is_empty() && rng.gen::<f64>() < 0.7 {
        // Mutate a top insight
        let parent = top_insights[rng.gen_range(0..top_insights.len())];
        mutate_insight(parent, &mut rng)
    } else if top_insights.len() >= 2 && rng.gen::<f64>() < 0.3 {
        // Cross-pollinate two insights
        let i1 = top_insights[rng.gen_range(0..top_insights.len())];
        let i2 = top_insights[rng.gen_range(0..top_insights.len())];
        cross_pollinate(i1, i2, &mut rng)
    } else {
        // Novel exploration
        random_experiment(&mut rng, state)
    }
}

fn mutate_insight(insight: &Insight, rng: &mut impl Rng) -> Experiment {
    let params = insight.parameters.clone();
    let generation = insight.generation + 1;

    // Choose mutation strategy based on experiment type
    match insight.experiment_type.as_str() {
        "eis_norm_sweep" => {
            let mut p: serde_json::Value = params.clone();
            // Perturb radius
            if let Some(r) = p.get("radius").and_then(|v| v.as_i64()) {
                let delta = rng.gen_range(-5..=5);
                p["radius"] = serde_json::json!((r + delta).max(2));
            }
            // Shift center
            if rng.gen::<f64>() < 0.5 {
                let da = rng.gen_range(-3..=3);
                let db = rng.gen_range(-3..=3);
                if let Some(a) = p.get("center_a").and_then(|v| v.as_i64()) {
                    p["center_a"] = serde_json::json!(a + da);
                }
                if let Some(b) = p.get("center_b").and_then(|v| v.as_i64()) {
                    p["center_b"] = serde_json::json!(b + db);
                }
            }
            Experiment {
                id: 0,
                experiment_type: "EisensteinNormSweep".to_string(),
                parameters: p,
                parent_insight_id: Some(insight.id),
                generation,
            }
        }
        "sbm_convergence" => {
            let mut p: serde_json::Value = params.clone();
            // Scale up variables
            if let Some(n) = p.get("n_vars").and_then(|v| v.as_u64()) {
                let factor = if rng.gen::<f64>() < 0.3 { 2 } else { 1 };
                p["n_vars"] = serde_json::json!((n as usize * factor).min(512));
            }
            // Perturb density
            if let Some(d) = p.get("density").and_then(|v| v.as_f64()) {
                let delta = rng.gen_range(-0.1..0.1);
                p["density"] = serde_json::json!((d + delta).clamp(0.05, 0.95));
            }
            // Try different Kerr coefficients
            if rng.gen::<f64>() < 0.3 {
                p["kerr_coefficient"] = serde_json::json!(rng.gen_range(0.1..1.0));
            }
            Experiment {
                id: 0,
                experiment_type: "SbmConvergence".to_string(),
                parameters: p,
                parent_insight_id: Some(insight.id),
                generation,
            }
        }
        "hex_disk_boundary" => {
            let mut p: serde_json::Value = params.clone();
            if let Some(r) = p.get("radius").and_then(|v| v.as_i64()) {
                let delta = rng.gen_range(-3..=5);
                p["radius"] = serde_json::json!((r + delta).max(2).min(50));
            }
            Experiment {
                id: 0,
                experiment_type: "HexDiskBoundary".to_string(),
                parameters: p,
                parent_insight_id: Some(insight.id),
                generation,
            }
        }
        _ => random_experiment(rng, &EngineState::new())
    }
}

fn cross_pollinate(i1: &Insight, i2: &Insight, rng: &mut impl Rng) -> Experiment {
    // Take parameters from one, experiment type from another
    let exp_type = match i1.experiment_type.as_str() {
        "eis_norm_sweep" => ExperimentType::EisensteinNormSweep,
        "sbm_convergence" => ExperimentType::SbmConvergence,
        "hex_disk_boundary" => ExperimentType::HexDiskBoundary,
        _ => ExperimentType::NovelCombination,
    };

    // Mix parameters
    let mut params = i1.parameters.clone();
    if let (_obj1, Some(obj2)) = (i1.parameters.as_object(), i2.parameters.as_object()) {
        if let serde_json::Value::Object(ref mut map) = params {
            for (key, value) in obj2 {
                if rng.gen::<f64>() < 0.5 {
                    map.insert(key.clone(), value.clone());
                }
            }
        }
    }

    Experiment {
        id: 0,
        experiment_type: exp_type.to_string(),
        parameters: params,
        parent_insight_id: Some(i1.id),
        generation: i1.generation.max(i2.generation) + 1,
    }
}

fn random_experiment(rng: &mut impl Rng, state: &EngineState) -> Experiment {
    // Choose type — bias toward underexplored regions
    let types = [
        ExperimentType::EisensteinNormSweep,
        ExperimentType::SbmConvergence,
        ExperimentType::HexDiskBoundary,
        ExperimentType::NovelCombination,
    ];

    let exp_type = types[rng.gen_range(0..types.len())].clone();

    let params = match exp_type {
        ExperimentType::EisensteinNormSweep => serde_json::json!({
            "radius": rng.gen_range(5..50),
            "density": rng.gen_range(1..4),
            "center_a": rng.gen_range(-10..10),
            "center_b": rng.gen_range(-10..10),
        }),
        ExperimentType::SbmConvergence => {
            let n_vars_choices = [16, 32, 64, 128, 256];
            serde_json::json!({
                "n_vars": n_vars_choices[rng.gen_range(0..n_vars_choices.len())],
                "density": rng.gen_range(0.1..0.9),
                "coupling_strength": rng.gen_range(0.5..2.0),
                "max_iterations": 2000,
                "kerr_coefficient": rng.gen_range(0.1..1.0),
            })
        }
        ExperimentType::HexDiskBoundary => serde_json::json!({
            "radius": rng.gen_range(3..30),
            "scan_rings": true,
            "center_a": rng.gen_range(-5..5),
            "center_b": rng.gen_range(-5..5),
        }),
        ExperimentType::NovelCombination => {
            let sbm_choices = [16, 32, 64];
            serde_json::json!({
                "eis_radius": rng.gen_range(5..30),
                "sbm_vars": sbm_choices[rng.gen_range(0..sbm_choices.len())],
                "coupling": rng.gen_range(0.3..1.5),
                "disk_radius": rng.gen_range(3..15),
            })
        }
        _ => serde_json::json!({}),
    };

    Experiment {
        id: state.total_experiments,
        experiment_type: exp_type.to_string(),
        parameters: params,
        parent_insight_id: None,
        generation: 0,
    }
}
