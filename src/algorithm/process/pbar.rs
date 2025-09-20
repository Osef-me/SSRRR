use crate::algorithm::smoothing::smooth_on_corners;
use crate::algorithm::process::ln::ln_sum;

/// Calcule les valeurs Pbar pour l'algorithme de star rating
/// 
/// # Arguments
/// * `_k` - Nombre de colonnes (non utilisé)
/// * `_t` - Temps total de la map (non utilisé)
/// * `x` - Paramètre de difficulté
/// * `note_seq` - Séquence des notes (colonne, hit_time, tail_time)
/// * `ln_rep` - Représentation sparse des long notes
/// * `anchor` - Valeurs d'anchor
/// * `base_corners` - Points de référence temporels
/// 
/// # Returns
/// Vecteur des valeurs Pbar
pub fn compute_pbar(
    _k: usize,
    _t: i64,
    x: f64,
    note_seq: &Vec<(usize, i64, i64)>,
    ln_rep: &(Vec<i64>, Vec<f64>, Vec<f64>),
    anchor: &Vec<f64>,
    base_corners: &Vec<f64>
) -> Vec<f64> {
    let n = base_corners.len();
    let mut p_step = vec![0.0; n];
    let stream_booster = |delta: f64| -> f64 {
        let r = 7.5 / delta;
        if 160.0 < r && r < 360.0 {
            1.0 + 1.7e-7 * (r - 160.0) * (r - 360.0).powi(2)
        } else { 1.0 }
    };

    for i in 0..note_seq.len().saturating_sub(1) {
        let h_l = note_seq[i].1 as f64;
        let h_r = note_seq[i + 1].1 as f64;
        let delta_time = h_r - h_l;
        if delta_time.abs() < 1e-9 {
            let spike = 1000.0 * (0.02 * (4.0 / x - 24.0)).powf(0.25);
            let left_idx = base_corners.partition_point(|&v| v < h_l);
            // right with side='right' => partition_point(|v| v <= h_l)
            let right_idx = base_corners.partition_point(|&v| v <= h_l);
            for idx in left_idx..right_idx {
                p_step[idx] += spike;
            }
            continue;
        }
        let left_idx = base_corners.partition_point(|&v| v < h_l);
        let right_idx = base_corners.partition_point(|&v| v < h_r);
        if left_idx >= right_idx { continue; }
        let delta = 0.001 * delta_time;
        let v = 1.0 + 6.0 * 0.001 * ln_sum(h_l, h_r, ln_rep);
        let b_val = stream_booster(delta);
        let inc = if delta < 2.0 * x / 3.0 {
            delta.powf(-1.0) * (0.08 * x.powf(-1.0) * (1.0 - 24.0 * x.powf(-1.0) * (delta - x / 2.0).powi(2))).powf(0.25) * b_val.max(v)
        } else {
            delta.powf(-1.0) * (0.08 * x.powf(-1.0) * (1.0 - 24.0 * x.powf(-1.0) * (x / 6.0).powi(2))).powf(0.25) * b_val.max(v)
        };
        for idx in left_idx..right_idx {
            let add = inc * anchor[idx];
            let alt = inc.max(inc * 2.0 - 10.0);
            p_step[idx] += add.min(alt);
        }
    }

    smooth_on_corners(base_corners, &p_step, 500.0, 0.001, "sum")
}
