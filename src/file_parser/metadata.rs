fn string_to_int(s: &str) -> i32 {
    s.parse::<f64>().unwrap_or(0.0) as i32
}

/// Currently a no-op placeholder to document metadata handling responsibility
pub fn read_metadata(line: &str) {
    if line.contains("[Metadata]") {
        // Not used currently
    }
}

pub fn read_overall_difficulty(line: &str) -> f64 {
    if line.contains("OverallDifficulty:") {
        let temp = line.trim();
        if let Some(pos) = temp.find(':') {
            let od_str = &temp[pos + 1..];
            return od_str.parse::<f64>().unwrap_or(-1.0);
        }
    }
    -1.0
}

pub fn read_column_count(line: &str) -> i32 {
    if line.contains("CircleSize:") {
        let temp = line.trim();
        let column_count = temp.chars().last().unwrap_or('0');
        let column_count_str = if column_count == '0' { "10" } else { &column_count.to_string() };
        return string_to_int(column_count_str);
    }
    -1
}


