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
        // two-pointer stream over adjacent columns without allocating a merged vec
        let (a, b) = if col == 0 {
            (&notes_by_column[0][..], &[][..])
        } else if col == k {
            (&notes_by_column[k - 1][..], &[][..])
        } else {
            (&notes_by_column[col - 1][..], &notes_by_column[col][..])
        };

        // initialize prev time as the first available note
        let mut ia = 0usize;
        let mut ib = 0usize;
        let mut have_prev = false;
        let mut prev_time: f64 = 0.0;
        if ia < a.len() && (ib >= b.len() || a[ia].hit_time <= b[ib].hit_time) {
            prev_time = a[ia].hit_time as f64;
            ia += 1;
            have_prev = true;
        } else if ib < b.len() {
            prev_time = b[ib].hit_time as f64;
            ib += 1;
            have_prev = true;
        }
        if !have_prev { continue; }

        let mut idx_start = 0usize;
        let mut idx_end = 0usize;
        loop {
            let next_time_opt = if ia < a.len() && (ib >= b.len() || a[ia].hit_time <= b[ib].hit_time) {
                let t = a[ia].hit_time as f64;
                ia += 1;
                Some(t)
            } else if ib < b.len() {
                let t = b[ib].hit_time as f64;
                ib += 1;
                Some(t)
            } else {
                None
            };
            let next_time = match next_time_opt { Some(t) => t, None => break };

            // interval [prev_time, next_time)
            while idx_start < base_corners.len() && base_corners[idx_start] < prev_time { idx_start += 1; }
            if idx_end < idx_start { idx_end = idx_start; }
            while idx_end < base_corners.len() && base_corners[idx_end] < next_time { idx_end += 1; }
            if idx_start < idx_end {
                let delta = 0.001 * (next_time - prev_time);
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
            prev_time = next_time;
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

/// Faster variant using precomputed active flags per column and index
pub fn compute_xbar_flags(
    k: usize,
    _t: i64,
    x: f64,
    notes_by_column: &[Vec<Note>],
    active_flags: &[Vec<bool>], // active_flags[col][idx]
    base_corners: &[f64]
) -> Vec<f64> {
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
        let (a, b) = if col == 0 {
            (&notes_by_column[0][..], &[][..])
        } else if col == k {
            (&notes_by_column[k - 1][..], &[][..])
        } else {
            (&notes_by_column[col - 1][..], &notes_by_column[col][..])
        };

        let mut ia = 0usize;
        let mut ib = 0usize;
        let mut have_prev = false;
        let mut prev_time: f64 = 0.0;
        if ia < a.len() && (ib >= b.len() || a[ia].hit_time <= b[ib].hit_time) {
            prev_time = a[ia].hit_time as f64;
            ia += 1;
            have_prev = true;
        } else if ib < b.len() {
            prev_time = b[ib].hit_time as f64;
            ib += 1;
            have_prev = true;
        }
        if !have_prev { continue; }

        let mut idx_start = 0usize;
        let mut idx_end = 0usize;
        loop {
            let next_time_opt = if ia < a.len() && (ib >= b.len() || a[ia].hit_time <= b[ib].hit_time) {
                let t = a[ia].hit_time as f64;
                ia += 1;
                Some(t)
            } else if ib < b.len() {
                let t = b[ib].hit_time as f64;
                ib += 1;
                Some(t)
            } else {
                None
            };
            let next_time = match next_time_opt { Some(t) => t, None => break };

            while idx_start < n && base_corners[idx_start] < prev_time { idx_start += 1; }
            if idx_end < idx_start { idx_end = idx_start; }
            while idx_end < n && base_corners[idx_end] < next_time { idx_end += 1; }
            if idx_start < idx_end {
                let delta = 0.001 * (next_time - prev_time);
                let inv = 1.0 / (x.max(delta));
                let mut val = 0.16 * inv * inv;

                let cond1 = if col == 0 { false } else { !active_flags[col - 1][idx_start] && !active_flags[col - 1][idx_end.min(n - 1)] };
                let cond2 = if col == k { false } else { !active_flags[col][idx_start] && !active_flags[col][idx_end.min(n - 1)] };
                if cond1 || cond2 { val *= cross_complement[col]; }
                for idx in idx_start..idx_end { x_ks[col][idx] = val; }
                let base = (delta.max(0.06).max(0.75 * x)).powf(-2.0);
                let fc = (0.4 * base - 80.0).max(0.0);
                for idx in idx_start..idx_end { fast_cross[col][idx] = fc; }
            }
            prev_time = next_time;
        }
    }

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

    smooth_on_corners(base_corners, &x_base, 500.0, 0.001, crate::algorithm::smoothing::SmoothMode::Sum)
}


