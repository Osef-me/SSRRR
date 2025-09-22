use crate::types::{StarRatingResult, ParseError};

fn string_to_int(s: &str) -> i32 {
    s.parse::<f64>().unwrap_or(0.0) as i32
}

/// Parse one [HitObjects] line and push into buffers
pub fn parse_hit_object_line(
    object_line: &str,
    column_count: i32,
    columns: &mut Vec<i32>,
    note_starts: &mut Vec<i32>,
    note_ends: &mut Vec<i32>,
    note_types: &mut Vec<i32>,
) -> StarRatingResult<()> {
    let params: Vec<&str> = object_line.split(',').collect();
    if params.len() < 6 {
        return Err(ParseError::InsufficientData(
            format!("Invalid hit object line: {}", object_line)
        ).into());
    }

    let x_pos = string_to_int(params[0]);
    let column_width = 512 / column_count;
    let column = x_pos / column_width;
    columns.push(column);

    let note_start = string_to_int(params[2]);
    note_starts.push(note_start);

    let note_type = string_to_int(params[3]);
    note_types.push(note_type);

    let last_param_chunk: Vec<&str> = params[5].split(':').collect();
    let note_end = string_to_int(last_param_chunk[0]);
    note_ends.push(note_end);

    Ok(())
}


