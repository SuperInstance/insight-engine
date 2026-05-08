//! Core types for the insight engine.

use serde::{Serialize, Deserialize};

/// An insight extracted from an experiment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insight {
    pub id: u64,
    pub iteration: u64,
    pub experiment_type: String,
    pub parameters: serde_json::Value,
    pub observations: Observations,
    pub insight_text: String,
    /// How surprising was this result? (0.0 = expected, 1.0 = wtf)
    pub surprise: f64,
    /// How novel? (0.0 = seen before, 1.0 = brand new)
    pub novelty: f64,
    /// Quality = surprise * novelty
    pub quality: f64,
    /// Which insight spawned this experiment?
    pub parent_id: Option<u64>,
    /// Generation depth (0 = seed, 1 = bred from seed, etc.)
    pub generation: u32,
}

/// Raw observations from an experiment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observations {
    /// Primary metric (varies by experiment type)
    pub primary: f64,
    /// Secondary metrics
    pub secondary: Vec<(String, f64)>,
    /// Whether the experiment "worked" (solver converged, constraint satisfied, etc.)
    pub success: bool,
    /// How many iterations/steps to converge
    pub convergence_steps: u32,
    /// Anomaly flags
    pub anomalies: Vec<String>,
}

/// An experiment ready to run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experiment {
    pub id: u64,
    pub experiment_type: String,
    pub parameters: serde_json::Value,
    pub parent_insight_id: Option<u64>,
    pub generation: u32,
}

/// The experiment types — each probes a different intersection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExperimentType {
    /// Eisenstein norm distribution analysis
    EisensteinNormSweep,
    /// Constraint density vs SBM convergence
    SbmConvergence,
    /// Hex disk boundary precision
    HexDiskBoundary,
    /// Ternary weight distribution in solved CSPs
    TernaryDistribution,
    /// Eisenstein → Ising energy mapping
    EisIsingMapping,
    /// Parallel AC-3 convergence rate
    Ac3ConvergenceRate,
    /// FLUX VM batch constraint throughput
    FluxBatchThroughput,
    /// Novel combination (mutator-generated)
    NovelCombination,
}

impl std::fmt::Display for ExperimentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// The engine state — persists across iterations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineState {
    pub iteration: u64,
    pub insights: Vec<Insight>,
    pub known_patterns: Vec<Pattern>,
    /// Track what parameter regions have been explored
    pub explored_regions: Vec<ExploredRegion>,
    /// Best insight so far
    pub best_insight: Option<Insight>,
    /// Total experiments run
    pub total_experiments: u64,
    /// Total insights promoted
    pub total_insights: u64,
}

/// A known pattern — used to calculate novelty.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub experiment_type: String,
    pub parameter_signature: String,
    pub result_range: (f64, f64),
    pub observation_count: u32,
}

/// A region of parameter space that's been explored.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExploredRegion {
    pub experiment_type: String,
    pub params: serde_json::Value,
    pub result: f64,
    pub surprise: f64,
}

impl EngineState {
    pub fn new() -> Self {
        EngineState {
            iteration: 0,
            insights: Vec::new(),
            known_patterns: Vec::new(),
            explored_regions: Vec::new(),
            best_insight: None,
            total_experiments: 0,
            total_insights: 0,
        }
    }

    /// Calculate novelty of a result based on how different it is from known patterns.
    pub fn calculate_novelty(&self, exp_type: &str, result: f64) -> f64 {
        let similar: Vec<&Pattern> = self.known_patterns.iter()
            .filter(|p| p.experiment_type.as_str() == exp_type)
            .collect();

        if similar.is_empty() {
            return 1.0; // Brand new
        }

        // How far is this result from the mean of similar experiments?
        let mean: f64 = similar.iter().map(|p| (p.result_range.0 + p.result_range.1) / 2.0).sum::<f64>() / similar.len() as f64;
        let std_dev: f64 = {
            let variance: f64 = similar.iter()
                .map(|p| {
                    let mid = (p.result_range.0 + p.result_range.1) / 2.0;
                    (mid - mean).powi(2)
                })
                .sum::<f64>() / similar.len() as f64;
            variance.sqrt().max(0.001)
        };

        let z_score = ((result - mean) / std_dev).abs();
        // Novelty from z-score: 0 deviations = 0 novelty, 3+ deviations = 1.0
        (z_score / 3.0).min(1.0)
    }

    /// Record an explored region
    pub fn record_region(&mut self, exp_type: ExperimentType, params: serde_json::Value, result: f64, surprise: f64) {
        self.explored_regions.push(ExploredRegion {
            experiment_type: exp_type.to_string(),
            params,
            result,
            surprise,
        });
    }

    /// Update known patterns with a new observation
    pub fn update_patterns(&mut self, exp_type: ExperimentType, sig: String, result: f64) {
        if let Some(pattern) = self.known_patterns.iter_mut().find(|p| p.parameter_signature == sig) {
            // Extend the range
            pattern.result_range.0 = pattern.result_range.0.min(result);
            pattern.result_range.1 = pattern.result_range.1.max(result);
            pattern.observation_count += 1;
        } else {
            self.known_patterns.push(Pattern {
                experiment_type: exp_type.to_string(),
                parameter_signature: sig,
                result_range: (result, result),
                observation_count: 1,
            });
        }
    }
}
