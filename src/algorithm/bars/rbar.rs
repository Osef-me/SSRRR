use crate::algorithm::smoothing::{smooth_on_corners, SmoothMode};
use crate::algorithm::utils::find_next_note_in_column;
use crate::types::Note;

/// Computes Rbar values for the star rating algorithm
/// 
/// # Arguments
/// * `_k` - Number of columns (unused)
/// * `_t` - Total map time (unused)
/// * `x` - Difficulty parameter
/// * `notes_by_column` - Notes organized by column
/// * `tail_sequence` - Long note tail sequence
/// * `base_corners` - Reference time points
/// 
/// # Returns
/// Vector of Rbar values
pub fn compute_rbar(
    _k: usize,
    _t: i64,
    x: f64,
    notes_by_column: &[Vec<Note>],
    tail_sequence: &[Note],
    base_corners: &[f64]
) -> Vec<f64> {
    let n = base_corners.len();
    let mut i_arr = vec![0.0; n];
    let mut r_step = vec![0.0; n];

    let mut times_by_column: Vec<Vec<i64>> = Vec::with_capacity(notes_by_column.len());
    for col in notes_by_column.iter() {
        times_by_column.push(col.iter().map(|note| note.hit_time).collect());
    }

    let mut i_list: Vec<f64> = Vec::with_capacity(tail_sequence.len());
    for i in 0..tail_sequence.len() {
        let note = &tail_sequence[i];
        let nxt = find_next_note_in_column((note.column, note.hit_time, note.tail_time), &times_by_column[note.column], notes_by_column);
        let h_j = nxt.1;
        let i_h = 0.001 * ((note.tail_time - note.hit_time - 80).abs() as f64) / x;
        let i_t = 0.001 * ((h_j - note.tail_time - 80).abs() as f64) / x;
        i_list.push(2.0 / (2.0 + (-5.0 * (i_h - 0.75)).exp() + (-5.0 * (i_t - 0.75)).exp()));
    }

    for i in 0..tail_sequence.len().saturating_sub(1) {
        let t_start = tail_sequence[i].tail_time;
        let t_end = tail_sequence[i + 1].tail_time;
        let left_idx = base_corners.partition_point(|&v| v < t_start as f64);
        let right_idx = base_corners.partition_point(|&v| v < t_end as f64);
        if left_idx >= right_idx { continue; }
        for idx in left_idx..right_idx {
            i_arr[idx] = 1.0 + i_list[i];
        }
        let delta_r = 0.001 * ((tail_sequence[i + 1].tail_time - tail_sequence[i].tail_time) as f64);
        for idx in left_idx..right_idx {
            r_step[idx] = 0.08 * delta_r.powf(-0.5_f64) * x.powf(-1.0) * (1.0 + 0.8 * (i_list[i] + i_list[i + 1]));
        }
    }

    smooth_on_corners(base_corners, &r_step, 500.0, 0.001, SmoothMode::Sum)
}


