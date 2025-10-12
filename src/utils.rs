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

    #[test]
    fn test_parse_time_iso8601() {
        let result = parse_time("2024-01-01T00:00:00Z");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1704067200);
    }

    #[test]
    fn test_parse_time_natural_hour_ago() {
        let result = parse_time("1 hour ago");
        assert!(result.is_ok());
        let expected = Utc::now().timestamp() - 3600;
        assert!((result.unwrap() - expected).abs() < 5);
    }

    #[test]
    fn test_parse_time_natural_days_ago() {
        let result = parse_time("2 days ago");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_time_invalid() {
        let result = parse_time("invalid time string xyz");
        assert!(result.is_err());
        match result.unwrap_err() {
            DatadogError::DateParseError(_) => {}
            _ => panic!("Expected DateParseError"),
        }
    }

    #[test]
    fn test_format_timestamp_valid() {
        let formatted = format_timestamp(1704067200);
        assert!(formatted.contains("2024-01-01"));
        assert!(formatted.contains("UTC"));
    }

    #[test]
    fn test_format_timestamp_negative() {
        let formatted = format_timestamp(-1);
        // Negative timestamps can be valid (before 1970), but very large negative values are invalid
        assert!(formatted.contains("1969") || formatted.contains("Invalid"));
    }

    #[test]
    fn test_parse_time_case_insensitive_now() {
        assert!(parse_time("NOW").is_ok());
        assert!(parse_time("Now").is_ok());
        assert!(parse_time("  now  ").is_ok());
    }
}
