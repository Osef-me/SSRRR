/// Type definitions for star rating calculation
/// 
/// This module contains all the data structures used throughout the application,
/// organized by functionality for better maintainability.

pub mod note;
pub mod map;
pub mod calculation;
pub mod error;

// Re-export commonly used types
pub use note::*;
pub use map::*;
pub use calculation::*;
pub use error::*;
