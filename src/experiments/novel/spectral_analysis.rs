//! Experiment: Spectral analysis of Eisenstein constraint graphs.
//!
//! Treat constraint graphs as adjacency matrices and study their
//! eigenvalue spectra. The spectrum encodes:
//! - Algebraic connectivity (Fiedler value)
//! - Community structure (spectral gap)
//! - Constraint hardness (spectral radius)
//!
//! KEY QUESTION: Do Eisenstein constraint graphs have DISTINCTIVE
//! spectral signatures compared to random graphs?

use crate::types::*;
use crate::types::{ExploredRegion, Pattern};
use crate::surprise::z_score;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SpectralParams {
    pub n_nodes: usize,
    pub edge_density: f64,
    /// Also generate Eisenstein-structured graph for comparison
    pub hex_radius: i32,
    pub coupling_pattern: CouplingPattern,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum CouplingPattern {
    NearestNeighbor,
    NextNearestNeighbor,
    NormWeighted,
    Random,
}

/// Minimal power iteration for dominant eigenvalue.
fn power_iteration(adj: &[Vec<f64>], n: usize, iterations: usize) -> (f64, Vec<f64>) {
    let mut v: Vec<f64> = (0..n).map(|_| rand::random::<f64>() - 0.5).collect();
    let norm = v.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-10);
    for x in v.iter_mut() { *x /= norm; }

    let mut eigenvalue = 0.0f64;
    for _ in 0..iterations {
        let mut new_v = vec![0.0f64; n];
        for i in 0..n {
            for j in 0..n {
                new_v[i] += adj[i][j] * v[j];
            }
        }
        eigenvalue = new_v.iter().zip(v.iter()).map(|(a, b)| a * b).sum::<f64>();
        let norm = new_v.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-10);
        for x in new_v.iter_mut() { *x /= norm; }
        v = new_v;
    }
    (eigenvalue, v)
}

/// Estimate second eigenvalue using deflation.
fn second_eigenvalue(adj: &[Vec<f64>], n: usize, v1: &[f64], iterations: usize) -> f64 {
    // Deflate: A' = A - λ₁ v₁ v₁ᵀ
    let (e1, _) = power_iteration(adj, n, iterations);
    let mut deflated = vec![vec![0.0f64; n]; n];
    for i in 0..n {
        for j in 0..n {
            deflated[i][j] = adj[i][j] - e1 * v1[i] * v1[j];
        }
    }
    let (e2, _) = power_iteration(&deflated, n, iterations);
    e2.abs()
}

pub fn run(params: &SpectralParams, state: &mut EngineState) -> Observations {
    let n = params.n_nodes;
    let mut anomalies = Vec::new();
    let mut secondary = Vec::new();

    // Build Eisenstein-structured adjacency matrix
    let mut adj = vec![vec![0.0f64; n]; n];

    match params.coupling_pattern {
        CouplingPattern::NearestNeighbor => {
            // Connect nodes to nearest 6 neighbors (hex lattice)
            for i in 0..n {
                for j in (i+1)..n.min(i+7) {
                    let weight = 1.0 / (1.0 + (i as f64 - j as f64).abs());
                    adj[i][j] = weight;
                    adj[j][i] = weight;
                }
            }
        }
        CouplingPattern::NormWeighted => {
            // Weight edges by Eisenstein norm of (i,j) difference
            for i in 0..n {
                for j in (i+1)..n {
                    if rand::random::<f64>() < params.edge_density {
                        let ai = (i as i32) % (params.hex_radius + 1);
                        let bi = (i as i32) / (params.hex_radius + 1);
                        let aj = (j as i32) % (params.hex_radius + 1);
                        let bj = (j as i32) / (params.hex_radius + 1);
                        let norm_val = crate::surprise::eis_norm(ai - aj, bi - bj).max(1) as f64;
                        let weight = 1.0 / norm_val;
                        adj[i][j] = weight;
                        adj[j][i] = weight;
                    }
                }
            }
        }
        CouplingPattern::NextNearestNeighbor => {
            // Connect to 12 nearest (hex NNN lattice)
            for i in 0..n {
                for j in (i+1)..n.min(i+13) {
                    let weight = if j - i <= 6 { 1.0 } else { 0.5 };
                    adj[i][j] = weight;
                    adj[j][i] = weight;
                }
            }
        }
        CouplingPattern::Random => {
            for i in 0..n {
                for j in (i+1)..n {
                    if rand::random::<f64>() < params.edge_density {
                        let weight = rand::random::<f64>();
                        adj[i][j] = weight;
                        adj[j][i] = weight;
                    }
                }
            }
        }
    }

    // Compute spectral properties
    let (eigenvalue1, v1) = power_iteration(&adj, n, 100);
    let eigenvalue2 = second_eigenvalue(&adj, n, &v1, 100);

    // Spectral gap = λ₁ - λ₂ (large = good community structure)
    let spectral_gap = eigenvalue1 - eigenvalue2;

    // Algebraic connectivity proxy (small λ₂ = poorly connected)
    let algebraic_connectivity = eigenvalue2;

    // Edge count
    let edge_count: usize = adj.iter().flat_map(|row| row.iter()).filter(|&&w| w > 0.0).count() / 2;

    // Node degree distribution
    let degrees: Vec<f64> = (0..n).map(|i| adj[i].iter().sum()).collect();
    let mean_degree = degrees.iter().sum::<f64>() / n as f64;
    let degree_variance = degrees.iter()
        .map(|d| (d - mean_degree).powi(2))
        .sum::<f64>() / n as f64;

    // Fiedler value approximation
    // Spectral gap tells us about constraint propagation speed
    let propagation_speed = spectral_gap.max(0.0);

    secondary.push(("spectral_radius".into(), eigenvalue1));
    secondary.push(("second_eigenvalue".into(), eigenvalue2));
    secondary.push(("spectral_gap".into(), spectral_gap));
    secondary.push(("algebraic_connectivity".into(), algebraic_connectivity));
    secondary.push(("edge_count".into(), edge_count as f64));
    secondary.push(("mean_degree".into(), mean_degree));
    secondary.push(("degree_variance".into(), degree_variance));
    secondary.push(("propagation_speed".into(), propagation_speed));

    // Anomaly detection
    if spectral_gap > eigenvalue1 * 0.5 {
        anomalies.push(format!(
            "Large spectral gap ({:.3}) relative to radius ({:.3}) — strong community structure",
            spectral_gap, eigenvalue1
        ));
    }

    if spectral_gap < eigenvalue1 * 0.05 {
        anomalies.push(format!(
            "Tiny spectral gap ({:.4}) — constraint graph is nearly disconnected",
            spectral_gap
        ));
    }

    if degree_variance > mean_degree.powi(2) {
        anomalies.push(format!(
            "High degree heterogeneity (var={:.1}, mean={:.1}) — hub-and-spoke structure",
            degree_variance, mean_degree
        ));
    }

    // Compare random vs structured: the ratio of spectral gaps
    // tells us if Eisenstein structure creates harder or easier constraint systems
    let structural_ratio = if eigenvalue1 > 0.0 { spectral_gap / eigenvalue1 } else { 0.0 };

    if matches!(params.coupling_pattern, CouplingPattern::NearestNeighbor | CouplingPattern::NormWeighted) {
        if structural_ratio > 0.3 {
            anomalies.push(format!(
                "Eisenstein-structured graph has high spectral ratio ({:.3}) — constraints propagate FAST",
                structural_ratio
            ));
        }
    }

    let signature = format!("spectral_n{}_{}_{:?}", n, params.hex_radius, params.coupling_pattern);
    state.explored_regions.push(ExploredRegion { experiment_type: format!("{:?}", "novel"), params: serde_json::json!({}), result: 0.0, surprise: 0.0 });
    state.known_patterns.push(Pattern { experiment_type: "SpectralAnalysis".into(), parameter_signature: signature.clone(), result_range: (spectral_gap, spectral_gap), observation_count: 1 });

    Observations {
        primary: spectral_gap,
        secondary,
        success: spectral_gap > 0.0,
        convergence_steps: 100,
        anomalies,
    }
}
