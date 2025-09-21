use std::collections::HashMap;
// use crate::errors::{StarRatingResult, CalculationError};

/// Calcule les valeurs d'anchor basées sur l'utilisation des touches
/// 
/// # Arguments
/// * `k` - Nombre de colonnes
/// * `key_usage_400` - Utilisation pondérée des touches avec fenêtre 400ms
/// * `base_corners` - Points de référence temporels
/// 
/// # Returns
/// Vecteur des valeurs d'anchor pour chaque point temporel
pub fn compute_anchor(
    k: usize,
    key_usage_400: &HashMap<usize, Vec<f64>>,
    base_corners: &[f64]
) -> Vec<f64> {
    let n = base_corners.len();
    let mut anchor = vec![0.0; n];
    for idx in 0..n {
        // collect counts per column at this base corner
        let mut counts: Vec<f64> = (0..k).map(|col| {
            key_usage_400.get(&col).map(|v| v[idx]).unwrap_or(0.0)
        }).collect();
        // sort descending (counts[::-1].sort() in python after reversing)
        counts.sort_by(|a, b| b.partial_cmp(a).expect("Valeurs finies attendues"));
        // filter nonzero
        let nonzero: Vec<f64> = counts.into_iter().filter(|&x| x != 0.0).collect();
        if nonzero.len() > 1 {
            let mut walk = 0.0;
            let mut max_walk = 0.0;
            for i in 0..(nonzero.len() - 1) {
                let a = nonzero[i];
                let b = nonzero[i + 1];
                let term = a * (1.0 - 4.0 * (0.5 - b / a).powi(2));
                walk += term;
                max_walk += a;
            }
            anchor[idx] = if max_walk.abs() > 0.0 { walk / max_walk } else { 0.0 };
        } else {
            anchor[idx] = 0.0;
        }
    }
    // anchor = 1 + np.minimum(anchor-0.18, 5*(anchor-0.22)**3)
    for v in anchor.iter_mut() {
        let a = *v - 0.18;
        let b = 5.0 * (*v - 0.22).powi(3);
        *v = 1.0 + a.min(b);
    }
    anchor
}
