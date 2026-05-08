//! Experiment: Eisenstein disk packing efficiency.
//!
//! How efficiently do disks of Eisenstein integers tile the lattice?
//! What packing densities are achievable? Where are the gaps?
//!
//! This connects to: sphere packing in R² (hexagonal is optimal!),
//! but what about DISCRETE packing with Eisenstein constraints?

use crate::types::*;
use crate::types::{ExploredRegion, Pattern};
use crate::surprise::eis_norm;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiskPackingParams {
    /// Maximum lattice radius to scan
    pub lattice_radius: i32,
    /// Disk radius (Eisenstein norm) for each packing center
    pub disk_radius: i32,
    /// Packing strategy
    pub strategy: PackingStrategy,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum PackingStrategy {
    /// Place disks greedily at lowest-norm uncovered site
    Greedy,
    /// Place disks at Eisenstein prime positions
    AtPrimes,
    /// Place at centers of norm rings (6, 12, 18, ...)
    AtNormRings,
    /// Random placement
    Random,
    /// Place at 6-fold symmetric positions
    Symmetric,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PackingResult {
    pub disks_placed: usize,
    pub coverage: f64,
    pub overlap_count: usize,
    /// Packing density = covered sites / total sites
    pub density: f64,
    /// Gaps: uncovered sites that are NOT near any disk boundary
    pub interior_gaps: usize,
    /// Hex efficiency: how close to optimal hex packing (π/(2√3) ≈ 0.9069)
    pub hex_efficiency: f64,
}

pub fn run(params: &DiskPackingParams, state: &mut EngineState) -> Observations {
    let r2 = (params.lattice_radius as i64).pow(2);
    let disk_r2 = (params.disk_radius as i64).pow(2);

    // Build lattice
    let mut sites: std::collections::HashSet<(i32, i32)> = std::collections::HashSet::new();
    for a in -params.lattice_radius..=params.lattice_radius {
        for b in -params.lattice_radius..=params.lattice_radius {
            if eis_norm(a, b) <= r2 {
                sites.insert((a, b));
            }
        }
    }
    let total_sites = sites.len();
    if total_sites == 0 {
        return Observations {
            primary: 0.0, secondary: vec![], success: false,
            convergence_steps: 0, anomalies: vec!["Empty lattice".into()],
        };
    }

    // Place disks according to strategy
    let mut covered: std::collections::HashSet<(i32, i32)> = std::collections::HashSet::new();
    let mut centers: Vec<(i32, i32)> = Vec::new();
    let mut uncovered_sites: Vec<(i32, i32)> = sites.iter().copied().collect();

    match params.strategy {
        PackingStrategy::Greedy => {
            uncovered_sites.sort_by_key(|&(a, b)| eis_norm(a, b));
            for &(a, b) in &uncovered_sites {
                if !covered.contains(&(a, b)) {
                    centers.push((a, b));
                    for da in -params.disk_radius..=params.disk_radius {
                        for db in -params.disk_radius..=params.disk_radius {
                            if eis_norm(a + da, b + db) <= disk_r2 {
                                let site = (a + da, b + db);
                                if sites.contains(&site) {
                                    covered.insert(site);
                                }
                            }
                        }
                    }
                }
            }
        }
        PackingStrategy::AtPrimes => {
            // Eisenstein primes: norm is prime in ℤ, or prime in ℤ[ω]
            for &(a, b) in &uncovered_sites {
                let n = eis_norm(a, b);
                if is_eisenstein_prime(n) {
                    centers.push((a, b));
                    for da in -params.disk_radius..=params.disk_radius {
                        for db in -params.disk_radius..=params.disk_radius {
                            if eis_norm(a + da, b + db) <= disk_r2 {
                                let site = (a + da, b + db);
                                if sites.contains(&site) {
                                    covered.insert(site);
                                }
                            }
                        }
                    }
                }
            }
        }
        PackingStrategy::AtNormRings => {
            // Place at norm ring centers (6, 12, 18, ...)
            for ring in 1..=params.lattice_radius {
                let ring_r2 = (ring as i64).pow(2);
                for &(a, b) in &uncovered_sites {
                    if eis_norm(a, b) == ring_r2 && !covered.contains(&(a, b)) {
                        centers.push((a, b));
                        for da in -params.disk_radius..=params.disk_radius {
                            for db in -params.disk_radius..=params.disk_radius {
                                if eis_norm(a + da, b + db) <= disk_r2 {
                                    let site = (a + da, b + db);
                                    if sites.contains(&site) {
                                        covered.insert(site);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        PackingStrategy::Symmetric => {
            // Place at 6-fold symmetric positions
            let hex_dirs = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, -1), (-1, 1)];
            for ring in 1..=(params.lattice_radius / params.disk_radius.max(1)) {
                for &(da, db) in &hex_dirs {
                    let (ca, cb) = (da * ring * params.disk_radius, db * ring * params.disk_radius);
                    if sites.contains(&(ca, cb)) {
                        centers.push((ca, cb));
                        for dda in -params.disk_radius..=params.disk_radius {
                            for ddb in -params.disk_radius..=params.disk_radius {
                                if eis_norm(ca + dda, cb + ddb) <= disk_r2 {
                                    let site = (ca + dda, cb + ddb);
                                    if sites.contains(&site) {
                                        covered.insert(site);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        PackingStrategy::Random => {
            let mut rng = rand::thread_rng();
            use rand::Rng;
            for _ in 0..(total_sites / (params.disk_radius as usize).max(1).pow(2).max(1)) {
                let idx = rng.gen_range(0..uncovered_sites.len());
                let (a, b) = uncovered_sites[idx];
                centers.push((a, b));
                for da in -params.disk_radius..=params.disk_radius {
                    for db in -params.disk_radius..=params.disk_radius {
                        if eis_norm(a + da, b + db) <= disk_r2 {
                            let site = (a + da, b + db);
                            if sites.contains(&site) {
                                covered.insert(site);
                            }
                        }
                    }
                }
            }
        }
    }

    // Compute overlaps
    let mut overlap_count = 0;
    let mut site_coverage: std::collections::HashMap<(i32, i32), usize> = std::collections::HashMap::new();
    for &(ca, cb) in &centers {
        for da in -params.disk_radius..=params.disk_radius {
            for db in -params.disk_radius..=params.disk_radius {
                if eis_norm(ca + da, cb + db) <= disk_r2 {
                    let site = (ca + da, cb + db);
                    if sites.contains(&site) {
                        *site_coverage.entry(site).or_insert(0) += 1;
                    }
                }
            }
        }
    }
    for &count in site_coverage.values() {
        if count > 1 { overlap_count += count - 1; }
    }

    // Interior gaps: uncovered sites not at the boundary
    let interior_gaps = sites.iter()
        .filter(|&&(a, b)| {
            !covered.contains(&(a, b)) &&
            eis_norm(a, b) < r2 - (params.disk_radius as i64 * 2).pow(2)
        })
        .count();

    let coverage = covered.len() as f64 / total_sites as f64;
    let optimal_packing = std::f64::consts::PI / (2.0 * 3.0f64.sqrt()); // ≈ 0.9069
    let hex_efficiency = coverage / optimal_packing;

    let mut anomalies = Vec::new();

    if hex_efficiency > 1.0 {
        anomalies.push(format!(
            "EXCEEDS hex packing limit ({:.3} > {:.3}) — discrete lattice allows tighter packing!",
            hex_efficiency, 1.0
        ));
    }

    if interior_gaps > 0 {
        anomalies.push(format!(
            "Interior gaps: {} uncovered sites deep inside lattice (packing has holes)",
            interior_gaps
        ));
    }

    if overlap_count > centers.len() / 2 {
        anomalies.push(format!(
            "High overlap: {} overlaps for {} disks ({:.1}%)",
            overlap_count, centers.len(),
            overlap_count as f64 / centers.len().max(1) as f64 * 100.0
        ));
    }

    let mut secondary = Vec::new();
    secondary.push(("disks_placed".into(), centers.len() as f64));
    secondary.push(("coverage".into(), coverage));
    secondary.push(("overlap_count".into(), overlap_count as f64));
    secondary.push(("interior_gaps".into(), interior_gaps as f64));
    secondary.push(("hex_efficiency".into(), hex_efficiency));
    secondary.push(("density".into(), coverage));

    let signature = format!("packing_r{}_d{}_{:?}", params.lattice_radius,
        params.disk_radius, params.strategy);
    state.explored_regions.push(ExploredRegion { experiment_type: format!("{:?}", "novel"), params: serde_json::json!({}), result: 0.0, surprise: 0.0 });
    state.known_patterns.push(Pattern { experiment_type: "DiskPacking".into(), parameter_signature: signature.clone(), result_range: (coverage, coverage), observation_count: 1 });

    Observations {
        primary: coverage,
        secondary,
        success: interior_gaps == 0 && coverage > 0.8,
        convergence_steps: centers.len() as u32,
        anomalies,
    }
}

/// Simple primality test for small values
fn is_eisenstein_prime(n: i64) -> bool {
    if n < 2 { return false; }
    if n == 2 || n == 3 || n == 5 { return true; }
    if n % 2 == 0 || n % 3 == 0 { return false; }
    let mut i = 5i64;
    while i * i <= n {
        if n % i == 0 || n % (i + 2) == 0 { return false; }
        i += 6;
    }
    true
}
