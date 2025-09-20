use std::collections::HashMap;

// Use the actual parser from file_parser module
use crate::file_parser::Parser;

pub fn preprocess_file(
    file_path: &str,
    mod_name: &str,
) -> (
    f64,                           // x (hit leniency)
    usize,                         // k (nombre de colonnes)
    i64,                           // t (durée totale)
    Vec<(usize, i64, i64)>,        // note_seq
    Vec<Vec<(usize, i64, i64)>>,   // note_seq_by_column
    Vec<(usize, i64, i64)>,        // ln_seq
    Vec<(usize, i64, i64)>,        // tail_seq
    Vec<Vec<(usize, i64, i64)>>,   // ln_seq_by_column
) {
    let mut p_obj = Parser::new(file_path);
    p_obj.process();
    let (column_count, columns, note_starts, note_ends, note_types, od) = p_obj.get_parsed_data();

    // note_seq = (column, head_time, tail_time)
    let mut note_seq = Vec::new();
    for i in 0..columns.len() {
        let k = columns[i] as usize;
        let mut h = note_starts[i] as i64;
        let mut t = if note_types[i] == 128 { note_ends[i] as i64 } else { -1 };

        if mod_name == "DT" {
            h = ((h as f64) * 2.0 / 3.0).floor() as i64;
            t = if t >= 0 { ((t as f64) * 2.0 / 3.0).floor() as i64 } else { t };
        } else if mod_name == "HT" {
            h = ((h as f64) * 4.0 / 3.0).floor() as i64;
            t = if t >= 0 { ((t as f64) * 4.0 / 3.0).floor() as i64 } else { t };
        }

        note_seq.push((k, h, t));
    }

    // hit leniency x
    let mut x = 0.3 * ((64.5 - (od * 3.0).ceil()) / 500.0).sqrt();
    x = x.min(0.6 * (x - 0.09) + 0.09);

    // tri note_seq par head_time puis par column
    note_seq.sort_by(|a, b| match a.1.cmp(&b.1) {
        std::cmp::Ordering::Equal => a.0.cmp(&b.0),
        other => other,
    });

    // notes groupées par colonne
    let mut note_dict: HashMap<usize, Vec<(usize, i64, i64)>> = HashMap::new();
    for tup in &note_seq {
        note_dict.entry(tup.0).or_insert_with(Vec::new).push(*tup);
    }

    let mut note_seq_by_column: Vec<Vec<(usize, i64, i64)>> = note_dict.values().cloned().collect();
    note_seq_by_column.sort_by(|a, b| a[0].0.cmp(&b[0].0));

    // long notes (LN) = celles avec t >= 0
    let ln_seq: Vec<(usize, i64, i64)> = note_seq.iter().filter(|n| n.2 >= 0).cloned().collect();

    let mut tail_seq = ln_seq.clone();
    tail_seq.sort_by(|a, b| a.2.cmp(&b.2));

    let mut ln_dict: HashMap<usize, Vec<(usize, i64, i64)>> = HashMap::new();
    for tup in &ln_seq {
        ln_dict.entry(tup.0).or_insert_with(Vec::new).push(*tup);
    }

    let mut ln_seq_by_column: Vec<Vec<(usize, i64, i64)>> = ln_dict.values().cloned().collect();
    ln_seq_by_column.sort_by(|a, b| a[0].0.cmp(&b[0].0));

    // nombre de colonnes et temps total
    let k = column_count as usize;
    let t = note_seq.iter().map(|n| n.1.max(n.2)).max().unwrap_or(0) + 1;

    (
        x,
        k,
        t,
        note_seq,
        note_seq_by_column,
        ln_seq,
        tail_seq,
        ln_seq_by_column,
    )
}
