use std::fs::File;
use std::io::{BufRead, BufReader};
use crate::types::{StarRatingResult, ParseError};

pub fn read_file_lines(path: &str) -> StarRatingResult<Vec<String>> {
    let file = File::open(path)
        .map_err(|e| ParseError::FileNotFound(format!("{}: {}", path, e)))?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| ParseError::InvalidLine(format!("Read error: {}", e)))?;
    Ok(lines)
}


