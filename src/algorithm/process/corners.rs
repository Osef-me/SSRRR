use std::collections::HashSet;

/// Calcule les différents types de corners pour l'algorithme de star rating
/// 
/// # Arguments
/// * `t` - Temps total de la map
/// * `note_seq` - Séquence des notes (colonne, hit_time, tail_time)
/// 
/// # Returns
/// Un tuple contenant (all_corners, base_corners, a_corners)
pub fn get_corners(t: i64, note_seq: &Vec<(usize, i64, i64)>) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let mut corners_base: HashSet<i64> = HashSet::new();
    for &(_, h, t) in note_seq.iter() {
        corners_base.insert(h);
        if t >= 0 { corners_base.insert(t); }
    }
    let snapshot: Vec<i64> = corners_base.iter().cloned().collect();
    for s in snapshot.iter() {
        corners_base.insert(s + 501);
        corners_base.insert(s - 499);
        corners_base.insert(s + 1);
    }
    corners_base.insert(0);
    corners_base.insert(t);
    let mut corners_base_vec: Vec<i64> = corners_base.into_iter().filter(|&s| 0 <= s && s <= t).collect();
    corners_base_vec.sort_unstable();

    let mut corners_a: HashSet<i64> = HashSet::new();
    for &(_, h, t) in note_seq.iter() {
        corners_a.insert(h);
        if t >= 0 { corners_a.insert(t); }
    }
    let snapshot_a: Vec<i64> = corners_a.iter().cloned().collect();
    for s in snapshot_a.iter() {
        corners_a.insert(s + 1000);
        corners_a.insert(s - 1000);
    }
    corners_a.insert(0);
    corners_a.insert(t);
    let mut corners_a_vec: Vec<i64> = corners_a.into_iter().filter(|&s| 0 <= s && s <= t).collect();
    corners_a_vec.sort_unstable();

    let mut all_corners: Vec<i64> = corners_base_vec.iter().cloned().collect();
    all_corners.extend(corners_a_vec.iter().cloned());
    all_corners.sort_unstable();
    all_corners.dedup();

    (
        all_corners.into_iter().map(|v| v as f64).collect(),
        corners_base_vec.into_iter().map(|v| v as f64).collect(),
        corners_a_vec.into_iter().map(|v| v as f64).collect(),
    )
}
