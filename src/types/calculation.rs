/// Star rating calculation result
#[derive(Debug, Clone)]
pub struct StarRating {
    /// Final star rating value
    pub rating: f64,
    /// Detailed calculation components
    pub components: StarRatingComponents,
}

/// Detailed components of star rating calculation
#[derive(Debug, Clone)]
pub struct StarRatingComponents {
    /// Speed values
    pub speed_values: Vec<f64>,
    /// Tech values
    pub tech_values: Vec<f64>,
    /// Difficulty values
    pub difficulty_values: Vec<f64>,
    /// Effective weights
    pub effective_weights: Vec<f64>,
    /// 93rd percentile
    pub percentile_93: f64,
    /// 83rd percentile
    pub percentile_83: f64,
    /// Weighted mean
    pub weighted_mean: f64,
}

impl StarRatingComponents {
    /// Creates new components
    pub fn new(
        speed_values: Vec<f64>,
        tech_values: Vec<f64>,
        difficulty_values: Vec<f64>,
        effective_weights: Vec<f64>,
        percentile_93: f64,
        percentile_83: f64,
        weighted_mean: f64,
    ) -> Self {
        Self {
            speed_values,
            tech_values,
            difficulty_values,
            effective_weights,
            percentile_93,
            percentile_83,
            weighted_mean,
        }
    }
}

/// Input parameters for star rating calculation
#[derive(Debug, Clone)]
pub struct CalculationInput {
    /// Hit leniency
    pub hit_leniency: f64,
    /// Number of columns
    pub column_count: usize,
    /// Total duration
    pub total_duration: i64,
    /// All notes
    pub notes: Vec<(usize, i64, i64)>,
    /// Notes by column
    pub notes_by_column: Vec<Vec<(usize, i64, i64)>>,
    /// Long notes
    pub long_notes: Vec<(usize, i64, i64)>,
    /// Tail sequence
    pub tail_sequence: Vec<(usize, i64, i64)>,
}

impl CalculationInput {
    /// Creates input from MapData
    pub fn from_map_data(map_data: &crate::types::map::MapData) -> Self {
        Self {
            hit_leniency: map_data.hit_leniency,
            column_count: map_data.column_count,
            total_duration: map_data.total_duration,
            notes: map_data.notes.iter()
                .map(|note| (note.column, note.hit_time, note.tail_time))
                .collect(),
            notes_by_column: map_data.notes_by_column.iter()
                .map(|col_notes| col_notes.iter()
                    .map(|note| (note.column, note.hit_time, note.tail_time))
                    .collect())
                .collect(),
            long_notes: map_data.long_notes.iter()
                .map(|note| (note.column, note.hit_time, note.tail_time))
                .collect(),
            tail_sequence: map_data.tail_sequence.iter()
                .map(|note| (note.column, note.hit_time, note.tail_time))
                .collect(),
        }
    }
}

/// Intermediate calculation results
#[derive(Debug, Clone)]
pub struct CalculationState {
    /// All corners
    pub all_corners: Vec<f64>,
    /// Base corners
    pub base_corners: Vec<f64>,
    /// A corners
    pub a_corners: Vec<f64>,
    /// Key usage
    pub key_usage: std::collections::HashMap<usize, Vec<bool>>,
    /// Active columns
    pub active_columns: Vec<Vec<usize>>,
    /// Key usage 400ms
    pub key_usage_400: std::collections::HashMap<usize, Vec<f64>>,
    /// Anchor values
    pub anchor: Vec<f64>,
}

/// Bar calculation results
#[derive(Debug, Clone)]
pub struct BarResults {
    /// J bar values
    pub jbar: Vec<f64>,
    /// X bar values
    pub xbar: Vec<f64>,
    /// P bar values
    pub pbar: Vec<f64>,
    /// A bar values
    pub abar: Vec<f64>,
    /// R bar values
    pub rbar: Vec<f64>,
    /// C array
    pub c_arr: Vec<f64>,
    /// Ks array
    pub ks_arr: Vec<f64>,
}

/// Final calculation values
#[derive(Debug, Clone)]
pub struct FinalValues {
    /// S values
    pub s_all: Vec<f64>,
    /// T values
    pub t_all: Vec<f64>,
    /// D values
    pub d_all: Vec<f64>,
}
