pub mod algorithm;
pub mod file_parser;
pub mod params;
pub mod types;

// Public re-exports for simplified API
pub use algorithm::process::preprocess::{preprocess_file, preprocess};