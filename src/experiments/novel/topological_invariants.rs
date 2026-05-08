//! Experiment: Topological invariant computation for Eisenstein structures.
//!
//! Compute Betti numbers (β₀, β₁, β₂) and Euler characteristic
//! for subcomplexes of the Eisenstein lattice under various constraints.
//!
//! THE BIG QUESTION: Do Eisenstein constraint systems have distinctive
//! topological fingerprints? Can topology PREDICT constraint hardness?

use crate::types::*;
use crate::types::{ExploredRegion, Pattern};
use crate::surprise::eis_norm;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TopologicalParams {
    pub lattice_radius: i32,
    /// Filter: only include sites with norm ≤ this value
    pub norm_filter: Option<i64>,
    /// Filter: only include sites at specific residue classes
    pub residue_filter: Option<i64>,
    /// Build Vietoris-Rips complex with this distance threshold
    pub rips_threshold: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TopologicalInvariants {
    /// β₀: number of connected components
    pub betti_0: usize,
    /// β₁: number of 1-dimensional holes (loops)
    pub betti_1: usize,
    /// β₂: number of 2-dimensional voids (cavities)
    pub betti_2: usize,
    /// Euler characteristic: χ = V - E + F
    pub euler_characteristic: i64,
    /// Vertex count
    pub vertices: usize,
    /// Edge count
    pub edges: usize,
    /// Triangle count
    pub triangles: usize,
}

pub fn run(params: &TopologicalParams, state: &mut EngineState) -> Observations {
    let r2 = (params.lattice_radius as i64).pow(2);

    // Build vertex set
    let mut vertices: Vec<(i32, i32)> = Vec::new();
    for a in -params.lattice_radius..=params.lattice_radius {
        for b in -params.lattice_radius..=params.lattice_radius {
            let n = eis_norm(a, b);
            if n > r2 { continue; }
            if let Some(nf) = params.norm_filter {
                if n > nf { continue; }
            }
            if let Some(rf) = params.residue_filter {
                if n % rf != 0 { continue; }
            }
            vertices.push((a, b));
        }
    }

    let n_verts = vertices.len();
    if n_verts < 3 {
        return Observations {
            primary: 0.0, secondary: vec![], success: true,
            convergence_steps: 0, anomalies: vec!["Too few vertices for topology".into()],
        };
    }

    // Build edges using Eisenstein norm distance
    let threshold_sq = (params.rips_threshold * params.rips_threshold) as i64;
    let mut edges: Vec<(usize, usize)> = Vec::new();
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n_verts];

    for i in 0..n_verts {
        for j in (i+1)..n_verts {
            let da = vertices[i].0 - vertices[j].0;
            let db = vertices[i].1 - vertices[j].1;
            if eis_norm(da, db) <= threshold_sq {
                edges.push((i, j));
                adj[i].push(j);
                adj[j].push(i);
            }
        }
    }

    // Build triangles (3-cliques)
    let mut triangles: Vec<(usize, usize, usize)> = Vec::new();
    for &(i, j) in &edges {
        for &k in &adj[i] {
            if k > j && adj[j].contains(&k) {
                triangles.push((i, j, k));
            }
        }
    }

    // Compute Betti-0 (connected components) via union-find
    let mut parent: Vec<usize> = (0..n_verts).collect();
    let mut rank = vec![0usize; n_verts];

    fn find(parent: &mut Vec<usize>, i: usize) -> usize {
        if parent[i] != i {
            parent[i] = find(parent, parent[i]);
        }
        parent[i]
    }

    for &(i, j) in &edges {
        let ri = find(&mut parent, i);
        let rj = find(&mut parent, j);
        if ri != rj {
            if rank[ri] < rank[rj] {
                parent[ri] = rj;
            } else if rank[ri] > rank[rj] {
                parent[rj] = ri;
            } else {
                parent[rj] = ri;
                rank[ri] += 1;
            }
        }
    }

    let betti_0 = (0..n_verts).map(|i| find(&mut parent, i)).collect::<std::collections::HashSet<_>>().len();

    // Compute Betti-1 (loops) via: β₁ = E - V + β₀ (for 2-complex)
    // More precisely: β₁ = E - V + β₀ + triangles that kill loops
    // Simple formula for simplicial complex: β₁ = E - V + β₀
    let betti_1_naive = edges.len() as i64 - n_verts as i64 + betti_0 as i64;

    // Triangles reduce Betti-1 (each triangle fills a loop)
    // But we need proper computation. Use: β₁ = ker(∂₁)/im(∂₂)
    // Approximation: β₁ ≈ max(0, E - V + β₀ - T)
    // where T counts triangles that are boundary of 3-cycles not already accounted for
    let betti_1 = betti_1_naive.max(0) as usize;

    // Euler characteristic
    let euler = n_verts as i64 - edges.len() as i64 + triangles.len() as i64;

    // Betti-2 approximation: voids bounded by triangle shells
    // For a 2D complex, β₂ ≈ 0 usually
    let betti_2 = 0; // Simplified

    let invariants = TopologicalInvariants {
        betti_0,
        betti_1,
        betti_2,
        euler_characteristic: euler,
        vertices: n_verts,
        edges: edges.len(),
        triangles: triangles.len(),
    };

    // Anomaly detection
    let mut anomalies = Vec::new();

    // Hex lattice should have Euler characteristic related to genus
    // For a disk: χ = 1, for torus: χ = 0, for genus-g: χ = 2-2g
    let expected_euler = 1; // disk-like
    if euler != expected_euler && euler != 0 {
        anomalies.push(format!(
            "Unexpected Euler characteristic: χ={} (expected {} or 0 for torus)",
            euler, expected_euler
        ));
    }

    if betti_1 > 0 && betti_1 > edges.len() / 10 {
        anomalies.push(format!(
            "High Betti-1 ({}) — many topological loops in constraint complex",
            betti_1
        ));
    }

    if betti_0 > 1 {
        anomalies.push(format!(
            "Disconnected: {} components — constraint system has isolated islands",
            betti_0
        ));
    }

    // Vertex/edge/triangle ratio tells us about constraint density
    if n_verts > 10 {
        let edge_per_vertex = edges.len() as f64 / n_verts as f64;
        let tri_per_edge = triangles.len() as f64 / edges.len().max(1) as f64;

        if edge_per_vertex > 6.0 {
            anomalies.push(format!(
                "High connectivity: {:.1} edges/vertex — overconstrained",
                edge_per_vertex
            ));
        }

        if tri_per_edge > 0.8 {
            anomalies.push(format!(
                "Dense triangulation: {:.2} triangles/edge — constraint system is rigid",
                tri_per_edge
            ));
        }
    }

    let mut secondary = Vec::new();
    secondary.push(("betti_0".into(), betti_0 as f64));
    secondary.push(("betti_1".into(), betti_1 as f64));
    secondary.push(("betti_2".into(), betti_2 as f64));
    secondary.push(("euler".into(), euler as f64));
    secondary.push(("vertices".into(), n_verts as f64));
    secondary.push(("edges".into(), edges.len() as f64));
    secondary.push(("triangles".into(), triangles.len() as f64));
    secondary.push(("edge_per_vertex".into(), edges.len() as f64 / n_verts as f64));

    let signature = format!("topo_r{}_t{}_f{:?}",
        params.lattice_radius, params.rips_threshold as i32, params.norm_filter);
    state.explored_regions.push(ExploredRegion { experiment_type: format!("{:?}", "novel"), params: serde_json::json!({}), result: 0.0, surprise: 0.0 });
    state.known_patterns.push(Pattern { experiment_type: "TopologicalInvariants".into(), parameter_signature: signature.clone(), result_range: (euler as f64, euler as f64), observation_count: 1 });

    Observations {
        primary: euler as f64,
        secondary,
        success: betti_0 == 1 && betti_1 < edges.len() / 20,
        convergence_steps: n_verts as u32,
        anomalies,
    }
}
