use crate::algorithm::smoothing::smooth_on_corners;
use crate::types::Note;

/// Computes Xbar values for the star rating algorithm
/// 
/// # Arguments
/// * `k` - Number of columns
/// * `_t` - Total map time (unused)
/// * `x` - Difficulty parameter
/// * `notes_by_column` - Notes organized by column
/// * `active_columns` - Active columns at each time point
/// * `base_corners` - Reference time points
/// 
/// # Returns
/// Vector of Xbar values
pub fn compute_xbar(
    k: usize,
    _t: i64,
    x: f64,
    notes_by_column: &[Vec<Note>],
    active_columns: &[Vec<usize>],
    base_corners: &[f64]
) -> Vec<f64> {
    // cross_matrix given in original script
    let cross_matrix: Vec<Vec<f64>> = vec![
        vec![-1.0],
        vec![0.075, 0.075],
        vec![0.125, 0.05, 0.125],
        vec![0.125, 0.125, 0.125, 0.125],
        vec![0.175, 0.25, 0.05, 0.25, 0.175],
        vec![0.175, 0.25, 0.175, 0.175, 0.25, 0.175],
        vec![0.225, 0.35, 0.25, 0.05, 0.25, 0.35, 0.225],
        vec![0.225, 0.35, 0.25, 0.225, 0.225, 0.25, 0.35, 0.225],
        vec![0.275, 0.45, 0.35, 0.25, 0.05, 0.25, 0.35, 0.45, 0.275],
        vec![0.275, 0.45, 0.35, 0.25, 0.275, 0.275, 0.25, 0.35, 0.45, 0.275],
        vec![0.325, 0.55, 0.45, 0.35, 0.25, 0.05, 0.25, 0.35, 0.45, 0.55, 0.325]
    ];
    let cross_coeff = cross_matrix.get(k).expect("cross_matrix[k] exists");
    let mut cross_complement: Vec<f64> = Vec::with_capacity(cross_coeff.len());
    for &c in cross_coeff.iter() { cross_complement.push(1.0 - c); }

    let n = base_corners.len();
    let mut x_ks: Vec<Vec<f64>> = vec![vec![0.0; n]; k + 1];
    let mut fast_cross: Vec<Vec<f64>> = vec![vec![0.0; n]; k + 1];

    for col in 0..=k {
        // build notes_in_pair
        let notes_in_pair: Vec<Note> = if col == 0 {
            notes_by_column[0].clone()
        } else if col == k {
            notes_by_column[k - 1].clone()
        } else {
            // merge sorted lists notes_by_column[col-1] and notes_by_column[col]
            let a = &notes_by_column[col - 1];
            let b = &notes_by_column[col];
            let mut merged = Vec::with_capacity(a.len() + b.len());
            let mut ia = 0usize;
            let mut ib = 0usize;
            while ia < a.len() && ib < b.len() {
                if a[ia].hit_time <= b[ib].hit_time {
                    merged.push(a[ia]);
                    ia += 1;
                } else {
                    merged.push(b[ib]);
                    ib += 1;
                }
            }
            if ia < a.len() { merged.extend_from_slice(&a[ia..]); }
            if ib < b.len() { merged.extend_from_slice(&b[ib..]); }
            merged
        };

        if notes_in_pair.len() < 2 { continue; }
        let mut idx_start = 0usize;
        let mut idx_end = 0usize;
        for i in 1..notes_in_pair.len() {
            let start = notes_in_pair[i - 1].hit_time as f64;
            let end = notes_in_pair[i].hit_time as f64;
            while idx_start < base_corners.len() && base_corners[idx_start] < start { idx_start += 1; }
            if idx_end < idx_start { idx_end = idx_start; }
            while idx_end < base_corners.len() && base_corners[idx_end] < end { idx_end += 1; }
            if idx_start >= idx_end { continue; }
            let delta = 0.001 * (end - start);
            let inv = 1.0 / (x.max(delta));
            let mut val = 0.16 * inv * inv;

            // check active_columns condition
            let cond1 = {
                let a0 = active_columns.get(idx_start).map(|v| v.contains(&(col - 1))).unwrap_or(false);
                let a1 = active_columns.get(idx_end).map(|v| v.contains(&(col - 1))).unwrap_or(false);
                !a0 && !a1
            };
            let cond2 = {
                let b0 = active_columns.get(idx_start).map(|v| v.contains(&col)).unwrap_or(false);
                let b1 = active_columns.get(idx_end).map(|v| v.contains(&col)).unwrap_or(false);
                !b0 && !b1
            };
            if cond1 || cond2 { val *= cross_complement[col]; }
            for idx in idx_start..idx_end { x_ks[col][idx] = val; }
            let base = (delta.max(0.06).max(0.75 * x)).powf(-2.0);
            let fc = (0.4 * base - 80.0).max(0.0);
            for idx in idx_start..idx_end { fast_cross[col][idx] = fc; }
        }
    }

    // compute X_base
    let mut x_base = vec![0.0; n];
    for i in 0..n {
        let mut sum1 = 0.0;
        for col in 0..=k { sum1 += x_ks[col][i] * cross_coeff[col]; }
        let mut sum2 = 0.0;
        for col in 0..k {
            let v1 = fast_cross[col][i];
            let v2 = fast_cross[col + 1][i];
            let c1 = cross_coeff[col];
            let c2 = cross_coeff[col + 1];
            sum2 += (v1 * c1 * v2 * c2).sqrt();
        }
        x_base[i] = sum1 + sum2;
    }

    let xbar = smooth_on_corners(base_corners, &x_base, 500.0, 0.001, crate::algorithm::smoothing::SmoothMode::Sum);
    xbar
}


