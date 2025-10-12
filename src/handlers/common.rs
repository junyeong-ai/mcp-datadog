use crate::error::Result;
use crate::utils::parse_time;
use serde_json::{Value, json};

/// Time parameters as timestamp format
pub enum TimeParams {
    Timestamp { from: i64, to: i64 },
}

pub trait TimeHandler {
    /// Parse time parameters from request - always returns timestamps
    fn parse_time(&self, params: &Value, _api_version: u8) -> Result<TimeParams> {
        let from_str = params["from"].as_str().unwrap_or("1 hour ago").to_string();

        let to_str = params["to"].as_str().unwrap_or("now").to_string();

        // Always parse to timestamps - individual APIs handle their own format conversion
        let from = parse_time(&from_str)?;
        let to = parse_time(&to_str)?;
        Ok(TimeParams::Timestamp { from, to })
    }
}

pub trait Paginator {
    /// Parse pagination parameters
    fn parse_pagination(&self, params: &Value) -> (usize, usize) {
        let page = params["page"].as_u64().unwrap_or(0) as usize;

        let page_size = params["page_size"].as_u64().unwrap_or(50) as usize;

        (page, page_size)
    }

    /// Apply pagination to a slice of data
    fn paginate<'a, T>(&self, data: &'a [T], page: usize, page_size: usize) -> &'a [T] {
        let start = page * page_size;
        let end = std::cmp::min(start + page_size, data.len());

        if start < data.len() {
            &data[start..end]
        } else {
            &data[0..0] // Empty slice
        }
    }
}

pub trait ResponseFormatter {
    /// Format standard list response
    fn format_list(&self, data: Value, pagination: Option<Value>, meta: Option<Value>) -> Value {
        let mut response = json!({ "data": data });

        if let Some(p) = pagination {
            response["pagination"] = p;
        }

        if let Some(m) = meta {
            response["meta"] = m;
        }

        response
    }

    /// Format standard detail response
    fn format_detail(&self, data: Value) -> Value {
        json!({ "data": data })
    }

    /// Format pagination metadata
    fn format_pagination(
        &self,
        page: usize,
        page_size: usize,
        total: usize,
    ) -> Value {
        json!({
            "page": page,
            "page_size": page_size,
            "total": total,
            "has_next": (page + 1) * page_size < total
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct TestHandler;
    impl TimeHandler for TestHandler {}
    impl Paginator for TestHandler {}
    impl ResponseFormatter for TestHandler {}

    #[test]
    fn test_time_handler_parse_time() {
        let handler = TestHandler;
        let params = json!({
            "from": "1609459200",
            "to": "1609462800"
        });

        let result = handler.parse_time(&params, 1);
        assert!(result.is_ok());

        if let Ok(TimeParams::Timestamp { from, to }) = result {
            assert!(from > 0);
            assert!(to > from);
        }
    }

    #[test]
    fn test_time_handler_defaults() {
        let handler = TestHandler;
        let params = json!({});

        // Should use defaults: "1 hour ago" and "now"
        let result = handler.parse_time(&params, 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_paginator_parse() {
        let handler = TestHandler;

        let params = json!({
            "page": 2,
            "page_size": 25
        });

        let (page, page_size) = handler.parse_pagination(&params);
        assert_eq!(page, 2);
        assert_eq!(page_size, 25);
    }

    #[test]
    fn test_paginator_defaults() {
        let handler = TestHandler;
        let params = json!({});

        let (page, page_size) = handler.parse_pagination(&params);
        assert_eq!(page, 0); // Default page
        assert_eq!(page_size, 50); // Default page_size
    }

    #[test]
    fn test_paginator_paginate() {
        let handler = TestHandler;
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        // First page
        let page1 = handler.paginate(&data, 0, 3);
        assert_eq!(page1, &[1, 2, 3]);

        // Second page
        let page2 = handler.paginate(&data, 1, 3);
        assert_eq!(page2, &[4, 5, 6]);

        // Last page (partial)
        let page4 = handler.paginate(&data, 3, 3);
        assert_eq!(page4, &[10]);

        // Beyond available data
        let page_empty = handler.paginate(&data, 10, 3);
        assert_eq!(page_empty.len(), 0);
    }

    #[test]
    fn test_response_formatter_list() {
        let handler = TestHandler;
        let data = json!(["item1", "item2"]);

        let response = handler.format_list(data.clone(), None, None);
        assert_eq!(response["data"], data);
        assert!(response["pagination"].is_null());
        assert!(response["meta"].is_null());
    }

    #[test]
    fn test_response_formatter_with_meta() {
        let handler = TestHandler;
        let data = json!(["item1"]);
        let meta = json!({"count": 1});

        let response = handler.format_list(data.clone(), None, Some(meta.clone()));
        assert_eq!(response["data"], data);
        assert_eq!(response["meta"], meta);
    }

    #[test]
    fn test_response_formatter_pagination() {
        let handler = TestHandler;

        let pagination = handler.format_pagination(0, 50, 150);
        assert_eq!(pagination["page"], 0);
        assert_eq!(pagination["page_size"], 50);
        assert_eq!(pagination["total"], 150);
        assert_eq!(pagination["has_next"], true);

        // Page 2: (2+1)*50 = 150, not < 150, so has_next = false
        let last_page = handler.format_pagination(2, 50, 150);
        assert_eq!(last_page["has_next"], false);

        // Page 1: (1+1)*50 = 100 < 150, so has_next = true
        let mid_page = handler.format_pagination(1, 50, 150);
        assert_eq!(mid_page["has_next"], true);
    }

    #[test]
    fn test_response_formatter_detail() {
        let handler = TestHandler;
        let data = json!({"id": 123, "name": "test"});

        let response = handler.format_detail(data.clone());
        assert_eq!(response["data"], data);
    }
}

