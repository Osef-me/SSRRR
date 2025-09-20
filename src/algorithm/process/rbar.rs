use crate::algorithm::smoothing::smooth_on_corners;
use crate::algorithm::utils::find_next_note_in_column;

/// Calcule les valeurs Rbar pour l'algorithme de star rating
/// 
/// # Arguments
/// * `_k` - Nombre de colonnes (non utilisé)
/// * `_t` - Temps total de la map (non utilisé)
/// * `x` - Paramètre de difficulté
/// * `note_seq_by_column` - Notes organisées par colonne
/// * `tail_seq` - Séquence des queues de long notes
/// * `base_corners` - Points de référence temporels
/// 
/// # Returns
/// Vecteur des valeurs Rbar
pub fn compute_rbar(
    _k: usize,
    _t: i64,
    x: f64,
    note_seq_by_column: &Vec<Vec<(usize, i64, i64)>>,
    tail_seq: &Vec<(usize, i64, i64)>,
    base_corners: &Vec<f64>
) -> Vec<f64> {
    let n = base_corners.len();
    let mut i_arr = vec![0.0; n];
    let mut r_step = vec![0.0; n];

    let mut times_by_column: Vec<Vec<i64>> = Vec::with_capacity(note_seq_by_column.len());
    for col in note_seq_by_column.iter() {
        times_by_column.push(col.iter().map(|&(_k, h, _t)| h).collect());
    }

    let mut i_list: Vec<f64> = Vec::with_capacity(tail_seq.len());
    for i in 0..tail_seq.len() {
        let (k, h_i, t_i) = tail_seq[i];
        let nxt = find_next_note_in_column((k, h_i, t_i), &times_by_column[k], note_seq_by_column);
        let h_j = nxt.1;
        let i_h = 0.001 * ((t_i - h_i - 80).abs() as f64) / x;
        let i_t = 0.001 * ((h_j - t_i - 80).abs() as f64) / x;
        i_list.push(2.0 / (2.0 + (-5.0 * (i_h - 0.75)).exp() + (-5.0 * (i_t - 0.75)).exp()));
    }

    for i in 0..tail_seq.len().saturating_sub(1) {
        let t_start = tail_seq[i].2;
        let t_end = tail_seq[i + 1].2;
        let left_idx = base_corners.partition_point(|&v| v < t_start as f64);
        let right_idx = base_corners.partition_point(|&v| v < t_end as f64);
        if left_idx >= right_idx { continue; }
        for idx in left_idx..right_idx {
            i_arr[idx] = 1.0 + i_list[i];
        }
        let delta_r = 0.001 * ((tail_seq[i + 1].2 - tail_seq[i].2) as f64);
        for idx in left_idx..right_idx {
            r_step[idx] = 0.08 * delta_r.powf(-0.5) * x.powf(-1.0) * (1.0 + 0.8 * (i_list[i] + i_list[i + 1]));
        }
    }

    smooth_on_corners(base_corners, &r_step, 500.0, 0.001, "sum")
}
