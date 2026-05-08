//! Minimal SBM solver for experiments.

pub use crate::eisenstein::{ConstraintGraph, BinaryConstraint};

#[derive(Debug, Clone)]
pub struct IsingModel {
    pub n_vars: usize,
    couplings: Vec<(usize, usize, f64)>,
    fields: Vec<f64>,
    pub k: f64,
}

pub struct SolveResult {
    pub spins: Vec<i8>,
    pub steps: usize,
    pub satisfied: bool,
    pub energy: f64,
}

impl IsingModel {
    pub fn from_graph(graph: &ConstraintGraph, coupling: f64, k: f64) -> Self {
        let mut couplings = Vec::new();
        let fields = vec![0.0; graph.n_vars];

        for bc in &graph.binary_constraints {
            match bc {
                BinaryConstraint::Equal(i, j) => couplings.push((*i, *j, coupling)),
                BinaryConstraint::NotEqual(i, j) => couplings.push((*i, *j, -coupling)),
                BinaryConstraint::Imply { from, to } => couplings.push((*from, *to, coupling * 0.5)),
            }
        }

        IsingModel { n_vars: graph.n_vars, couplings, fields, k }
    }

    pub fn new(n_vars: usize) -> Self {
        IsingModel {
            n_vars,
            couplings: Vec::new(),
            fields: vec![0.0; n_vars],
            k: 0.5,
        }
    }

    pub fn add_coupling(&mut self, i: usize, j: usize, c: f64) {
        self.couplings.push((i, j, c));
    }

    pub fn add_field(&mut self, i: usize, h: f64) {
        if i < self.fields.len() {
            self.fields[i] = h;
        }
    }

    pub fn solve(&self, max_iterations: u32, _kerr: f64) -> SolveResult {
        let (spins, steps) = self.solve_with_tracking(max_iterations as usize);
        let energy = self.energy(&spins);
        let satisfied = !spins.iter().any(|&s| s == 0);
        SolveResult { spins, steps, satisfied, energy }
    }

    pub fn solve_with_tracking(&self, max_iterations: usize) -> (Vec<i8>, usize) {
        let n = self.n_vars;
        if n == 0 { return (vec![], 0); }

        let mut x = vec![0.01; n];
        let mut y = vec![0.01; n];
        let dt = 0.005;

        let mut converged_at = max_iterations;
        for iter in 0..max_iterations {
            let mut forces = self.fields.clone();
            for &(i, j, c) in &self.couplings {
                forces[i] += c * x[j];
                forces[j] += c * x[i];
            }

            for i in 0..n {
                let dx2 = (1.0 - self.k) * x[i] - x[i].powi(3) + forces[i];
                y[i] += dx2 * dt;
                x[i] += y[i] * dt;
                x[i] = x[i].clamp(-2.0, 2.0);
                y[i] = y[i].clamp(-2.0, 2.0);
            }

            if iter > 100 && iter % 50 == 0 {
                if x.iter().all(|&xi| xi.abs() > 0.9) {
                    converged_at = iter;
                    break;
                }
            }
        }

        let spins: Vec<i8> = x.iter().map(|&xi| {
            if xi > 0.3 { 1 } else if xi < -0.3 { -1 } else { 0 }
        }).collect();

        (spins, converged_at)
    }

    pub fn energy(&self, spins: &[i8]) -> f64 {
        let mut h = 0.0;
        for &(i, j, c) in &self.couplings {
            h -= c * (spins[i] as f64) * (spins[j] as f64);
        }
        h
    }
}
