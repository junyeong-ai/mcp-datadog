// Test data builders for constructing mock responses
// These builders provide a fluent API for creating test fixtures

use serde_json::{json, Value};
use std::collections::HashMap;

/// Builder for constructing mock HTTP responses
pub struct MockResponseBuilder {
    status_code: u16,
    headers: HashMap<String, String>,
    body: Option<Value>,
}

impl MockResponseBuilder {
    /// Create a new mock response builder with default 200 status
    pub fn new() -> Self {
        Self {
            status_code: 200,
            headers: HashMap::new(),
            body: None,
        }
    }

    /// Create a success response builder (200 OK)
    pub fn success() -> Self {
        Self::new()
    }

    /// Create an error response builder with specified status code
    pub fn error(status_code: u16) -> Self {
        Self {
            status_code,
            headers: HashMap::new(),
            body: None,
        }
    }

    /// Set the HTTP status code
    pub fn with_status(mut self, status: u16) -> Self {
        self.status_code = status;
        self
    }

    /// Add a header to the response
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set the response body as JSON
    pub fn with_body_json(mut self, json: Value) -> Self {
        self.body = Some(json);
        self
    }

    /// Set the response body from a JSON string
    pub fn with_body_str(mut self, json_str: &str) -> Self {
        let json: Value = serde_json::from_str(json_str)
            .expect("Invalid JSON in with_body_str");
        self.body = Some(json);
        self
    }

    /// Build the final mock response as a JSON value
    pub fn build(self) -> (u16, HashMap<String, String>, Option<Value>) {
        (self.status_code, self.headers, self.body)
    }

    /// Build and return just the body (for simple cases)
    pub fn build_body(self) -> Value {
        self.body.unwrap_or(json!(null))
    }
}

impl Default for MockResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for Datadog metrics responses
pub struct MetricsResponseBuilder {
    series: Vec<Value>,
    from_ts: i64,
    to_ts: i64,
}

impl MetricsResponseBuilder {
    pub fn new() -> Self {
        Self {
            series: Vec::new(),
            from_ts: 0,
            to_ts: 0,
        }
    }

    pub fn with_series(mut self, metric: &str, points: Vec<(i64, f64)>) -> Self {
        let point_arrays: Vec<Vec<serde_json::Number>> = points
            .into_iter()
            .map(|(ts, val)| {
                vec![
                    serde_json::Number::from(ts),
                    serde_json::Number::from_f64(val).expect("Invalid f64"),
                ]
            })
            .collect();

        self.series.push(json!({
            "metric": metric,
            "points": point_arrays,
            "scope": "host:test",
        }));
        self
    }

    pub fn with_time_range(mut self, from: i64, to: i64) -> Self {
        self.from_ts = from;
        self.to_ts = to;
        self
    }

    pub fn build(self) -> Value {
        json!({
            "series": self.series,
            "from": self.from_ts,
            "to": self.to_ts,
        })
    }
}

impl Default for MetricsResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for Datadog logs responses
pub struct LogsResponseBuilder {
    logs: Vec<Value>,
    next_cursor: Option<String>,
}

impl LogsResponseBuilder {
    pub fn new() -> Self {
        Self {
            logs: Vec::new(),
            next_cursor: None,
        }
    }

    pub fn with_log(mut self, message: &str, timestamp: i64) -> Self {
        self.logs.push(json!({
            "content": {
                "message": message,
                "timestamp": timestamp,
            }
        }));
        self
    }

    pub fn with_cursor(mut self, cursor: &str) -> Self {
        self.next_cursor = Some(cursor.to_string());
        self
    }

    pub fn build(self) -> Value {
        let mut result = json!({
            "data": self.logs,
        });

        if let Some(cursor) = self.next_cursor {
            result["meta"] = json!({"page": {"after": cursor}});
        }

        result
    }
}

impl Default for LogsResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_response_builder() {
        let (status, headers, body) = MockResponseBuilder::new()
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body_json(json!({"test": "data"}))
            .build();

        assert_eq!(status, 200);
        assert_eq!(headers.get("content-type"), Some(&"application/json".to_string()));
        assert_eq!(body, Some(json!({"test": "data"})));
    }

    #[test]
    fn test_metrics_response_builder() {
        let response = MetricsResponseBuilder::new()
            .with_series("cpu.usage", vec![(1000, 42.5), (2000, 43.0)])
            .with_time_range(1000, 2000)
            .build();

        assert_eq!(response["series"].as_array().unwrap().len(), 1);
        assert_eq!(response["from"], 1000);
        assert_eq!(response["to"], 2000);
    }

    #[test]
    fn test_logs_response_builder() {
        let response = LogsResponseBuilder::new()
            .with_log("Test message", 1234567890)
            .with_cursor("next_page_cursor")
            .build();

        assert_eq!(response["data"].as_array().unwrap().len(), 1);
        assert_eq!(response["meta"]["page"]["after"], "next_page_cursor");
    }
}
