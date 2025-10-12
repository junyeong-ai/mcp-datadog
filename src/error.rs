use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatadogError {
    #[error("API request failed: {0}")]
    ApiError(String),

    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("Invalid date format: {0}")]
    DateParseError(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Rate limit exceeded")]
    RateLimitError,

    #[error("Timeout occurred")]
    TimeoutError,
}

pub type Result<T> = std::result::Result<T, DatadogError>;