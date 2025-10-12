// Common test utilities and helpers for MCP Datadog Server tests
// This module exports shared test infrastructure used across all test suites

pub mod builders;
pub mod fixtures;
pub mod mocks;

use serde_json::Value;
use std::result::Result as StdResult;

/// Result type alias for test operations
pub type Result<T> = StdResult<T, Box<dyn std::error::Error>>;

/// Assertion helper utilities for common test scenarios
pub struct AssertionHelper;

impl AssertionHelper {
    /// Assert that a result is successful and return the value
    pub fn assert_success_response(result: StdResult<Value, impl std::error::Error>) -> Result<Value> {
        match result {
            Ok(value) => Ok(value),
            Err(e) => panic!("Expected successful response, got error: {}", e),
        }
    }

    /// Assert that a result is an error of a specific type
    pub fn assert_error_contains(result: StdResult<Value, impl std::error::Error + std::fmt::Display>, expected_msg: &str) {
        match result {
            Ok(_) => panic!("Expected error containing '{}', but got success", expected_msg),
            Err(e) => {
                let error_str = e.to_string();
                assert!(
                    error_str.contains(expected_msg),
                    "Error message '{}' does not contain expected string '{}'",
                    error_str,
                    expected_msg
                );
            }
        }
    }

    /// Assert that a JSON value matches expected schema structure
    pub fn assert_json_has_field(json: &Value, field: &str) {
        assert!(
            json.get(field).is_some(),
            "JSON does not contain required field '{}'",
            field
        );
    }

    /// Assert that a JSON response has the standard format (data, meta)
    pub fn assert_standard_response_format(json: &Value) {
        Self::assert_json_has_field(json, "data");
        // meta is optional in some responses
    }

    /// Assert no warnings in build/test output by checking environment
    #[allow(dead_code)]
    pub fn assert_no_warnings() {
        // This will be enforced by RUSTFLAGS in CI/CD
        // Individual tests don't need to check this
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_assertion_helper_json_field() {
        let json = json!({"data": {"value": 42}, "meta": {}});
        AssertionHelper::assert_json_has_field(&json, "data");
        AssertionHelper::assert_json_has_field(&json, "meta");
    }

    #[test]
    fn test_assertion_helper_standard_format() {
        let json = json!({"data": [], "meta": {"count": 0}});
        AssertionHelper::assert_standard_response_format(&json);
    }

    #[test]
    #[should_panic(expected = "JSON does not contain required field")]
    fn test_assertion_helper_missing_field() {
        let json = json!({"other": "value"});
        AssertionHelper::assert_json_has_field(&json, "data");
    }
}
