use std::collections::HashMap;
use crate::algorithm::smoothing::{smooth_on_corners, SmoothMode};
use crate::types::Note;

/// Computes Jbar values for the star rating algorithm
/// 
/// # Arguments
/// * `k` - Number of columns
/// * `_t` - Total map time (unused)
/// * `x` - Difficulty parameter
/// * `notes_by_column` - Notes organized by column
/// * `base_corners` - Reference time points
/// 
/// # Returns
/// Tuple (delta_ks, jbar) - deltas per column and Jbar values
pub fn compute_jbar(
    k: usize,
    _t: i64,
    x: f64,
    notes_by_column: &[Vec<Note>],
    base_corners: &[f64]
) -> (HashMap<usize, Vec<f64>>, Vec<f64>) {
    let n = base_corners.len();
    let mut j_ks: Vec<Vec<f64>> = vec![vec![0.0; n]; k];
    let mut delta_ks: Vec<Vec<f64>> = vec![vec![1e9; n]; k];
    let jack_nerfer = |delta: f64| -> f64 {
        1.0 - 7e-5 * (0.15 + (delta - 0.08).abs()).powf(-4.0)
    };

    let x_quarter = x.powf(0.25);
    for col in 0..k {
        let notes = &notes_by_column[col];
        if notes.len() < 2 { continue; }
        let mut left_idx = 0usize;
        let mut right_idx = 0usize;
        for i in 0..(notes.len() - 1) {
            let start = notes[i].hit_time as f64;
            let end = notes[i + 1].hit_time as f64;
            while left_idx < base_corners.len() && base_corners[left_idx] < start { left_idx += 1; }
            if right_idx < left_idx { right_idx = left_idx; }
            while right_idx < base_corners.len() && base_corners[right_idx] < end { right_idx += 1; }
            if left_idx >= right_idx { continue; }
            let delta = 0.001 * (end - start);
            let inv_delta = 1.0 / delta.max(1e-12);
            let val = inv_delta * (1.0 / (delta + 0.11 * x_quarter).max(1e-12));
            let j_val = val * jack_nerfer(delta);
            for idx in left_idx..right_idx { j_ks[col][idx] = j_val; }
            for idx in left_idx..right_idx { delta_ks[col][idx] = delta; }
        }
    }

    // Smooth each column's J_ks
    let mut jbar_ks: Vec<Vec<f64>> = Vec::with_capacity(k);
    for col in 0..k { jbar_ks.push(smooth_on_corners(base_corners, &j_ks[col], 500.0, 0.001, SmoothMode::Sum)); }

    // Aggregate across columns using weighted average
    let mut jbar = vec![0.0; n];
    for i in 0..n {
        let mut num = 0.0;
        let mut den = 0.0;
        for col in 0..k {
            let v = jbar_ks[col][i];
            let w = 1.0 / delta_ks[col][i];
            num += (v.max(0.0).powf(5.0)) * w;
            den += w;
        }
        let val = num / den.max(1e-9);
        jbar[i] = val.powf(1.0 / 5.0);
    }

    // Convert back to HashMap to preserve public API
    let mut delta_ks_map: HashMap<usize, Vec<f64>> = HashMap::with_capacity(k);
    for col in 0..k {
        delta_ks_map.insert(col, delta_ks[col].clone());
    }
    (delta_ks_map, jbar)
}


