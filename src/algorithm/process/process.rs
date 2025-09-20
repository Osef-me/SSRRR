use crate::algorithm::smoothing::rescale_high;
use crate::algorithm::interpolation::{interp_values, step_interp};
use crate::algorithm::process::preprocess::preprocess_file;

// Import des modules décomposés
use crate::algorithm::process::corners::get_corners;
use crate::algorithm::process::key_usage::{get_key_usage, get_key_usage_400};
use crate::algorithm::process::anchor::compute_anchor;
use crate::algorithm::process::ln::ln_bodies_count_sparse_representation;
use crate::algorithm::process::jbar::compute_jbar;
use crate::algorithm::process::xbar::compute_xbar;
use crate::algorithm::process::pbar::compute_pbar;
use crate::algorithm::process::abar::compute_abar;
use crate::algorithm::process::rbar::compute_rbar;
use crate::algorithm::process::ck::compute_c_and_ks;

/// Fonction principale de calcul du star rating
/// 
/// # Arguments
/// * `file_path` - Chemin vers le fichier .osu
/// * `mod_name` - Nom du mod à appliquer
/// 
/// # Returns
/// Valeur du star rating calculé
pub fn calculate(file_path: &str, mod_name: &str) -> f64 {
    // === Basic Setup and Parsing ===
    let (x, k, t, note_seq, note_seq_by_column, ln_seq, tail_seq, _ln_seq_by_column) =
        preprocess_file(file_path, mod_name);

    let (all_corners, base_corners, a_corners) = get_corners(t, &note_seq);

    let key_usage = get_key_usage(k, t, &note_seq, &base_corners);
    let active_columns: Vec<Vec<usize>> = (0..base_corners.len())
        .map(|i| {
            (0..k)
                .filter(|&col| key_usage[&col][i])
                .collect()
        })
        .collect();

    let key_usage_400 = get_key_usage_400(k, t, &note_seq, &base_corners);
    let anchor = compute_anchor(k, &key_usage_400, &base_corners);

    let (delta_ks, mut jbar) = compute_jbar(k, t, x, &note_seq_by_column, &base_corners);
    jbar = interp_values(&all_corners, &base_corners, &jbar);

    let mut xbar = compute_xbar(k, t, x, &note_seq_by_column, &active_columns, &base_corners);
    xbar = interp_values(&all_corners, &base_corners, &xbar);

    let ln_rep = ln_bodies_count_sparse_representation(&ln_seq, t);

    let mut pbar = compute_pbar(k, t, x, &note_seq, &ln_rep, &anchor, &base_corners);
    pbar = interp_values(&all_corners, &base_corners, &pbar);

    let mut abar = compute_abar(
        k,
        t,
        x,
        &note_seq_by_column,
        &active_columns,
        &delta_ks,
        &a_corners,
        &base_corners,
    );
    abar = interp_values(&all_corners, &a_corners, &abar);

    let mut rbar = compute_rbar(k, t, x, &note_seq_by_column, &tail_seq, &base_corners);
    rbar = interp_values(&all_corners, &base_corners, &rbar);

    let (c_step, ks_step) = compute_c_and_ks(k, t, &note_seq, &key_usage, &base_corners);
    let c_arr = step_interp(&all_corners, &base_corners, &c_step);
    let ks_arr = step_interp(&all_corners, &base_corners, &ks_step);

    // === Final Computations ===
    let mut s_all = Vec::new();
    let mut t_all = Vec::new();
    let mut d_all = Vec::new();

    for i in 0..all_corners.len() {
        let j = jbar[i];
        let x = xbar[i];
        let p = pbar[i];
        let a = abar[i];
        let r = rbar[i];
        let c = c_arr[i];
        let ks = ks_arr[i];

        let s_val = ((0.4 * (a.powf(3.0 / ks) * (j.min(8.0 + 0.85 * j))).powf(1.5))
            + (0.6 * (a.powf(2.0 / 3.0) * (0.8 * p + r * 35.0 / (c + 8.0))).powf(1.5)))
            .powf(2.0 / 3.0);

        let t_val = (a.powf(3.0 / ks) * x) / (x + s_val + 1.0);
        let d_val = 2.7 * s_val.sqrt() * t_val.powf(1.5) + s_val * 0.27;

        s_all.push(s_val);
        t_all.push(t_val);
        d_all.push(d_val);
    }

    // === Weighted aggregation ===
    let mut gaps = vec![0.0; all_corners.len()];
    if all_corners.len() >= 2 {
        gaps[0] = (all_corners[1] - all_corners[0]) as f64 / 2.0;
        gaps[all_corners.len() - 1] =
            (all_corners[all_corners.len() - 1] - all_corners[all_corners.len() - 2]) as f64 / 2.0;

        for i in 1..all_corners.len() - 1 {
            gaps[i] = (all_corners[i + 1] - all_corners[i - 1]) as f64 / 2.0;
        }
    }

    let effective_weights: Vec<f64> = c_arr
        .iter()
        .zip(gaps.iter())
        .map(|(c, g)| c * g)
        .collect();

    // Trier D et poids
    let mut idx: Vec<usize> = (0..d_all.len()).collect();
    idx.sort_by(|&i, &j| d_all[i].partial_cmp(&d_all[j]).unwrap());

    let d_sorted: Vec<f64> = idx.iter().map(|&i| d_all[i]).collect();
    let w_sorted: Vec<f64> = idx.iter().map(|&i| effective_weights[i]).collect();

    let mut cum_weights = Vec::new();
    let mut acc = 0.0;
    for &w in &w_sorted {
        acc += w;
        cum_weights.push(acc);
    }
    let total_weight = *cum_weights.last().unwrap_or(&1.0);
    let norm_cum_weights: Vec<f64> = cum_weights.iter().map(|cw| cw / total_weight).collect();

    let target_percentiles = [0.945, 0.935, 0.925, 0.915, 0.845, 0.835, 0.825, 0.815];
    let mut indices = Vec::new();
    for &p in &target_percentiles {
        if let Some(pos) = norm_cum_weights.iter().position(|&v| v >= p) {
            indices.push(pos);
        }
    }

    let percentile_93 = indices[..4].iter().map(|&i| d_sorted[i]).sum::<f64>() / 4.0;
    let percentile_83 = indices[4..8].iter().map(|&i| d_sorted[i]).sum::<f64>() / 4.0;

    let mut num = 0.0;
    let mut den = 0.0;
    for i in 0..d_sorted.len() {
        num += d_sorted[i].powf(5.0) * w_sorted[i];
        den += w_sorted[i];
    }
    let weighted_mean = (num / den).powf(1.0 / 5.0);

    // Final SR calculation
    let mut sr =
        (0.88 * percentile_93) * 0.25 + (0.94 * percentile_83) * 0.2 + weighted_mean * 0.55;
    sr = sr / 8.0 * 8.0;

    let total_notes: f64 = note_seq.len() as f64
        + 0.5 * ln_seq
            .iter()
            .map(|&(_, h, t)| ((t - h).min(1000).max(0) as f64) / 200.0)
            .sum::<f64>();

    sr *= total_notes / (total_notes + 60.0);
    sr = rescale_high(sr);
    sr *= 0.975;

    sr
}
