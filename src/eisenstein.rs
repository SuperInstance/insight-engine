//! Minimal local types for experiments (avoid circular deps with guard2mask-gpu).

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TernaryWeight { Neg, Zero, Pos }

#[derive(Debug, Clone, Copy)]
pub struct Domain(pub u8);

#[derive(Debug, Clone)]
pub enum Priority { Hard, Soft, Default }

#[derive(Debug, Clone)]
pub struct Constraint {
    pub name: String,
    pub priority: Priority,
    pub checks: Vec<Check>,
}

#[derive(Debug, Clone)]
pub enum Check {
    Range { start: f64, end: f64 },
    Equal(String),
    NotEqual(String),
}

#[derive(Debug, Clone)]
pub enum BinaryConstraint {
    Equal(usize, usize),
    NotEqual(usize, usize),
    Imply { from: usize, to: usize },
}

#[derive(Debug, Clone)]
pub struct ConstraintGraph {
    pub n_vars: usize,
    pub var_names: Vec<String>,
    pub initial_domains: Vec<Domain>,
    pub binary_constraints: Vec<BinaryConstraint>,
    pub adjacency: Vec<Vec<(usize, usize)>>,
}

impl ConstraintGraph {
    pub fn build(constraints: &[Constraint]) -> Result<Self, String> {
        let n_vars = constraints.len();
        let var_names: Vec<String> = constraints.iter().map(|c| c.name.clone()).collect();
        let name_to_idx: std::collections::HashMap<String, usize> = var_names.iter()
            .enumerate().map(|(i, n)| (n.clone(), i)).collect();

        let mut binary_constraints = Vec::new();
        let mut adjacency = vec![Vec::new(); n_vars];

        for (idx, c) in constraints.iter().enumerate() {
            for check in &c.checks {
                match check {
                    Check::Equal(target) => {
                        if let Some(&j) = name_to_idx.get(target) {
                            binary_constraints.push(BinaryConstraint::Equal(idx, j));
                            adjacency[idx].push((j, binary_constraints.len() - 1));
                            adjacency[j].push((idx, binary_constraints.len() - 1));
                        }
                    }
                    Check::NotEqual(target) => {
                        if let Some(&j) = name_to_idx.get(target) {
                            binary_constraints.push(BinaryConstraint::NotEqual(idx, j));
                            adjacency[idx].push((j, binary_constraints.len() - 1));
                            adjacency[j].push((idx, binary_constraints.len() - 1));
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(ConstraintGraph { n_vars, var_names, initial_domains: vec![], binary_constraints, adjacency })
    }
}
