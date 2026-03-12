use thiserror::Error;

#[derive(Error, Debug)]
pub enum SddNavigatorError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Scan error: {0}")]
    Scan(String),
    
    #[error("Path not found: {0}")]
    PathNotFound(String),
    
    #[error("Invalid annotation format: {0}")]
    InvalidAnnotation(String),
    
    #[error("Coverage calculation error: {0}")]
    CoverageCalculation(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, SddNavigatorError>;
