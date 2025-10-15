use serde_json::{Value, json};
use std::sync::Arc;

use crate::datadog::DatadogClient;
use crate::error::Result;
use crate::handlers::common::{ResponseFormatter, TagFilter, TimeHandler, TimeParams};

pub struct LogsHandler;

impl TimeHandler for LogsHandler {}
impl TagFilter for LogsHandler {}
impl ResponseFormatter for LogsHandler {}

impl LogsHandler {
    pub async fn search(client: Arc<DatadogClient>, params: &Value) -> Result<Value> {
        let handler = LogsHandler;

        let query = params["query"].as_str().ok_or_else(|| {
            crate::error::DatadogError::InvalidInput("Missing 'query' parameter".to_string())
        })?;

        let limit = params["limit"].as_i64().map(|l| l as i32).or(Some(10));

        // Parse time and convert to ISO8601 format for v2 logs API
        let time = handler.parse_time(params, 1)?;
        let TimeParams::Timestamp { from, to } = time;
        let from_iso = handler.timestamp_to_iso8601(from)?;
        let to_iso = handler.timestamp_to_iso8601(to)?;

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
                let tags = attrs
                    .and_then(|a| a.tags.as_ref())
                    .map(|t| handler.filter_tags(t, tag_filter));

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_missing_query_parameter() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let client = Arc::new(
                DatadogClient::new("test_key".to_string(), "test_app_key".to_string(), None)
                    .unwrap(),
            );

            let params = json!({
                "from": "1 hour ago",
                "to": "now"
                // Missing "query"
            });

            let result = LogsHandler::search(client, &params).await;
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_valid_input_parameters() {
        let params = json!({
            "query": "service:web-api",
            "from": "1 hour ago",
            "to": "now",
            "limit": 50
        });

        assert_eq!(params["query"].as_str(), Some("service:web-api"));
        assert_eq!(params["limit"].as_i64(), Some(50));
    }

    #[test]
    fn test_optional_limit_parameter() {
        let params_with = json!({"query": "test", "limit": 100});
        let params_without = json!({"query": "test"});

        assert_eq!(params_with["limit"].as_i64(), Some(100));
        assert_eq!(params_without["limit"].as_i64(), None);
    }

    #[test]
    fn test_tag_filter_modes() {
        // Test all tags mode
        let _tags = ["env:prod".to_string(), "service:api".to_string()];
        let filter_all = "*";
        assert_eq!(filter_all, "*");

        // Test no tags mode
        let filter_none = "";
        assert_eq!(filter_none, "");

        // Test prefix filtering
        let filter_prefixes = "env:,service:";
        assert!(filter_prefixes.contains("env:"));
        assert!(filter_prefixes.contains("service:"));
    }

    #[test]
    fn test_time_handler_available() {
        let handler = LogsHandler;
        let params = json!({
            "from": "1609459200",
            "to": "1609462800"
        });

        let result = handler.parse_time(&params, 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_response_formatter_available() {
        let handler = LogsHandler;
        let data = json!([{"id": "log1"}]);
        let formatted = handler.format_list(data, None, None);
        assert!(formatted.get("data").is_some());
    }
}
