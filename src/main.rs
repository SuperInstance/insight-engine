use insight_engine::{run_engine, EngineConfig};

fn main() {
    let config = EngineConfig {
        max_iterations: 50,
        surprise_threshold: 0.5,
        quality_threshold: 0.2,
    };

    let state = run_engine(config);

    // Save results
    let json = serde_json::to_string_pretty(&state).unwrap();
    std::fs::write("insight-results.json", &json).ok();
    println!("\nResults saved to insight-results.json");
}
