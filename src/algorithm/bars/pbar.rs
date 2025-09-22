use crate::algorithm::smoothing::{smooth_on_corners, SmoothMode};
use crate::algorithm::calculations::ln::ln_sum;
use crate::types::Note;

/// Computes Pbar values for the star rating algorithm
/// 
/// # Arguments
/// * `_k` - Number of columns (unused)
/// * `_t` - Total map time (unused)
/// * `x` - Difficulty parameter
/// * `notes` - Note sequence
/// * `ln_rep` - Sparse representation of long notes
/// * `anchor` - Anchor values
/// * `base_corners` - Reference time points
/// 
/// # Returns
/// Vector of Pbar values
pub fn compute_pbar(
    _k: usize,
    _t: i64,
    x: f64,
    notes: &[Note],
    ln_rep: &(Vec<i64>, Vec<f64>, Vec<f64>),
    anchor: &[f64],
    base_corners: &[f64]
) -> Vec<f64> {
    let n = base_corners.len();
    let mut p_step = vec![0.0; n];
    let stream_booster = |delta: f64| -> f64 {
        let r = 7.5 / delta;
        if 160.0 < r && r < 360.0 {
            1.0 + 1.7e-7 * (r - 160.0) * (r - 360.0).powi(2)
        } else { 1.0 }
    };

    // Precompute constants
    let x_inv = 1.0 / x.max(1e-12);
    let x_inv_quarter = (0.08 * x_inv).powf(0.25);
    let two_thirds_x = (2.0 / 3.0) * x;
    let x_over_six = x / 6.0;
    let x_over_six_sq = x_over_six * x_over_six;

    let mut left_idx = 0usize;
    let mut right_idx = 0usize;
    for i in 0..notes.len().saturating_sub(1) {
        let h_l = notes[i].hit_time as f64;
        let h_r = notes[i + 1].hit_time as f64;
        let delta_time = h_r - h_l;
        if delta_time.abs() < 1e-9 {
            let spike = 1000.0 * (0.02 * (4.0 / x - 24.0)).powf(0.25);
            while left_idx < base_corners.len() && base_corners[left_idx] < h_l { left_idx += 1; }
            if right_idx < left_idx { right_idx = left_idx; }
            while right_idx < base_corners.len() && base_corners[right_idx] <= h_l { right_idx += 1; }
            for idx in left_idx..right_idx {
                p_step[idx] += spike;
            }
            continue;
        }
        
        while left_idx < base_corners.len() && base_corners[left_idx] < h_l { left_idx += 1; }
        if right_idx < left_idx { right_idx = left_idx; }
        while right_idx < base_corners.len() && base_corners[right_idx] < h_r { right_idx += 1; }
        if left_idx >= right_idx { continue; }
        let delta = 0.001 * delta_time;
        let v = 1.0 + 6.0 * 0.001 * ln_sum(h_l, h_r, ln_rep);
        let b_val = stream_booster(delta);
        let inc = if delta < two_thirds_x {
            let inv_delta = 1.0 / delta.max(1e-12);
            let inner = 1.0 - 24.0 * x_inv * (delta - x * 0.5).powi(2);
            inv_delta * (x_inv_quarter * inner.max(0.0).powf(0.25)) * b_val.max(v)
        } else {
            let inv_delta = 1.0 / delta.max(1e-12);
            let inner = 1.0 - 24.0 * x_inv * x_over_six_sq;
            inv_delta * (x_inv_quarter * inner.max(0.0).powf(0.25)) * b_val.max(v)
        };
        for idx in left_idx..right_idx {
            let add = inc * anchor[idx];
            let alt = (inc * 2.0 - 10.0).max(inc);
            p_step[idx] += add.min(alt);
        }
    }

    smooth_on_corners(base_corners, &p_step, 500.0, 0.001, SmoothMode::Sum)
}


