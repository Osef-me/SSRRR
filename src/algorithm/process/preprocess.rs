
// Use the actual parser from file_parser module
use crate::file_parser::Parser;
use crate::types::{MapData, StarRatingResult};


/// Parse un fichier .osu et retourne les données sous forme de MapData
pub fn preprocess_file(
    file_path: &str,
    mod_name: &str,
) -> StarRatingResult<MapData> {
    let mut parser = Parser::new(file_path);
    parser.process()?;
    let mut map_data = parser.get_map_data()?;

    // Appliquer les mods (DT/HT)
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

    // Recalculer les données organisées après application des mods
    map_data.notes.sort_by(|a, b| match a.hit_time.cmp(&b.hit_time) {
        std::cmp::Ordering::Equal => a.column.cmp(&b.column),
        other => other,
    });

    // Reorganiser par colonne
    map_data.notes_by_column = vec![Vec::new(); map_data.column_count];
    for note in &map_data.notes {
        if note.column < map_data.notes_by_column.len() {
            map_data.notes_by_column[note.column].push(*note);
        }
    }

    // Recalculer les long notes
    map_data.long_notes = map_data.notes.iter()
        .filter(|note| note.is_long_note())
        .cloned()
        .collect();

    // Recalculer la séquence des queues
    map_data.tail_sequence = map_data.long_notes.clone();
    map_data.tail_sequence.sort_by(|a, b| a.tail_time.cmp(&b.tail_time));

    // Reorganiser les long notes par colonne
    map_data.long_notes_by_column = vec![Vec::new(); map_data.column_count];
    for note in &map_data.long_notes {
        if note.column < map_data.long_notes_by_column.len() {
            map_data.long_notes_by_column[note.column].push(*note);
        }
    }

    // Calculer hit leniency
    let mut x = 0.3 * ((64.5 - (map_data.overall_difficulty * 3.0).ceil()) / 500.0).sqrt();
    x = x.min(0.6 * (x - 0.09) + 0.09);
    map_data.hit_leniency = x;

    // Recalculer la durée totale
    map_data.total_duration = map_data.notes.iter()
        .map(|note| note.hit_time.max(note.tail_time))
        .max()
        .unwrap_or(0) + 1;

    Ok(map_data)
}
