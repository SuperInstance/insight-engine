use rand::Rng;
use crate::types::*;
use crate::experiments;
use crate::experiments::novel::hypothesis_generator::*;
use crate::mutators;
use crate::observers;

/// Run the insight engine with frontier-driven discovery.
pub fn run_engine(config: EngineConfig) -> EngineState {
    let mut state = EngineState::new();
    let mut frontier = KnowledgeFrontier::default();
    let mut rng = rand::thread_rng();

    println!("╔══════════════════════════════════════════════════╗");
    println!("║   INSIGHT ENGINE v2 — frontier-driven discovery ║");
    println!("║   experiments breed novel experiment TYPES       ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!();
    println!("Config: {} iterations, quality threshold {:.2}", config.max_iterations, config.quality_threshold);
    println!();

    for i in 0..config.max_iterations {
        state.iteration = i;

        // 1. Decide what experiment to run
        // Priority: frontier items > mutation of past insights > random exploration
        let experiment = if i % 3 == 0 && !frontier.items.is_empty() {
            // Every 3rd iteration, chase the frontier
            if let Some(item) = frontier.pop_highest_priority() {
                let (novel_type, params) = design_experiment(&item, &mut rng);
                println!("🔍 FRONTIER: {} → {:?}", item.description.chars().take(60).collect::<String>(), novel_type);
                Experiment {
                    id: state.total_experiments,
                    experiment_type: format!("{:?}", novel_type),
                    parameters: params,
                    parent_insight_id: None,
                    generation: 0,
                }
            } else {
                mutators::breed_next(&state)
            }
        } else if i % 5 == 0 {
            // Every 5th iteration, try a completely novel experiment type
            let novel_type = [
                "NormMultiplicative", "PhaseTransition", "SpectralAnalysis",
                "SymmetryBreaking", "DiskPacking", "ConstraintCascade",
                "TopologicalInvariants",
            ][rng.gen_range(0..7)];
            let params = generate_random_novel_params(novel_type, &mut rng);
            println!("🆕 NOVEL: {} experiment", novel_type);
            Experiment {
                id: state.total_experiments,
                experiment_type: novel_type.to_string(),
                parameters: params,
                parent_insight_id: None,
                generation: 0,
            }
        } else {
            mutators::breed_next(&state)
        };

        // 2. Execute
        let observations = execute_novel(&experiment, &mut state);

        // 3. Observe
        let insight = observers::extract(&experiment, &observations, &state);

        // 4. Update frontier with new anomalies
        let new_frontier_items = extract_frontier(&insight);
        for item in new_frontier_items {
            frontier.add(item);
        }

        // 5. Promote if quality exceeds threshold
        if insight.quality >= config.quality_threshold {
            state.total_insights += 1;
            state.insights.push(insight.clone());

            if state.best_insight.as_ref().map_or(true, |b| insight.quality > b.quality) {
                state.best_insight = Some(insight.clone());
            }
        }

        // 6. Print progress
        if insight.quality >= config.quality_threshold || i % 5 == 0 {
            print_progress(i, &experiment, &insight, &frontier);
        }
    }

    print_summary(&state, &frontier);
    state
}

fn execute_novel(experiment: &Experiment, state: &mut EngineState) -> Observations {
    match experiment.experiment_type.as_str() {
        "NormMultiplicative" => {
            let params: novel_norm_multiplicative::NormMultiplicativeParams =
                serde_json::from_value(experiment.parameters.clone()).unwrap_or(
                    novel_norm_multiplicative::NormMultiplicativeParams {
                        max_norm: 100, resonance_threshold: 1.5, compare_with_integers: true,
                    }
                );
            novel_norm_multiplicative::run(&params, state)
        }
        "PhaseTransition" => {
            let params: novel_phase_transition::PhaseTransitionParams =
                serde_json::from_value(experiment.parameters.clone()).unwrap_or(
                    novel_phase_transition::PhaseTransitionParams {
                        n_variables: 32, max_density: 1.0, density_steps: 20,
                        samples_per_density: 10, max_iterations: 1000, kerr_coefficient: 0.5,
                    }
                );
            novel_phase_transition::run(&params, state)
        }
        "SpectralAnalysis" => {
            let params: novel_spectral_analysis::SpectralParams =
                serde_json::from_value(experiment.parameters.clone()).unwrap_or(
                    novel_spectral_analysis::SpectralParams {
                        n_nodes: 64, edge_density: 0.3, hex_radius: 5,
                        coupling_pattern: novel_spectral_analysis::CouplingPattern::NormWeighted,
                    }
                );
            novel_spectral_analysis::run(&params, state)
        }
        "SymmetryBreaking" => {
            let params: novel_symmetry_breaking::SymmetryBreakingParams =
                serde_json::from_value(experiment.parameters.clone()).unwrap_or(
                    novel_symmetry_breaking::SymmetryBreakingParams {
                        lattice_radius: 10, perturbation_site: 0,
                        cascade_depth: 50, temperature: 0.5,
                    }
                );
            novel_symmetry_breaking::run(&params, state)
        }
        "DiskPacking" => {
            let params: novel_disk_packing::DiskPackingParams =
                serde_json::from_value(experiment.parameters.clone()).unwrap_or(
                    novel_disk_packing::DiskPackingParams {
                        lattice_radius: 15, disk_radius: 3,
                        strategy: novel_disk_packing::PackingStrategy::Greedy,
                    }
                );
            novel_disk_packing::run(&params, state)
        }
        "ConstraintCascade" => {
            let params: novel_constraint_cascade::ConstraintCascadeParams =
                serde_json::from_value(experiment.parameters.clone()).unwrap_or(
                    novel_constraint_cascade::ConstraintCascadeParams {
                        n_variables: 64, n_constraints: 100,
                        initial_breaks: 2, propagation: novel_constraint_cascade::PropagationRule::MajorityVote,
                        max_cascade_steps: 50,
                    }
                );
            novel_constraint_cascade::run(&params, state)
        }
        "TopologicalInvariants" => {
            let params: novel_topological_invariants::TopologicalParams =
                serde_json::from_value(experiment.parameters.clone()).unwrap_or(
                    novel_topological_invariants::TopologicalParams {
                        lattice_radius: 10, norm_filter: None, residue_filter: None,
                        rips_threshold: 1.5,
                    }
                );
            novel_topological_invariants::run(&params, state)
        }
        // Legacy types
        "EisensteinNormSweep" => {
            let params: experiments::eis_norm_sweep::EisNormParams =
                serde_json::from_value(experiment.parameters.clone()).unwrap_or_default();
            experiments::eis_norm_sweep::run(&params, state)
        }
        "SbmConvergence" => {
            let params: experiments::sbm_convergence::SbmParams =
                serde_json::from_value(experiment.parameters.clone()).unwrap_or_default();
            experiments::sbm_convergence::run(&params, state)
        }
        "HexDiskBoundary" => {
            let params: experiments::hex_disk_boundary::HexDiskParams =
                serde_json::from_value(experiment.parameters.clone()).unwrap_or_default();
            experiments::hex_disk_boundary::run(&params, state)
        }
        _ => {
            // Default: run norm sweep
            let params = experiments::eis_norm_sweep::EisNormParams::default();
            experiments::eis_norm_sweep::run(&params, state)
        }
    }
}

use crate::experiments::novel::{
    norm_multiplicative as novel_norm_multiplicative,
    phase_transition as novel_phase_transition,
    spectral_analysis as novel_spectral_analysis,
    symmetry_breaking as novel_symmetry_breaking,
    disk_packing as novel_disk_packing,
    constraint_cascade as novel_constraint_cascade,
    topological_invariants as novel_topological_invariants,
};

fn generate_random_novel_params(exp_type: &str, rng: &mut impl rand::Rng) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    match exp_type {
        "NormMultiplicative" => {
            map.insert("max_norm".into(), serde_json::json!(rng.gen_range(50..500)));
            map.insert("resonance_threshold".into(), serde_json::json!(rng.gen_range(1.0f64..3.0)));
            map.insert("compare_with_integers".into(), serde_json::json!(true));
        }
        "PhaseTransition" => {
            let n_vars_choices = [16, 32, 64, 128];
            map.insert("n_variables".into(), serde_json::json!(n_vars_choices[rng.gen_range(0..n_vars_choices.len())]));
            map.insert("max_density".into(), serde_json::json!(rng.gen_range(0.5f64..2.0)));
            map.insert("density_steps".into(), serde_json::json!(rng.gen_range(10u64..30)));
            map.insert("samples_per_density".into(), serde_json::json!(rng.gen_range(5u64..20)));
            map.insert("max_iterations".into(), serde_json::json!(1000u64));
            map.insert("kerr_coefficient".into(), serde_json::json!(rng.gen_range(0.1f64..1.0)));
        }
        "SpectralAnalysis" => {
            let n_choices = [32, 64, 128];
            map.insert("n_nodes".into(), serde_json::json!(n_choices[rng.gen_range(0..n_choices.len())]));
            map.insert("edge_density".into(), serde_json::json!(rng.gen_range(0.1f64..0.5)));
            map.insert("hex_radius".into(), serde_json::json!(rng.gen_range(3i32..10)));
            let patterns = ["NearestNeighbor", "NormWeighted", "NextNearestNeighbor", "Random"];
            map.insert("coupling_pattern".into(), serde_json::json!(patterns[rng.gen_range(0..patterns.len())]));
        }
        "SymmetryBreaking" => {
            map.insert("lattice_radius".into(), serde_json::json!(rng.gen_range(5i32..20)));
            map.insert("perturbation_site".into(), serde_json::json!(rng.gen_range(0usize..10)));
            map.insert("cascade_depth".into(), serde_json::json!(rng.gen_range(20u32..100)));
            map.insert("temperature".into(), serde_json::json!(rng.gen_range(0.1f64..2.0)));
        }
        "DiskPacking" => {
            map.insert("lattice_radius".into(), serde_json::json!(rng.gen_range(10i32..30)));
            map.insert("disk_radius".into(), serde_json::json!(rng.gen_range(2i32..8)));
            let strategies = ["Greedy", "AtPrimes", "AtNormRings", "Symmetric"];
            map.insert("strategy".into(), serde_json::json!(strategies[rng.gen_range(0..strategies.len())]));
        }
        "ConstraintCascade" => {
            let n_vars = [32, 64, 128, 256];
            map.insert("n_variables".into(), serde_json::json!(n_vars[rng.gen_range(0..n_vars.len())]));
            map.insert("n_constraints".into(), serde_json::json!(rng.gen_range(20usize..200)));
            map.insert("initial_breaks".into(), serde_json::json!(rng.gen_range(1usize..10)));
            let props = ["MajorityVote", "AnyNeighbor", "EisensteinWeighted"];
            map.insert("propagation".into(), serde_json::json!(props[rng.gen_range(0..props.len())]));
            map.insert("max_cascade_steps".into(), serde_json::json!(50u64));
        }
        "TopologicalInvariants" => {
            map.insert("lattice_radius".into(), serde_json::json!(rng.gen_range(5i32..15)));
            let nf = if rng.gen_bool(0.5) { Some(rng.gen_range(5i64..50)) } else { None };
            map.insert("norm_filter".into(), serde_json::json!(nf));
            let rf = if rng.gen_bool(0.3) { Some(rng.gen_range(2i64..7)) } else { None };
            map.insert("residue_filter".into(), serde_json::json!(rf));
            map.insert("rips_threshold".into(), serde_json::json!(rng.gen_range(1.0f64..3.0)));
        }
        _ => {}
    }
    serde_json::Value::Object(map)
}

fn print_progress(iteration: u64, experiment: &Experiment, insight: &Insight, frontier: &KnowledgeFrontier) {
    let quality_bar = "█".repeat(((insight.quality * 20.0) as usize).min(20));
    let empty_bar = "░".repeat(20 - ((insight.quality * 20.0) as usize).min(20));

    println!("━━━ Iter {} ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", iteration);
    println!("Type: {} │ Quality: [{}{}] {:.3}", experiment.experiment_type, quality_bar, empty_bar, insight.quality);
    println!("Result: {:.4} (surprise={:.3}, novelty={:.3})",
        insight.observations.primary, insight.surprise, insight.novelty);

    if !insight.observations.anomalies.is_empty() {
        println!("Anomalies ({}):", insight.observations.anomalies.len());
        for a in insight.observations.anomalies.iter().take(3) {
            println!("  ⚡ {}", a.chars().take(80).collect::<String>());
        }
    }

    println!("Frontier: {} items pending", frontier.items.len());
    println!();
}

fn print_summary(state: &EngineState, frontier: &KnowledgeFrontier) {
    println!();
    println!("╔══════════════════════════════════════════════════╗");
    println!("║           INSIGHT ENGINE v2 RESULTS              ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!();
    println!("Total experiments: {}", state.total_experiments);
    println!("Total insights:    {}", state.total_insights);
    println!("Frontier items:    {} pending, {} explored", frontier.items.len(), frontier.explored.len());

    if let Some(ref best) = state.best_insight {
        println!();
        println!("🏆 BEST INSIGHT (quality {:.3}):", best.quality);
        println!("   Type: {}", best.experiment_type);
        for line in best.insight_text.lines().take(5) {
            println!("   {}", line);
        }
    }

    // Group insights by type
    let mut type_counts: std::collections::HashMap<String, (usize, f64)> = std::collections::HashMap::new();
    for i in &state.insights {
        let entry = type_counts.entry(i.experiment_type.clone()).or_insert((0, 0.0));
        entry.0 += 1;
        entry.1 += i.quality;
    }
    println!();
    println!("INSIGHTS BY TYPE:");
    for (t, (count, total_q)) in &type_counts {
        println!("  {}: {} insights, avg quality {:.3}", t, count, total_q / *count as f64);
    }

    // Top frontier items (unexplored)
    if !frontier.items.is_empty() {
        println!();
        println!("UNEXPLORED FRONTIER (top 10):");
        for item in frontier.items.iter().take(10) {
            println!("  [{:.2}] {} → {:?}", item.priority,
                item.description.chars().take(60).collect::<String>(),
                item.follow_up_type);
        }
    }

    // Top insights
    let mut sorted: Vec<&Insight> = state.insights.iter().collect();
    sorted.sort_by(|a, b| b.quality.partial_cmp(&a.quality).unwrap_or(std::cmp::Ordering::Equal));

    if sorted.len() > 1 {
        println!();
        println!("TOP 10 INSIGHTS:");
        for (rank, insight) in sorted.iter().take(10).enumerate() {
            println!("  {}. [{:.3}] {} — {}",
                rank + 1, insight.quality, insight.experiment_type,
                insight.insight_text.lines().next().unwrap_or("").chars().take(60).collect::<String>()
            );
        }
    }
}

#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub max_iterations: u64,
    pub surprise_threshold: f64,
    pub quality_threshold: f64,
}

impl Default for EngineConfig {
    fn default() -> Self {
        EngineConfig {
            max_iterations: 100,
            surprise_threshold: 0.5,
            quality_threshold: 0.2,
        }
    }
}
