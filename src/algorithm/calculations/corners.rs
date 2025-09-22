use std::collections::HashSet;
use crate::types::Note;

/// Computes the different types of corners for the star rating algorithm
/// 
/// # Arguments
/// * `t` - Total map time
/// * `notes` - Note sequence
/// 
/// # Returns
/// Returns a tuple (all_corners, base_corners, a_corners)
pub fn get_corners(t: i64, notes: &[Note]) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let mut corners_base: HashSet<i64> = HashSet::new();
    for note in notes.iter() {
        corners_base.insert(note.hit_time);
        if note.tail_time >= 0 { corners_base.insert(note.tail_time); }
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
    for note in notes.iter() {
        corners_a.insert(note.hit_time);
        if note.tail_time >= 0 { corners_a.insert(note.tail_time); }
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

