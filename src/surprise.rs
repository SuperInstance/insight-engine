//! Shared math utilities.

/// Eisenstein norm
pub fn eis_norm(a: i32, b: i32) -> i64 {
    let a = a as i64; let b = b as i64;
    a * a - a * b + b * b
}

/// Check if point is in hex disk
pub fn eis_in_disk(pa: i32, pb: i32, ca: i32, cb: i32, r_sq: i64) -> bool {
    let da = pa as i64 - ca as i64;
    let db = pb as i64 - cb as i64;
    let norm = da * da - da * db + db * db;
    norm <= r_sq
}

/// Calculate z-score
pub fn z_score(value: f64, mean: f64, variance: f64) -> f64 {
    let std_dev = variance.sqrt().max(0.0001);
    (value - mean) / std_dev
}

/// Relative error
pub fn relative_error(actual: f64, expected: f64) -> f64 {
    if expected.abs() < 0.0001 { return 0.0; }
    ((actual - expected) / expected).abs()
}
