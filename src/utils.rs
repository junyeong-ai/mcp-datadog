use crate::error::{DatadogError, Result};
use chrono::{DateTime, Utc};
use interim::{Dialect, parse_date_string};

/// Parse a time expression into a Unix timestamp
/// Supports:
/// - Natural language: "1 hour ago", "yesterday", "last week"
/// - ISO 8601: "2024-01-01T00:00:00Z"
/// - Unix timestamp: "1704067200"
/// - Special keywords: "now"
pub fn parse_time(input: &str) -> Result<i64> {
    // Handle special case
    if input.trim().to_lowercase() == "now" {
        return Ok(Utc::now().timestamp());
    }

    // Try parsing as Unix timestamp first
    if let Ok(timestamp) = input.parse::<i64>() {
        return Ok(timestamp);
    }

    // Try natural language parsing with interim
    if let Ok(dt) = parse_date_string(input, Utc::now(), Dialect::Us) {
        return Ok(dt.timestamp());
    }

    // Try ISO 8601 format
    if let Ok(dt) = DateTime::parse_from_rfc3339(input) {
        return Ok(dt.timestamp());
    }

    Err(DatadogError::DateParseError(format!(
        "Unable to parse time expression: '{}'",
        input
    )))
}

/// Convert timestamp to human-readable format
pub fn format_timestamp(timestamp: i64) -> String {
    if let Some(dt) = DateTime::from_timestamp(timestamp, 0) {
        dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    } else {
        format!("Invalid timestamp: {}", timestamp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_time_unix() {
        let result = parse_time("1704067200");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1704067200);
    }

    #[test]
    fn test_parse_time_now() {
        let result = parse_time("now");
        assert!(result.is_ok());
        let now = Utc::now().timestamp();
        assert!((result.unwrap() - now).abs() < 2);
    }

    #[test]
    fn test_parse_time_natural() {
        let result = parse_time("yesterday");
        assert!(result.is_ok());
    }
}
