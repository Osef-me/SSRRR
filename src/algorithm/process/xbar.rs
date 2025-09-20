use std::collections::HashMap;
use crate::algorithm::smoothing::smooth_on_corners;

/// Calcule les valeurs Xbar pour l'algorithme de star rating
/// 
/// # Arguments
/// * `k` - Nombre de colonnes
/// * `_t` - Temps total de la map (non utilisé)
/// * `x` - Paramètre de difficulté
/// * `note_seq_by_column` - Notes organisées par colonne
/// * `active_columns` - Colonnes actives à chaque point temporel
/// * `base_corners` - Points de référence temporels
/// 
/// # Returns
/// Vecteur des valeurs Xbar
pub fn compute_xbar(
    k: usize,
    _t: i64,
    x: f64,
    note_seq_by_column: &Vec<Vec<(usize, i64, i64)>>,
    active_columns: &Vec<Vec<usize>>,
    base_corners: &Vec<f64>
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

    let n = base_corners.len();
    let mut x_ks: HashMap<usize, Vec<f64>> = HashMap::new();
    let mut fast_cross: HashMap<usize, Vec<f64>> = HashMap::new();
    for col in 0..=k {
        x_ks.insert(col, vec![0.0; n]);
        fast_cross.insert(col, vec![0.0; n]);
    }

    for col in 0..=k {
        // build notes_in_pair
        let notes_in_pair: Vec<(usize, i64, i64)> = if col == 0 {
            note_seq_by_column[0].clone()
        } else if col == k {
            note_seq_by_column[k - 1].clone()
        } else {
            // merge sorted lists note_seq_by_column[col-1] and note_seq_by_column[col]
            let a = &note_seq_by_column[col - 1];
            let b = &note_seq_by_column[col];
            let mut merged = Vec::with_capacity(a.len() + b.len());
            let mut ia = 0usize;
            let mut ib = 0usize;
            while ia < a.len() && ib < b.len() {
                if a[ia].1 <= b[ib].1 {
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
        for i in 1..notes_in_pair.len() {
            let start = notes_in_pair[i - 1].1;
            let end = notes_in_pair[i].1;
            let idx_start = base_corners.partition_point(|&v| v < start as f64);
            let idx_end = base_corners.partition_point(|&v| v < end as f64);
            if idx_start >= idx_end { continue; }
            let delta = 0.001 * ((notes_in_pair[i].1 - notes_in_pair[i - 1].1) as f64);
            let mut val = 0.16 * (x.max(delta)).powf(-2.0);

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
            if cond1 || cond2 {
                val *= 1.0 - cross_coeff[col];
            }
            if let Some(xvec) = x_ks.get_mut(&col) {
                for idx in idx_start..idx_end {
                    xvec[idx] = val;
                }
            }
            if let Some(fvec) = fast_cross.get_mut(&col) {
                let base = (delta.max(0.06).max(0.75 * x)).powf(-2.0);
                let fc = (0.4 * base - 80.0).max(0.0);
                for idx in idx_start..idx_end {
                    fvec[idx] = fc;
                }
            }
        }
    }

    // compute X_base
    let mut x_base = vec![0.0; n];
    for i in 0..n {
        let mut sum1 = 0.0;
        for col in 0..=k {
            let coeff = cross_coeff[col];
            sum1 += x_ks.get(&col).unwrap()[i] * coeff;
        }
        let mut sum2 = 0.0;
        for col in 0..k {
            let v1 = fast_cross.get(&col).unwrap()[i];
            let v2 = fast_cross.get(&(col + 1)).unwrap()[i];
            let c1 = cross_coeff[col];
            let c2 = cross_coeff[col + 1];
            sum2 += (v1 * c1 * v2 * c2).sqrt();
        }
        x_base[i] = sum1 + sum2;
    }

    let xbar = smooth_on_corners(base_corners, &x_base, 500.0, 0.001, "sum");
    xbar
}
