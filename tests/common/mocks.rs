// Mock implementations for testing
// Provides test doubles that don't make real HTTP calls

use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Mock API call record for verification
#[derive(Debug, Clone)]
pub struct ApiCall {
    pub method: String,
    pub endpoint: String,
    pub params: HashMap<String, String>,
}

/// Mock response to return from API calls
#[derive(Debug, Clone)]
pub struct MockResponse {
    pub status: u16,
    pub body: Value,
    pub headers: HashMap<String, String>,
}

impl MockResponse {
    pub fn success(body: Value) -> Self {
        Self {
            status: 200,
            body,
            headers: HashMap::new(),
        }
    }

    pub fn error(status: u16, message: &str) -> Self {
        Self {
            status,
            body: serde_json::json!({"error": message}),
            headers: HashMap::new(),
        }
    }
}

/// Mock Datadog client for testing without real API calls
pub struct MockDatadogClient {
    /// Expected calls (method, endpoint) -> response
    expectations: Arc<Mutex<HashMap<(String, String), Vec<MockResponse>>>>,
    /// History of actual calls made
    call_history: Arc<Mutex<Vec<ApiCall>>>,
    /// Default response if no expectation set
    default_response: Arc<Mutex<Option<MockResponse>>>,
}

impl MockDatadogClient {
    pub fn new() -> Self {
        Self {
            expectations: Arc::new(Mutex::new(HashMap::new())),
            call_history: Arc::new(Mutex::new(Vec::new())),
            default_response: Arc::new(Mutex::new(None)),
        }
    }

    /// Set up an expectation for a specific API call
    pub fn expect_call(&self, method: &str, endpoint: &str) -> ExpectationBuilder {
        ExpectationBuilder {
            mock: self,
            method: method.to_string(),
            endpoint: endpoint.to_string(),
        }
    }

    /// Set a default response for any unexpected calls
    pub fn with_default_response(&self, response: MockResponse) {
        *self.default_response.lock().unwrap() = Some(response);
    }

    /// Record an API call and return the mocked response
    pub fn call(&self, method: &str, endpoint: &str, params: HashMap<String, String>) -> Result<MockResponse, String> {
        // Record the call
        self.call_history.lock().unwrap().push(ApiCall {
            method: method.to_string(),
            endpoint: endpoint.to_string(),
            params,
        });

        // Find matching expectation
        let key = (method.to_string(), endpoint.to_string());
        let mut expectations = self.expectations.lock().unwrap();

        if let Some(responses) = expectations.get_mut(&key) {
            if !responses.is_empty() {
                return Ok(responses.remove(0));
            }
        }

        // Use default response if available
        if let Some(default) = self.default_response.lock().unwrap().as_ref() {
            return Ok(default.clone());
        }

        Err(format!("No expectation set for {} {}", method, endpoint))
    }

    /// Verify that an endpoint was called
    pub fn verify_called(&self, endpoint: &str) -> bool {
        self.call_history
            .lock()
            .unwrap()
            .iter()
            .any(|call| call.endpoint == endpoint)
    }

    /// Get the number of times an endpoint was called
    pub fn call_count(&self, endpoint: &str) -> usize {
        self.call_history
            .lock()
            .unwrap()
            .iter()
            .filter(|call| call.endpoint == endpoint)
            .count()
    }

    /// Get all recorded calls
    pub fn get_call_history(&self) -> Vec<ApiCall> {
        self.call_history.lock().unwrap().clone()
    }

    /// Get the last call made
    pub fn get_last_call(&self) -> Option<ApiCall> {
        self.call_history.lock().unwrap().last().cloned()
    }

    /// Reset all expectations and call history
    pub fn reset(&self) {
        self.expectations.lock().unwrap().clear();
        self.call_history.lock().unwrap().clear();
        *self.default_response.lock().unwrap() = None;
    }
}

impl Default for MockDatadogClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for setting up call expectations
pub struct ExpectationBuilder<'a> {
    mock: &'a MockDatadogClient,
    method: String,
    endpoint: String,
}

impl<'a> ExpectationBuilder<'a> {
    /// Set the response to return for this expectation
    pub fn return_response(self, response: MockResponse) {
        let key = (self.method, self.endpoint);
        self.mock
            .expectations
            .lock()
            .unwrap()
            .entry(key)
            .or_insert_with(Vec::new)
            .push(response);
    }

    /// Set multiple responses (for testing retry logic)
    pub fn return_responses(self, responses: Vec<MockResponse>) {
        let key = (self.method, self.endpoint);
        self.mock
            .expectations
            .lock()
            .unwrap()
            .entry(key)
            .or_insert_with(Vec::new)
            .extend(responses);
    }

    /// Return a successful response with JSON body
    pub fn return_json(self, body: Value) {
        self.return_response(MockResponse::success(body));
    }

    /// Return an error response
    pub fn return_error(self, status: u16, message: &str) {
        self.return_response(MockResponse::error(status, message));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_mock_client_expectation() {
        let mock = MockDatadogClient::new();

        mock.expect_call("GET", "/api/v1/metrics")
            .return_json(json!({"series": []}));

        let response = mock.call("GET", "/api/v1/metrics", HashMap::new()).unwrap();
        assert_eq!(response.status, 200);
        assert!(mock.verify_called("/api/v1/metrics"));
    }

    #[test]
    fn test_mock_client_call_count() {
        let mock = MockDatadogClient::new();
        mock.with_default_response(MockResponse::success(json!({})));

        mock.call("GET", "/api/v1/test", HashMap::new()).unwrap();
        mock.call("GET", "/api/v1/test", HashMap::new()).unwrap();

        assert_eq!(mock.call_count("/api/v1/test"), 2);
    }

    #[test]
    fn test_mock_client_retry_responses() {
        let mock = MockDatadogClient::new();

        mock.expect_call("GET", "/api/v1/test")
            .return_responses(vec![
                MockResponse::error(500, "Server error"),
                MockResponse::error(500, "Server error"),
                MockResponse::success(json!({"ok": true})),
            ]);

        // First call fails
        let r1 = mock.call("GET", "/api/v1/test", HashMap::new()).unwrap();
        assert_eq!(r1.status, 500);

        // Second call fails
        let r2 = mock.call("GET", "/api/v1/test", HashMap::new()).unwrap();
        assert_eq!(r2.status, 500);

        // Third call succeeds
        let r3 = mock.call("GET", "/api/v1/test", HashMap::new()).unwrap();
        assert_eq!(r3.status, 200);
    }
}
