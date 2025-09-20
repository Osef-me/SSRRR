pub fn find_next_note_in_column(
    note: (usize, i64, i64),
    times: &Vec<i64>,
    note_seq_by_column: &Vec<Vec<(usize, i64, i64)>>
) -> (usize, i64, i64) {
    let (k, h, _t) = note;
    // bisect_left
    let idx = times.partition_point(|&val| val < h);
    if idx + 1 < note_seq_by_column[k].len() {
        note_seq_by_column[k][idx + 1]
    } else {
        (0, 1_000_000_000, 1_000_000_000)
    }
}