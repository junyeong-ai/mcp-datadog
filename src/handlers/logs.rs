use serde_json::{Value, json};
use std::sync::Arc;

use crate::datadog::DatadogClient;
use crate::error::Result;
use crate::handlers::common::{ResponseFormatter, TimeHandler, TimeParams};

pub struct LogsHandler;

impl TimeHandler for LogsHandler {}
impl ResponseFormatter for LogsHandler {}

impl LogsHandler {
    pub async fn search(client: Arc<DatadogClient>, params: &Value) -> Result<Value> {
        let handler = LogsHandler;

        let query = params["query"].as_str().ok_or_else(|| {
            crate::error::DatadogError::InvalidInput("Missing 'query' parameter".to_string())
        })?;

        let limit = params["limit"].as_i64().map(|l| l as i32).or(Some(10));

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

        // Determine tag filter: parameter > env var > "*" (all tags)
        let tag_filter = params["tag_filter"]
            .as_str()
            .or_else(|| client.get_tag_filter())
            .unwrap_or("*");

        let logs = response
            .data
            .unwrap_or_default()
            .iter()
            .map(|log| {
                let attrs = log.attributes.as_ref();

                // Apply tag filtering with explicit keywords
                let tags = match tag_filter {
                    "*" => {
                        // "*" = return all tags (no filtering)
                        attrs.and_then(|a| a.tags.clone())
                    }
                    "" => {
                        // "" = exclude all tags
                        None
                    }
                    filter => {
                        // Specific prefixes = filter tags
                        attrs.and_then(|a| a.tags.as_ref().map(|tags| {
                            let prefixes: Vec<&str> = filter.split(',').map(str::trim).collect();
                            tags.iter()
                                .filter(|tag| prefixes.iter().any(|p| tag.starts_with(p)))
                                .cloned()
                                .collect::<Vec<_>>()
                        }))
                    }
                };

                json!({
                    "id": log.id,
                    "timestamp": attrs.and_then(|a| a.timestamp.clone()),
                    "message": attrs.and_then(|a| a.message.clone()),
                    "host": attrs.and_then(|a| a.host.clone()),
                    "service": attrs.and_then(|a| a.service.clone()),
                    "tags": tags,
                    "status": attrs.and_then(|a| a.status.clone())
                })
            })
            .collect::<Vec<_>>();

        let meta = json!({
            "query": query,
            "from": from_iso,
            "to": to_iso,
            "total": logs.len()
        });

        Ok(handler.format_list(json!(logs), None, Some(meta)))
    }
}
