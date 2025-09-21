use crate::types::note::Note;

/// Parsed osu! map data
#[derive(Debug, Clone)]
pub struct MapData {
    /// Hit leniency parameter
    pub hit_leniency: f64,
    /// Number of columns in the map
    pub column_count: usize,
    /// Total duration in milliseconds
    pub total_duration: i64,
    /// All notes sorted by hit time
    pub notes: Vec<Note>,
    /// Notes organized by column
    pub notes_by_column: Vec<Vec<Note>>,
    /// Long notes only
    pub long_notes: Vec<Note>,
    /// Tail sequence sorted by end time
    pub tail_sequence: Vec<Note>,
    /// Long notes organized by column
    pub long_notes_by_column: Vec<Vec<Note>>,
    /// Overall difficulty of the map
    pub overall_difficulty: f64,
}

impl MapData {
    /// Creates a new empty MapData
    pub fn new() -> Self {
        Self {
            hit_leniency: 0.0,
            column_count: 0,
            total_duration: 0,
            notes: Vec::new(),
            notes_by_column: Vec::new(),
            long_notes: Vec::new(),
            tail_sequence: Vec::new(),
            long_notes_by_column: Vec::new(),
            overall_difficulty: 0.0,
        }
    }

    /// Returns the total number of notes
    pub fn note_count(&self) -> usize {
        self.notes.len()
    }

    /// Returns the number of long notes
    pub fn long_note_count(&self) -> usize {
        self.long_notes.len()
    }

    /// Returns the number of simple notes
    pub fn simple_note_count(&self) -> usize {
        self.note_count() - self.long_note_count()
    }
}

impl Default for MapData {
    fn default() -> Self {
        Self::new()
    }
}
