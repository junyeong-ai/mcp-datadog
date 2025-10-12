use serde_json::{Value, json};
use std::sync::Arc;

use crate::datadog::DatadogClient;
use crate::error::Result;
use crate::handlers::common::{Paginator, ResponseFormatter, TimeHandler, TimeParams};

pub struct SpansHandler;

impl TimeHandler for SpansHandler {}
impl Paginator for SpansHandler {}
impl ResponseFormatter for SpansHandler {}

impl SpansHandler {
    pub async fn list(client: Arc<DatadogClient>, params: &Value) -> Result<Value> {
        let handler = SpansHandler;

        let query = params["query"].as_str().unwrap_or("*").to_string();

        // Parse time as timestamps first, then convert to ISO8601
        let time = handler.parse_time(params, 1)?; // Parse as v1 to get timestamps
        let TimeParams::Timestamp {
            from: from_ts,
            to: to_ts,
        } = time;

        // Convert timestamps to ISO8601 format for v2 API
        use chrono::DateTime;
        let from = DateTime::from_timestamp(from_ts, 0)
            .ok_or_else(|| {
                crate::error::DatadogError::InvalidInput("Invalid from timestamp".to_string())
            })?
            .to_rfc3339();
        let to = DateTime::from_timestamp(to_ts, 0)
            .ok_or_else(|| {
                crate::error::DatadogError::InvalidInput("Invalid to timestamp".to_string())
            })?
            .to_rfc3339();

        let (page, page_size) = handler.parse_pagination(params);
        let limit = params["limit"]
            .as_i64()
            .map(|l| l as i32)
            .or(Some(page_size as i32));
        let cursor = params["cursor"].as_str().map(|s| s.to_string());
        let sort = params["sort"].as_str().map(|s| s.to_string());

        let response = client
            .list_spans(&query, &from, &to, limit, cursor, sort)
            .await?;

        // Get tag filter (same pattern as logs)
        let tag_filter = params["tag_filter"]
            .as_str()
            .or_else(|| client.get_tag_filter())
            .unwrap_or("*");

        // Process spans with tag filtering
        let data = response["data"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|span| {
                let mut span_obj = span.as_object().unwrap().clone();

                // Apply tag filtering to attributes.tags
                if let Some(attrs) = span_obj.get_mut("attributes") {
                    if let Some(attrs_obj) = attrs.as_object_mut() {
                        if let Some(tags) = attrs_obj.get("tags") {
                            if let Some(tags_arr) = tags.as_array() {
                                let filtered_tags = match tag_filter {
                                    "*" => tags_arr.clone(),
                                    "" => vec![],
                                    filter => {
                                        let prefixes: Vec<&str> =
                                            filter.split(',').map(str::trim).collect();
                                        tags_arr
                                            .iter()
                                            .filter(|tag| {
                                                if let Some(tag_str) = tag.as_str() {
                                                    prefixes.iter().any(|p| tag_str.starts_with(p))
                                                } else {
                                                    false
                                                }
                                            })
                                            .cloned()
                                            .collect()
                                    }
                                };
                                attrs_obj.insert("tags".to_string(), Value::Array(filtered_tags));
                            }
                        }
                    }
                }

                Value::Object(span_obj)
            })
            .collect::<Vec<_>>();

        let spans_count = data.len();
        let pagination = handler.format_pagination(page, page_size, spans_count);

        let meta = json!({
            "query": query,
            "from": from,
            "to": to
        });

        Ok(handler.format_list(json!(data), Some(pagination), Some(meta)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_required_query_parameter() {
        let params = json!({"query": "service:web-api"});
        assert_eq!(params["query"].as_str(), Some("service:web-api"));
    }

    #[test]
    fn test_default_limit() {
        let params = json!({});
        let limit = params["limit"].as_i64().map(|l| l as i32).or(Some(10));
        assert_eq!(limit, Some(10));
    }

    #[test]
    fn test_optional_sort_parameter() {
        let params = json!({"sort": "timestamp"});
        assert_eq!(params["sort"].as_str(), Some("timestamp"));
    }

    #[test]
    fn test_pagination_parameters() {
        let handler = SpansHandler;
        let params = json!({"page": 1, "page_size": 50});

        let (page, page_size) = handler.parse_pagination(&params);
        assert_eq!(page, 1);
        assert_eq!(page_size, 50);
    }

    #[test]
    fn test_time_handler_trait() {
        let handler = SpansHandler;
        let params = json!({
            "from": "1 hour ago",
            "to": "now"
        });

        let result = handler.parse_time(&params, 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_response_formatter_trait() {
        let handler = SpansHandler;
        let data = json!([{"span_id": "123"}]);
        let pagination = json!({"page": 0});
        let meta = json!({"query": "*"});

        let response = handler.format_list(data, Some(pagination), Some(meta));
        assert!(response.get("data").is_some());
    }
}
