use crate::types::MapData;

/// Applique les mods (DT/HT) sur les notes
pub fn apply_mods(map_data: &mut MapData, mod_name: &str) {
    if mod_name == "DT" {
        for note in &mut map_data.notes {
            note.hit_time = ((note.hit_time as f64) * 2.0 / 3.0).floor() as i64;
            if note.is_long_note() {
                note.tail_time = ((note.tail_time as f64) * 2.0 / 3.0).floor() as i64;
            }
        }
    } else if mod_name == "HT" {
        for note in &mut map_data.notes {
            note.hit_time = ((note.hit_time as f64) * 4.0 / 3.0).floor() as i64;
            if note.is_long_note() {
                note.tail_time = ((note.tail_time as f64) * 4.0 / 3.0).floor() as i64;
            }
        }
    }
}

/// Sorts and rebuilds per-column groupings and LN sequences
pub fn rebuild_groupings(map_data: &mut MapData) {
    // Trier les notes par temps de hit puis par colonne
    map_data.notes.sort_by(|a, b| match a.hit_time.cmp(&b.hit_time) {
        std::cmp::Ordering::Equal => a.column.cmp(&b.column),
        other => other,
    });

    // Rebuild per-column grouping
    map_data.notes_by_column = vec![Vec::new(); map_data.column_count];
    for note in &map_data.notes {
        if note.column < map_data.notes_by_column.len() {
            map_data.notes_by_column[note.column].push(*note);
        }
    }

    // Recompute long notes
    map_data.long_notes = map_data.notes.iter()
        .filter(|note| note.is_long_note())
        .cloned()
        .collect();

    // Recompute tail sequence
    map_data.tail_sequence = map_data.long_notes.clone();
    map_data.tail_sequence.sort_by(|a, b| a.tail_time.cmp(&b.tail_time));

    // Rebuild long notes per column
    map_data.long_notes_by_column = vec![Vec::new(); map_data.column_count];
    for note in &map_data.long_notes {
        if note.column < map_data.long_notes_by_column.len() {
            map_data.long_notes_by_column[note.column].push(*note);
        }
    }
}

/// Recomputes hit leniency from OD
pub fn recompute_hit_leniency(map_data: &mut MapData) {
    let mut x = 0.3 * ((64.5 - (map_data.overall_difficulty * 3.0).ceil()) / 500.0).sqrt();
    x = x.min(0.6 * (x - 0.09) + 0.09);
    map_data.hit_leniency = x;
}

/// Recomputes total map duration
pub fn recompute_total_duration(map_data: &mut MapData) {
    map_data.total_duration = map_data.notes.iter()
        .map(|note| note.hit_time.max(note.tail_time))
        .max()
        .unwrap_or(0) + 1;
}


