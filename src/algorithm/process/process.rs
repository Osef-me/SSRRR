use crate::algorithm::smoothing::rescale_high;
use crate::algorithm::interpolation::{interp_values, step_interp};
use crate::types::{MapData, StarRating, StarRatingComponents, StarRatingResult};

// Import des modules décomposés
use crate::algorithm::calculations::corners::get_corners;
use crate::algorithm::calculations::key_usage::{get_key_usage, get_key_usage_400};
use crate::algorithm::calculations::anchor::compute_anchor;
use crate::algorithm::calculations::ln::ln_bodies_count_sparse_representation;
use crate::algorithm::bars::jbar::compute_jbar;
use crate::algorithm::bars::xbar::compute_xbar;
use crate::algorithm::bars::pbar::compute_pbar;
use crate::algorithm::bars::abar::compute_abar;
use crate::algorithm::bars::rbar::compute_rbar;
use crate::algorithm::calculations::ck::compute_c_and_ks;


/// Main star rating calculation function
/// 
/// # Arguments
/// * `map_data` - Parsed map data
/// 
/// # Returns
/// Detailed star rating calculation result
pub fn calculate(map_data: &MapData) -> StarRatingResult<StarRating> {
    let rating = calculate_internal(map_data);

    Ok(StarRating {
        rating,
        components: StarRatingComponents::new(
            Vec::new(), // TODO: Extract components from algorithm
            Vec::new(),
            Vec::new(),
            Vec::new(),
            0.0,
            0.0,
            0.0,
        ),
    })
}

/// Internal calculation function
fn calculate_internal(map_data: &MapData) -> f64 {
    // === Phase 1: Data preparation ===
    let (all_corners, base_corners, a_corners) = get_corners(map_data.total_duration, &map_data.notes);
    let key_usage = get_key_usage(map_data.column_count, map_data.total_duration, &map_data.notes, &base_corners);
    let active_columns = compute_active_columns(&key_usage, map_data.column_count, base_corners.len());
    let key_usage_400 = get_key_usage_400(map_data.column_count, map_data.total_duration, &map_data.notes, &base_corners);
    let anchor = compute_anchor(map_data.column_count, &key_usage_400, &base_corners);

    // === Phase 2: Bar calculations ===
    let (jbar, xbar, pbar, abar, rbar, c_arr, ks_arr) = compute_all_bars(
        map_data, &active_columns, &a_corners, &base_corners, &all_corners, &anchor
    );

    // === Phase 3: Final value calculations ===
    let (_s_all, _t_all, d_all) = compute_final_values(&jbar, &xbar, &pbar, &abar, &rbar, &c_arr, &ks_arr);

    // === Phase 4: Weighted aggregation ===
    let (percentile_93, percentile_83, weighted_mean) = compute_weighted_aggregation(&d_all, &c_arr, &all_corners);

    // === Phase 5: Final star rating calculation ===
    compute_final_star_rating(percentile_93, percentile_83, weighted_mean, &map_data.notes, &map_data.long_notes)
}

/// Computes active columns for each time point
#[inline]
fn compute_active_columns(key_usage: &std::collections::HashMap<usize, Vec<bool>>, k: usize, n: usize) -> Vec<Vec<usize>> {
    (0..n)
        .map(|i| {
            let mut active = Vec::with_capacity(k);
            for col in 0..k {
                if key_usage.get(&col).map_or(false, |v| v[i]) {
                    active.push(col);
                }
            }
            active
        })
        .collect()
}

/// Calculates all bars (jbar, xbar, pbar, abar, rbar) and c/ks arrays
fn compute_all_bars(
    map_data: &MapData,
    active_columns: &[Vec<usize>],
    a_corners: &[f64],
    base_corners: &[f64],
    all_corners: &[f64],
    anchor: &[f64],
) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let (delta_ks, mut jbar) = compute_jbar(map_data.column_count, map_data.total_duration, map_data.hit_leniency, &map_data.notes_by_column, base_corners);
    jbar = interp_values(all_corners, base_corners, &jbar);

    let mut xbar = compute_xbar(map_data.column_count, map_data.total_duration, map_data.hit_leniency, &map_data.notes_by_column, active_columns, base_corners);
    xbar = interp_values(all_corners, base_corners, &xbar);

    let ln_rep = ln_bodies_count_sparse_representation(&map_data.long_notes, map_data.total_duration);
    let mut pbar = compute_pbar(map_data.column_count, map_data.total_duration, map_data.hit_leniency, &map_data.notes, &ln_rep, anchor, base_corners);
    pbar = interp_values(all_corners, base_corners, &pbar);

    let mut abar = compute_abar(map_data.column_count, map_data.total_duration, map_data.hit_leniency, &map_data.notes_by_column, active_columns, &delta_ks, a_corners, base_corners);
    abar = interp_values(all_corners, a_corners, &abar);

    let mut rbar = compute_rbar(map_data.column_count, map_data.total_duration, map_data.hit_leniency, &map_data.notes_by_column, &map_data.tail_sequence, base_corners);
    rbar = interp_values(all_corners, base_corners, &rbar);

    let key_usage = get_key_usage(map_data.column_count, map_data.total_duration, &map_data.notes, base_corners);
    let (c_step, ks_step) = compute_c_and_ks(map_data.column_count, map_data.total_duration, &map_data.notes, &key_usage, base_corners);
    let c_arr = step_interp(all_corners, base_corners, &c_step);
    let ks_arr = step_interp(all_corners, base_corners, &ks_step);

    (jbar, xbar, pbar, abar, rbar, c_arr, ks_arr)
}

/// Computes final S, T and D values
fn compute_final_values(
    jbar: &[f64],
    xbar: &[f64],
    pbar: &[f64],
    abar: &[f64],
    rbar: &[f64],
    c_arr: &[f64],
    ks_arr: &[f64],
) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let s_all: Vec<f64> = jbar.iter()
        .zip(xbar.iter())
        .zip(pbar.iter())
        .zip(abar.iter())
        .zip(rbar.iter())
        .zip(c_arr.iter())
        .zip(ks_arr.iter())
        .map(|((((((&j, &_x), &p), &a), &r), &c), &ks)| {
            ((0.4 * (a.powf(3.0 / ks) * (j.min(8.0 + 0.85 * j))).powf(1.5))
                + (0.6 * (a.powf(2.0 / 3.0) * (0.8 * p + r * 35.0 / (c + 8.0))).powf(1.5)))
                .powf(2.0 / 3.0)
        })
        .collect();

    let t_all: Vec<f64> = s_all.iter()
        .zip(xbar.iter())
        .zip(abar.iter())
        .zip(ks_arr.iter())
        .map(|(((&s_val, &x), &a), &ks)| (a.powf(3.0 / ks) * x) / (x + s_val + 1.0))
        .collect();

    let d_all: Vec<f64> = s_all.iter()
        .zip(t_all.iter())
        .map(|(&s_val, &t_val)| 2.7 * s_val.sqrt() * t_val.powf(1.5) + s_val * 0.27)
        .collect();

    (s_all, t_all, d_all)
}

/// Computes weighted aggregation and percentiles
fn compute_weighted_aggregation(
    d_all: &[f64],
    c_arr: &[f64],
    all_corners: &[f64],
) -> (f64, f64, f64) {
    // Calculate gaps
    let gaps = compute_gaps(all_corners);
    
    // Calculate effective weights
    let effective_weights: Vec<f64> = c_arr.iter()
        .zip(gaps.iter())
        .map(|(c, g)| c * g)
        .collect();

    // Sort and calculate percentiles - use unstable sort for better performance
    let mut indices: Vec<usize> = (0..d_all.len()).collect();
    indices.sort_unstable_by(|&i, &j| d_all[i].partial_cmp(&d_all[j]).expect("Valeurs finies attendues"));

    let d_sorted: Vec<f64> = indices.iter().map(|&i| d_all[i]).collect();
    let w_sorted: Vec<f64> = indices.iter().map(|&i| effective_weights[i]).collect();

    // Calculate cumulative weights
    let cum_weights: Vec<f64> = w_sorted.iter()
        .scan(0.0, |acc, &w| {
            *acc += w;
            Some(*acc)
        })
        .collect();

    let total_weight = cum_weights.last().unwrap_or(&1.0);
    let norm_cum_weights: Vec<f64> = cum_weights.iter()
        .map(|cw| cw / total_weight)
        .collect();

    // Calculate percentiles
    let target_percentiles = [0.945, 0.935, 0.925, 0.915, 0.845, 0.835, 0.825, 0.815];
    let indices: Vec<usize> = target_percentiles.iter()
        .filter_map(|&p| norm_cum_weights.iter().position(|&v| v >= p))
        .collect();

    let percentile_93 = indices[..4].iter().map(|&i| d_sorted[i]).sum::<f64>() / 4.0;
    let percentile_83 = indices[4..8].iter().map(|&i| d_sorted[i]).sum::<f64>() / 4.0;

    // Calculate weighted mean
    let (num, den) = d_sorted.iter()
        .zip(w_sorted.iter())
        .fold((0.0, 0.0), |(num, den), (&d, &w)| {
            (num + d.powf(5.0) * w, den + w)
        });
    let weighted_mean = (num / den).powf(1.0 / 5.0);

    (percentile_93, percentile_83, weighted_mean)
}

/// Computes gaps between corners
#[inline]
fn compute_gaps(all_corners: &[f64]) -> Vec<f64> {
    let n = all_corners.len();
    if n < 2 {
        return vec![0.0; n];
    }

    let mut gaps = Vec::with_capacity(n);
    gaps.push((all_corners[1] - all_corners[0]) / 2.0);
    
    for i in 1..n - 1 {
        gaps.push((all_corners[i + 1] - all_corners[i - 1]) / 2.0);
    }
    
    gaps.push((all_corners[n - 1] - all_corners[n - 2]) / 2.0);
    gaps
}

/// Computes final star rating
fn compute_final_star_rating(
    percentile_93: f64,
    percentile_83: f64,
    weighted_mean: f64,
    notes: &[crate::types::Note],
    long_notes: &[crate::types::Note],
) -> f64 {
    let mut sr = (0.88 * percentile_93) * 0.25 + (0.94 * percentile_83) * 0.2 + weighted_mean * 0.55;
    sr = sr / 8.0 * 8.0;

    let total_notes: f64 = notes.len() as f64
        + 0.5 * long_notes.iter()
            .map(|note| {
                let duration = note.duration();
                ((duration).min(1000).max(0) as f64) / 200.0
            })
            .sum::<f64>();

    sr *= total_notes / (total_notes + 60.0);
    sr = rescale_high(sr);
    sr *= 0.975;

    sr
}
