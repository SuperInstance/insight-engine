//! Hypothesis Generator — breeds new experiment TYPES from anomalies.
//!
//! This is the KEY INNOVATION. Not just parameter sweep, but
//! META-LEVEL discovery: anomalies in one experiment type suggest
//! entirely new experiment types.
//!
//! The generator maintains a "knowledge frontier" — things we've
//! discovered that we don't understand yet — and designs experiments
//! to probe those frontiers.

use crate::types::*;
use crate::experiments::novel::*;
use rand::Rng;

/// A knowledge frontier item: something observed but not understood.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FrontierItem {
    pub description: String,
    pub source_experiment: String,
    pub anomaly_text: String,
    pub follow_up_type: NovelExperimentType,
    pub priority: f64, // higher = more interesting
}

/// Novel experiment types discovered by the hypothesis generator.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum NovelExperimentType {
    NormMultiplicative,
    PhaseTransition,
    SpectralAnalysis,
    SymmetryBreaking,
    DiskPacking,
    ConstraintCascade,
    TopologicalInvariants,
    /// Meta: compare results across multiple experiment types
    CrossCorrelation,
    /// Meta: drill into a specific anomaly with focused parameters
    AnomalyDrilldown,
    /// Meta: scale up an interesting small-scale result
    ScaleUp,
}

/// The knowledge frontier: what we know we don't know.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct KnowledgeFrontier {
    pub items: Vec<FrontierItem>,
    /// Insights that have been followed up on
    pub explored: Vec<String>,
}

impl KnowledgeFrontier {
    pub fn add(&mut self, item: FrontierItem) {
        // Don't add duplicates
        if !self.items.iter().any(|i| i.description == item.description) {
            self.items.push(item);
            self.items.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap_or(std::cmp::Ordering::Equal));
        }
    }

    pub fn pop_highest_priority(&mut self) -> Option<FrontierItem> {
        if self.items.is_empty() { return None; }
        let item = self.items.remove(0);
        self.explored.push(item.description.clone());
        Some(item)
    }
}

/// Extract frontier items from an insight.
pub fn extract_frontier(insight: &Insight) -> Vec<FrontierItem> {
    let mut items = Vec::new();

    for anomaly in &insight.observations.anomalies {
        // Classify the anomaly and suggest follow-up experiment types
        let (follow_up, priority) = classify_anomaly(anomaly, insight);

        items.push(FrontierItem {
            description: format!("{} → {}", insight.experiment_type, anomaly),
            source_experiment: insight.experiment_type.clone(),
            anomaly_text: anomaly.clone(),
            follow_up_type: follow_up,
            priority: insight.quality * priority,
        });
    }

    // Also generate cross-correlation hypotheses from secondary metrics
    if insight.observations.secondary.len() >= 3 {
        // Look for unusual metric ratios
        let metrics: std::collections::HashMap<String, f64> = insight.observations.secondary.iter().cloned().collect();

        if let (Some(&coverage), Some(&gaps)) = (metrics.get("coverage"), metrics.get("interior_gaps")) {
            if coverage > 0.7 && gaps > 0.0 {
                items.push(FrontierItem {
                    description: format!("High coverage ({:.2}) with interior gaps ({}) — packing topology",
                        coverage, gaps as i64),
                    source_experiment: insight.experiment_type.clone(),
                    anomaly_text: format!("{} gaps at {:.0}% coverage", gaps as i64, coverage * 100.0),
                    follow_up_type: NovelExperimentType::TopologicalInvariants,
                    priority: 0.8,
                });
            }
        }

        if let (Some(&spectral_gap), Some(&sat_rate)) = (metrics.get("spectral_gap"), metrics.get("satisfaction_rate")) {
            if spectral_gap > 0.5 && sat_rate < 0.5 {
                items.push(FrontierItem {
                    description: format!("Large spectral gap ({:.3}) but low SAT ({:.2}) — contradiction?",
                        spectral_gap, sat_rate),
                    source_experiment: insight.experiment_type.clone(),
                    anomaly_text: "Structural contradiction: well-connected but unsatisfiable".into(),
                    follow_up_type: NovelExperimentType::PhaseTransition,
                    priority: 0.9,
                });
            }
        }
    }

    items
}

fn classify_anomaly(anomaly: &str, insight: &Insight) -> (NovelExperimentType, f64) {
    let lower = anomaly.to_lowercase();

    if lower.contains("resonance") || lower.contains("factor") || lower.contains("multiplicat") {
        (NovelExperimentType::NormMultiplicative, 0.9)
    } else if lower.contains("phase") || lower.contains("critical") || lower.contains("sharp") {
        (NovelExperimentType::PhaseTransition, 0.95)
    } else if lower.contains("spectral") || lower.contains("eigenvalue") || lower.contains("community") {
        (NovelExperimentType::SpectralAnalysis, 0.85)
    } else if lower.contains("cascade") || lower.contains("collapse") || lower.contains("fragile") {
        (NovelExperimentType::ConstraintCascade, 0.9)
    } else if lower.contains("symmetry") || lower.contains("anisotrop") || lower.contains("direction") {
        (NovelExperimentType::SymmetryBreaking, 0.85)
    } else if lower.contains("packing") || lower.contains("gap") || lower.contains("coverage") {
        (NovelExperimentType::DiskPacking, 0.8)
    } else if lower.contains("euler") || lower.contains("betti") || lower.contains("topolog") || lower.contains("loop") {
        (NovelExperimentType::TopologicalInvariants, 0.85)
    } else if lower.contains("fractal") || lower.contains("dimension") {
        (NovelExperimentType::ScaleUp, 0.7)
    } else if lower.contains("cluster") || lower.contains("mod-3") {
        (NovelExperimentType::CrossCorrelation, 0.75)
    } else {
        // Default: drill into the anomaly
        (NovelExperimentType::AnomalyDrilldown, 0.6)
    }
}

/// Generate experiment parameters from a frontier item.
pub fn design_experiment(frontier: &FrontierItem, rng: &mut impl Rng) -> (NovelExperimentType, serde_json::Value) {
    match frontier.follow_up_type {
        NovelExperimentType::NormMultiplicative => {
            let max_norm = rng.gen_range(50..500);
            (NovelExperimentType::NormMultiplicative, serde_json::json!({
                "max_norm": max_norm,
                "resonance_threshold": 1.5,
                "compare_with_integers": true,
            }))
        }
        NovelExperimentType::PhaseTransition => {
            let n_vars = [16, 32, 64, 128][rng.gen_range(0..4)];
            (NovelExperimentType::PhaseTransition, serde_json::json!({
                "n_variables": n_vars,
                "max_density": 1.0,
                "density_steps": 20,
                "samples_per_density": 10,
                "max_iterations": 1000,
                "kerr_coefficient": 0.5,
            }))
        }
        NovelExperimentType::SpectralAnalysis => {
            let pattern = ["NearestNeighbor", "NormWeighted", "NextNearestNeighbor", "Random"][rng.gen_range(0..4)];
            let n_nodes = [32, 64, 128][rng.gen_range(0..3)];
            (NovelExperimentType::SpectralAnalysis, serde_json::json!({
                "n_nodes": n_nodes,
                "edge_density": rng.gen_range(0.1..0.5),
                "hex_radius": rng.gen_range(3..10),
                "coupling_pattern": pattern,
            }))
        }
        NovelExperimentType::SymmetryBreaking => {
            (NovelExperimentType::SymmetryBreaking, serde_json::json!({
                "lattice_radius": rng.gen_range(5..20),
                "perturbation_site": rng.gen_range(0..10),
                "cascade_depth": rng.gen_range(20..100),
                "temperature": rng.gen_range(0.1..2.0),
            }))
        }
        NovelExperimentType::DiskPacking => {
            let strategy = ["Greedy", "AtPrimes", "AtNormRings", "Symmetric"][rng.gen_range(0..4)];
            (NovelExperimentType::DiskPacking, serde_json::json!({
                "lattice_radius": rng.gen_range(10..30),
                "disk_radius": rng.gen_range(2..8),
                "strategy": strategy,
            }))
        }
        NovelExperimentType::ConstraintCascade => {
            let propagation = ["MajorityVote", "AnyNeighbor", "Probabilistic", "EisensteinWeighted"][rng.gen_range(0..4)];
            let n_vars = [32, 64, 128, 256][rng.gen_range(0..4)];
            (NovelExperimentType::ConstraintCascade, serde_json::json!({
                "n_variables": n_vars,
                "n_constraints": rng.gen_range(20..200),
                "initial_breaks": rng.gen_range(1..10),
                "propagation": propagation,
                "max_cascade_steps": 50,
            }))
        }
        NovelExperimentType::TopologicalInvariants => {
            let norm_filter = if rng.gen_bool(0.5) { Some(rng.gen_range(5..50)) } else { None };
            let residue_filter = if rng.gen_bool(0.3) { Some(rng.gen_range(2..7)) } else { None };
            (NovelExperimentType::TopologicalInvariants, serde_json::json!({
                "lattice_radius": rng.gen_range(5..15),
                "norm_filter": norm_filter,
                "residue_filter": residue_filter,
                "rips_threshold": rng.gen_range(1.0..3.0),
            }))
        }
        NovelExperimentType::CrossCorrelation => {
            // Run a different type than the source
            (NovelExperimentType::SpectralAnalysis, serde_json::json!({
                "n_nodes": 64,
                "edge_density": 0.3,
                "hex_radius": 5,
                "coupling_pattern": "NormWeighted",
            }))
        }
        NovelExperimentType::AnomalyDrilldown => {
            // Focus on the same type but with targeted parameters
            (NovelExperimentType::SymmetryBreaking, serde_json::json!({
                "lattice_radius": rng.gen_range(3..10),
                "perturbation_site": 0, // center
                "cascade_depth": 200,
                "temperature": 0.01, // low temp = deterministic
            }))
        }
        NovelExperimentType::ScaleUp => {
            let n_vars = [256, 512, 1024][rng.gen_range(0..3)];
            (NovelExperimentType::PhaseTransition, serde_json::json!({
                "n_variables": n_vars,
                "max_density": 1.0,
                "density_steps": 30,
                "samples_per_density": 20,
                "max_iterations": 2000,
                "kerr_coefficient": 0.5,
            }))
        }
    }
}
