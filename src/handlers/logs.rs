use std::sync::Arc;
use serde_json::{json, Value};

use crate::datadog::DatadogClient;
use crate::error::Result;
use crate::handlers::common::{TimeHandler, TimeParams, ResponseFormatter};

pub struct LogsHandler;

impl TimeHandler for LogsHandler {}
impl ResponseFormatter for LogsHandler {}

impl LogsHandler {
    pub async fn search(
        client: Arc<DatadogClient>,
        params: &Value,
    ) -> Result<Value> {
        let handler = LogsHandler;
        
        let query = params["query"]
            .as_str()
            .ok_or_else(|| crate::error::DatadogError::InvalidInput("Missing 'query' parameter".to_string()))?;
        
        let limit = params["limit"]
            .as_i64()
            .map(|l| l as i32)
            .or(Some(10));
        
        // Parse time - v2 API uses string format initially but we need to convert from user input
        let time = handler.parse_time(params, 1)?; // Parse as v1 to get timestamps
        
        // Convert timestamps to ISO format for v2 logs API
        let TimeParams::Timestamp { from, to } = time;
        let from_iso = chrono::DateTime::from_timestamp(from, 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| "1 hour ago".to_string());
        
        let to_iso = chrono::DateTime::from_timestamp(to, 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| "now".to_string());
        
        let response = client.search_logs(query, &from_iso, &to_iso, limit).await?;
        
        if let Some(errors) = response.errors {
            return Err(crate::error::DatadogError::ApiError(errors.join(", ")));
        }
        
        let logs = response.data.unwrap_or_default().iter().map(|log| {
            let attrs = log.attributes.as_ref();
            json!({
                "id": log.id,
                "timestamp": attrs.and_then(|a| a.timestamp.clone()),
                "message": attrs.and_then(|a| a.message.clone()),
                "host": attrs.and_then(|a| a.host.clone()),
                "service": attrs.and_then(|a| a.service.clone()),
                "tags": attrs.and_then(|a| a.tags.clone()),
                "status": attrs.and_then(|a| a.status.clone()),
                "attributes": attrs.and_then(|a| a.attributes.clone())
            })
        }).collect::<Vec<_>>();
        
        let meta = json!({
            "query": query,
            "from": from_iso,
            "to": to_iso,
            "total": logs.len()
        });
        
        Ok(handler.format_list(
            json!(logs),
            None,
            Some(meta)
        ))
    }
}