use std::collections::HashMap;

/// Représentation sparse des corps de long notes
/// 
/// # Arguments
/// * `ln_seq` - Séquence des long notes (colonne, hit_time, tail_time)
/// * `t` - Temps total de la map
/// 
/// # Returns
/// Tuple contenant (points, cumsum, values) pour la représentation sparse
pub fn ln_bodies_count_sparse_representation(
    ln_seq: &Vec<(usize, i64, i64)>,
    t: i64
) -> (Vec<i64>, Vec<f64>, Vec<f64>) {
    let mut diff: HashMap<i64, f64> = HashMap::new();
    for &(_k, h, tail) in ln_seq.iter() {
        let t0 = (h + 60).min(tail);
        let t1 = (h + 120).min(tail);
        *diff.entry(t0).or_insert(0.0) += 1.3;
        *diff.entry(t1).or_insert(0.0) += -1.3 + 1.0;
        *diff.entry(tail).or_insert(0.0) += -1.0;
    }
    let mut points: Vec<i64> = diff.keys().cloned().collect();
    points.push(0);
    points.push(t);
    points.sort_unstable();
    points.dedup();

    let mut values: Vec<f64> = Vec::with_capacity(points.len().saturating_sub(1));
    let mut cumsum: Vec<f64> = Vec::with_capacity(points.len());
    cumsum.push(0.0);
    let mut curr = 0.0;
    for i in 0..(points.len().saturating_sub(1)) {
        let t = points[i];
        if let Some(dv) = diff.get(&t) {
            curr += *dv;
        }
        let v = curr.min(2.5 + 0.5 * curr);
        values.push(v);
        let seg_length = (points[i + 1] - points[i]) as f64;
        let last = *cumsum.last().unwrap();
        cumsum.push(last + seg_length * v);
    }
    (points, cumsum, values)
}

/// Calcule la somme des valeurs LN sur un intervalle
/// 
/// # Arguments
/// * `a` - Temps de début
/// * `b` - Temps de fin
/// * `ln_rep` - Représentation sparse des LN (points, cumsum, values)
/// 
/// # Returns
/// Valeur de la somme sur l'intervalle
pub fn ln_sum(a: f64, b: f64, ln_rep: &(Vec<i64>, Vec<f64>, Vec<f64>)) -> f64 {
    let (points, cumsum, values) = ln_rep;
    // points are i64; find i = bisect_right(points, a) - 1
    let i = points.partition_point(|&p| (p as f64) <= a);
    let i = if i == 0 { 0usize } else { i - 1 };
    let j = points.partition_point(|&p| (p as f64) <= b);
    let j = if j == 0 { 0usize } else { j - 1 };
    let mut total = 0.0;
    if i == j {
        total = (b - a) * values[i];
    } else {
        total += (points[i + 1] as f64 - a) * values[i];
        if j > i + 1 {
            total += cumsum[j] - cumsum[i + 1];
        }
        total += (b - points[j] as f64) * values[j];
    }
    total
}
