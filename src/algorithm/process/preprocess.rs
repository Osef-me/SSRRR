
// Use the actual parser from file_parser module
use crate::file_parser::Parser;
use crate::types::{MapData, StarRatingResult};
use super::normalize::{apply_mods, rebuild_groupings, recompute_hit_leniency, recompute_total_duration};
use std::fs;


/// Parse a .osu file and return data as MapData
pub fn preprocess_file(
    file_path: &str,
    mod_name: &str,
) -> StarRatingResult<MapData> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| crate::types::ParseError::FileNotFound(format!("{}: {}", file_path, e)))?;
    preprocess(&content, mod_name)
}

/// Parse in-memory .osu content and return data as MapData
pub fn preprocess(
    osu_content: &str,
    mod_name: &str,
) -> StarRatingResult<MapData> {
    let mut parser = Parser::new("");
    parser.process_content(osu_content)?;
    let mut map_data = parser.get_map_data()?;
    apply_mods(&mut map_data, mod_name);
    rebuild_groupings(&mut map_data);
    recompute_hit_leniency(&mut map_data);
    recompute_total_duration(&mut map_data);

    Ok(map_data)
}

// supprimé: alias inutile
