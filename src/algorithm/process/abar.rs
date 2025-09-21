use std::collections::HashMap;
use crate::algorithm::smoothing::smooth_on_corners;
use crate::types::Note;

/// Calcule les valeurs Abar pour l'algorithme de star rating
/// 
/// # Arguments
/// * `k` - Nombre de colonnes
/// * `_t` - Temps total de la map (non utilisé)
/// * `_x` - Paramètre de difficulté (non utilisé)
/// * `_notes_by_column` - Notes organisées par colonne (non utilisé)
/// * `active_columns` - Colonnes actives à chaque point temporel
/// * `delta_ks` - Deltas par colonne
/// * `a_corners` - Points de référence temporels pour A
/// * `base_corners` - Points de référence temporels de base
/// 
/// # Returns
/// Vecteur des valeurs Abar
pub fn compute_abar(
    k: usize,
    _t: i64,
    _x: f64,
    _notes_by_column: &[Vec<Note>],
    active_columns: &[Vec<usize>],
    delta_ks: &HashMap<usize, Vec<f64>>,
    a_corners: &[f64],
    base_corners: &[f64]
) -> Vec<f64> {
    let n = base_corners.len();
    // dks: k-1 x n
    let mut dks: Vec<Vec<f64>> = vec![vec![0.0; n]; k.saturating_sub(1)];
    for i in 0..n {
        let cols = &active_columns[i];
        for j in 0..cols.len().saturating_sub(1) {
            let k0 = cols[j];
            let k1 = cols[j + 1];
            let dk0 = delta_ks.get(&k0).unwrap()[i];
            let dk1 = delta_ks.get(&k1).unwrap()[i];
            dks[k0][i] = (dk0 - dk1).abs() + 0.4 * ((dk0.max(dk1) - 0.11).max(0.0));
        }
    }

    let mut a_step = vec![1.0; a_corners.len()];
    for (i, &s) in a_corners.iter().enumerate() {
        let mut idx = base_corners.partition_point(|&v| v < s);
        if idx >= base_corners.len() { idx = base_corners.len() - 1; }
        let cols = &active_columns[idx];
        for j in 0..cols.len().saturating_sub(1) {
            let k0 = cols[j];
            let k1 = cols[j + 1];
            let d_val = dks[k0][idx];
            let dk0 = delta_ks.get(&k0).unwrap()[idx];
            let dk1 = delta_ks.get(&k1).unwrap()[idx];
            if d_val < 0.02 {
                a_step[i] *= (0.75 + 0.5 * dk0.max(dk1)).min(1.0);
            } else if d_val < 0.07 {
                a_step[i] *= (0.65 + 5.0 * d_val + 0.5 * dk0.max(dk1)).min(1.0);
            }
        }
    }
    smooth_on_corners(a_corners, &a_step, 250.0, 1.0, "avg")
}
