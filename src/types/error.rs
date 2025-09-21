use std::fmt;

/// Main application error types
#[derive(Debug)]
pub enum StarRatingError {
    /// File reading error
    FileError(std::io::Error),
    /// Parsing error
    ParseError(String),
    /// Calculation error
    CalculationError(String),
    /// Missing data error
    MissingData(String),
    /// Invalid format error
    InvalidFormat(String),
}

impl fmt::Display for StarRatingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StarRatingError::FileError(e) => write!(f, "File error: {}", e),
            StarRatingError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            StarRatingError::CalculationError(msg) => write!(f, "Calculation error: {}", msg),
            StarRatingError::MissingData(msg) => write!(f, "Missing data: {}", msg),
            StarRatingError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
        }
    }
}

impl std::error::Error for StarRatingError {}

/// Automatic conversion from io::Error
impl From<std::io::Error> for StarRatingError {
    fn from(error: std::io::Error) -> Self {
        StarRatingError::FileError(error)
    }
}

/// Standard result type for the application
pub type StarRatingResult<T> = Result<T, StarRatingError>;

/// Parsing-specific errors
#[derive(Debug)]
pub enum ParseError {
    /// File not found
    FileNotFound(String),
    /// Invalid line
    InvalidLine(String),
    /// Missing section
    MissingSection(String),
    /// Invalid value
    InvalidValue(String, String),
    /// Insufficient data
    InsufficientData(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::FileNotFound(path) => write!(f, "File not found: {}", path),
            ParseError::InvalidLine(line) => write!(f, "Invalid line: {}", line),
            ParseError::MissingSection(section) => write!(f, "Missing section: {}", section),
            ParseError::InvalidValue(field, value) => write!(f, "Invalid value for {}: {}", field, value),
            ParseError::InsufficientData(msg) => write!(f, "Insufficient data: {}", msg),
        }
    }
}

impl std::error::Error for ParseError {}

/// Calculation-specific errors
#[derive(Debug)]
pub enum CalculationError {
    /// Division by zero
    DivisionByZero(String),
    /// Unexpected negative value
    NegativeValue(String, f64),
    /// Empty data
    EmptyData(String),
    /// Index out of bounds
    IndexOutOfBounds(String, usize, usize),
    /// Invalid number (NaN or infinity)
    InvalidNumber(String, f64),
}

impl fmt::Display for CalculationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalculationError::DivisionByZero(context) => write!(f, "Division by zero in: {}", context),
            CalculationError::NegativeValue(context, value) => write!(f, "Unexpected negative value in {}: {}", context, value),
            CalculationError::EmptyData(context) => write!(f, "Empty data in: {}", context),
            CalculationError::IndexOutOfBounds(context, index, max) => write!(f, "Index {} out of bounds in {} (max: {})", index, context, max),
            CalculationError::InvalidNumber(context, value) => write!(f, "Invalid number in {}: {}", context, value),
        }
    }
}

impl std::error::Error for CalculationError {}

/// Conversion from ParseError to StarRatingError
impl From<ParseError> for StarRatingError {
    fn from(error: ParseError) -> Self {
        StarRatingError::ParseError(error.to_string())
    }
}

/// Conversion from CalculationError to StarRatingError
impl From<CalculationError> for StarRatingError {
    fn from(error: CalculationError) -> Self {
        StarRatingError::CalculationError(error.to_string())
    }
}
