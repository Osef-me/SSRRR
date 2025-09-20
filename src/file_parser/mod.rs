use std::fs::File;
use std::io::{BufRead, BufReader};

fn string_to_int(s: &str) -> i32 {
    s.parse::<f64>().unwrap_or(0.0) as i32
}

// Parser Class that can be used on other class.
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
    pub fn process(&mut self) {
        // Ouvre le fichier et lit toutes les lignes en mémoire
        let file = File::open(&self.file_path).expect("Failed to open file");
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();

        let mut hit_objects_started = false;

        for line in &lines {
            // Lire les métadonnées (si besoin)
            self.read_metadata(line);

            // Lire le nombre de colonnes
            let temp_cc = self.read_column_count(line);
            if temp_cc != -1 {
                self.column_count = temp_cc;
            }

            // Lire la difficulté globale
            let temp_od = self.read_overall_difficulty(line);
            if temp_od != -1.0 {
                self.od = temp_od;
            }

            // Commence à parser les notes après la section [HitObjects]
            if line.contains("[HitObjects]") {
                hit_objects_started = true;
                continue;
            }

            if hit_objects_started && !line.trim().is_empty() {
                self.parse_hit_object(line, self.column_count);
            }
        }
    }
    
    // Read metadata from .osu file.
    fn read_metadata(&self, line: &str) {
        if line.contains("[Metadata]") {
            // Skip metadata section - not needed for parsing
        }
    }

    fn read_overall_difficulty(&self, line: &str) -> f64 {
        if line.contains("OverallDifficulty:") {
            let temp = line.trim();
            if let Some(pos) = temp.find(':') {
                let od_str = &temp[pos + 1..];
                return od_str.parse::<f64>().unwrap_or(-1.0);
            }
        }
        -1.0
    }

    // Read mode: key count.
    fn read_column_count(&self, line: &str) -> i32 {
        if line.contains("CircleSize:") {
            let temp = line.trim();
            let column_count = temp.chars().last().unwrap_or('0');
            let column_count_str = if column_count == '0' { "10" } else { &column_count.to_string() };
            return string_to_int(column_count_str);
        }
        -1
    }


    // Helper function for read_note().
    // Store all note information in 4 arrays: column, type, start, end.
    // If note_end is 0, the note is a single note, otherwise a hold.
    fn parse_hit_object(&mut self, object_line: &str, column_count: i32) {
        let params: Vec<&str> = object_line.split(',').collect();
        if params.len() < 6 {
            return;
        }

        let x_pos = string_to_int(params[0]);
        let column_width = 512 / column_count;
        let column = x_pos / column_width;
        self.columns.push(column);

        let note_start = string_to_int(params[2]);
        self.note_starts.push(note_start);

        // 1: single note
        // 128: Hold(LN)
        let note_type = string_to_int(params[3]);
        self.note_types.push(note_type);

        let last_param_chunk: Vec<&str> = params[5].split(':').collect();
        let note_end = string_to_int(last_param_chunk[0]);
        self.note_ends.push(note_end);
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
}
