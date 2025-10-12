use std::sync::Arc;
use serde_json::{json, Value};

use crate::datadog::DatadogClient;
use crate::error::Result;
use crate::handlers::common::{TimeHandler, TimeParams, Paginator, ResponseFormatter};

pub struct SpansHandler;

impl TimeHandler for SpansHandler {}
impl Paginator for SpansHandler {}
impl ResponseFormatter for SpansHandler {}

impl SpansHandler {
    pub async fn list(
        client: Arc<DatadogClient>,
        params: &Value,
    ) -> Result<Value> {
        let handler = SpansHandler;
        
        let query = params["query"]
            .as_str()
            .unwrap_or("*")
            .to_string();
        
        // Parse time as timestamps first, then convert to ISO8601
        let time = handler.parse_time(params, 1)?; // Parse as v1 to get timestamps
        let TimeParams::Timestamp { from: from_ts, to: to_ts } = time;
        
        // Convert timestamps to ISO8601 format for v2 API
        use chrono::DateTime;
        let from = DateTime::from_timestamp(from_ts, 0)
            .ok_or_else(|| crate::error::DatadogError::InvalidInput("Invalid from timestamp".to_string()))?
            .to_rfc3339();
        let to = DateTime::from_timestamp(to_ts, 0)
            .ok_or_else(|| crate::error::DatadogError::InvalidInput("Invalid to timestamp".to_string()))?
            .to_rfc3339();
        
        let (page, page_size) = handler.parse_pagination(params);
        let limit = params["limit"].as_i64().map(|l| l as i32).or(Some(page_size as i32));
        let cursor = params["cursor"].as_str().map(|s| s.to_string());
        let sort = params["sort"].as_str().map(|s| s.to_string());

        let response = client.list_spans(
            &query,
            &from,
            &to,
            limit,
            cursor,
            sort,
        ).await?;

        // Return raw response for debugging
        let data = response["data"].as_array().unwrap_or(&vec![]).clone();
        let spans_count = data.len();
        
        let pagination = handler.format_pagination(page, page_size, spans_count, spans_count);
        
        let meta = json!({
            "query": query,
            "from": from,
            "to": to,
            "raw_response": response,
            "count": spans_count
        });
        
        Ok(handler.format_list(json!(data), Some(pagination), Some(meta)))
    }
}