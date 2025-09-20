use std::collections::HashMap;
use crate::algorithm::smoothing::smooth_on_corners;

/// Calcule les valeurs Jbar pour l'algorithme de star rating
/// 
/// # Arguments
/// * `k` - Nombre de colonnes
/// * `_t` - Temps total de la map (non utilisé)
/// * `x` - Paramètre de difficulté
/// * `note_seq_by_column` - Notes organisées par colonne
/// * `base_corners` - Points de référence temporels
/// 
/// # Returns
/// Tuple contenant (delta_ks, jbar) - les deltas par colonne et les valeurs Jbar
pub fn compute_jbar(
    k: usize,
    _t: i64,
    x: f64,
    note_seq_by_column: &Vec<Vec<(usize, i64, i64)>>,
    base_corners: &Vec<f64>
) -> (HashMap<usize, Vec<f64>>, Vec<f64>) {
    let n = base_corners.len();
    let mut j_ks: HashMap<usize, Vec<f64>> = HashMap::new();
    let mut delta_ks: HashMap<usize, Vec<f64>> = HashMap::new();
    for col in 0..k {
        j_ks.insert(col, vec![0.0; n]);
        delta_ks.insert(col, vec![1e9; n]);
    }
    let jack_nerfer = |delta: f64| -> f64 {
        1.0 - 7e-5 * (0.15 + (delta - 0.08).abs()).powf(-4.0)
    };

    for col in 0..k {
        let notes = &note_seq_by_column[col];
        if notes.len() < 2 { continue; }
        for i in 0..(notes.len() - 1) {
            let start = notes[i].1;
            let end = notes[i + 1].1;
            let left_idx = base_corners.partition_point(|&v| v < start as f64);
            let right_idx = base_corners.partition_point(|&v| v < end as f64);
            if left_idx >= right_idx { continue; }
            let delta = 0.001 * ((end - start) as f64);
            let val = delta.powf(-1.0) * (delta + 0.11 * x.powf(0.25)).powf(-1.0);
            let j_val = val * jack_nerfer(delta);
            if let Some(jvec) = j_ks.get_mut(&col) {
                for idx in left_idx..right_idx {
                    jvec[idx] = j_val;
                }
            }
            if let Some(dvec) = delta_ks.get_mut(&col) {
                for idx in left_idx..right_idx {
                    dvec[idx] = delta;
                }
            }
        }
    }

    // Smooth each column's J_ks
    let mut jbar_ks: HashMap<usize, Vec<f64>> = HashMap::new();
    for col in 0..k {
        let jvec = j_ks.get(&col).unwrap();
        let sm = smooth_on_corners(base_corners, jvec, 500.0, 0.001, "sum");
        jbar_ks.insert(col, sm);
    }

    // Aggregate across columns using weighted average
    let mut jbar = vec![0.0; n];
    for i in 0..n {
        let mut num = 0.0;
        let mut den = 0.0;
        for col in 0..k {
            let v = jbar_ks.get(&col).unwrap()[i];
            let w = 1.0 / delta_ks.get(&col).unwrap()[i];
            num += (v.max(0.0).powf(5.0)) * w;
            den += w;
        }
        let val = num / den.max(1e-9);
        jbar[i] = val.powf(1.0 / 5.0);
    }

    (delta_ks, jbar)
}
