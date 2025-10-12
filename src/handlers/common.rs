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
