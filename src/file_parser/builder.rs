use crate::types::{Note, MapData, StarRatingResult};

use super::hit_objects::parse_hit_object_line;
use super::metadata::{read_column_count, read_overall_difficulty, read_metadata};
use super::reader::{read_file_lines};

/// Parser that processes .osu content into intermediate buffers, then builds MapData
pub struct Parser {
    file_path: String,
    od: f64,
    column_count: i32,
    columns: Vec<i32>,
    note_starts: Vec<i32>,
    note_ends: Vec<i32>,
    note_types: Vec<i32>,
}

impl Parser {
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
            od: -1.0,
            column_count: -1,
            columns: Vec::new(),
            note_starts: Vec::new(),
            note_ends: Vec::new(),
            note_types: Vec::new(),
        }
    }

    /// Read file from disk and process
    pub fn process(&mut self) -> StarRatingResult<()> {
        let lines = read_file_lines(&self.file_path)?;
        self.process_lines(&lines)
    }

    /// Parse raw .osu content provided as &str
    pub fn process_content(&mut self, content: &str) -> StarRatingResult<()> {
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        self.process_lines(&lines)
    }

    /// Common factor: apply parsing on a list of lines
    fn process_lines(&mut self, lines: &[String]) -> StarRatingResult<()> {
        let mut hit_objects_started = false;

        for line in lines {
            // Metadata
            read_metadata(line);

            // Column count
            let temp_cc = read_column_count(line);
            if temp_cc != -1 {
                self.column_count = temp_cc;
            }

            // Overall difficulty
            let temp_od = read_overall_difficulty(line);
            if temp_od != -1.0 {
                self.od = temp_od;
            }

            // Start of [HitObjects]
            if line.contains("[HitObjects]") {
                hit_objects_started = true;
                continue;
            }

            if hit_objects_started && !line.trim().is_empty() {
                parse_hit_object_line(
                    line,
                    self.column_count,
                    &mut self.columns,
                    &mut self.note_starts,
                    &mut self.note_ends,
                    &mut self.note_types,
                )?;
            }
        }

        Ok(())
    }

    pub fn get_parsed_data(&self) -> (i32, Vec<i32>, Vec<i32>, Vec<i32>, Vec<i32>, f64) {
        (
            self.column_count,
            self.columns.clone(),
            self.note_starts.clone(),
            self.note_ends.clone(),
            self.note_types.clone(),
            self.od,
        )
    }

    /// Build MapData from parsed buffers
    pub fn get_map_data(&self) -> StarRatingResult<MapData> {
        let mut notes = Vec::with_capacity(self.columns.len());

        for i in 0..self.columns.len() {
            let column = self.columns[i] as usize;
            let hit_time = self.note_starts[i] as i64;
            let tail_time = if self.note_types[i] == 128 { 
                self.note_ends[i] as i64 
            } else { 
                -1 
            };

            let note = if tail_time >= 0 {
                Note::long_note(column, hit_time, tail_time)
            } else {
                Note::simple(column, hit_time)
            };

            notes.push(note);
        }

        notes.sort_by(|a, b| match a.hit_time.cmp(&b.hit_time) {
            std::cmp::Ordering::Equal => a.column.cmp(&b.column),
            other => other,
        });

        let mut notes_by_column: Vec<Vec<Note>> = {
            let mut v = Vec::with_capacity(self.column_count as usize);
            for _ in 0..self.column_count.max(0) as usize { v.push(Vec::new()); }
            v
        };
        for note in &notes {
            if note.column < notes_by_column.len() {
                notes_by_column[note.column].push(*note);
            }
        }

        let long_notes: Vec<Note> = notes.iter()
            .filter(|note| note.is_long_note())
            .cloned()
            .collect();

        let mut tail_sequence = long_notes.clone();
        tail_sequence.sort_by(|a, b| a.tail_time.cmp(&b.tail_time));

        let mut long_notes_by_column: Vec<Vec<Note>> = {
            let mut v = Vec::with_capacity(self.column_count as usize);
            for _ in 0..self.column_count.max(0) as usize { v.push(Vec::new()); }
            v
        };
        for note in &long_notes {
            if note.column < long_notes_by_column.len() {
                long_notes_by_column[note.column].push(*note);
            }
        }

        let total_duration = notes.iter()
            .map(|note| note.hit_time.max(note.tail_time))
            .max()
            .unwrap_or(0) + 1;

        Ok(MapData {
            hit_leniency: 0.0,
            column_count: self.column_count as usize,
            total_duration,
            notes,
            notes_by_column,
            long_notes,
            tail_sequence,
            long_notes_by_column,
            overall_difficulty: self.od,
        })
    }
}


