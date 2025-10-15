use serde_json::{Value, json};
use std::sync::Arc;

use crate::cache::DataCache;
use crate::datadog::DatadogClient;
use crate::error::Result;
use crate::handlers::common::{Paginator, ResponseFormatter};

pub struct MonitorsHandler;

impl Paginator for MonitorsHandler {}
impl ResponseFormatter for MonitorsHandler {}

impl MonitorsHandler {
    pub async fn list(
        client: Arc<DatadogClient>,
        cache: Arc<DataCache>,
        params: &Value,
    ) -> Result<Value> {
        let handler = MonitorsHandler;
        let tags = params["tags"].as_str().map(|s| s.to_string());

        let monitor_tags = params["monitor_tags"].as_str().map(|s| s.to_string());

        let (page, page_size) = handler.parse_pagination(params);

        let cache_key = crate::cache::create_cache_key(
            "monitors",
            &json!({
                "tags": tags,
                "monitor_tags": monitor_tags
            }),
        );

        let monitors = if page == 0 {
            let fresh_monitors = client.list_monitors(tags, monitor_tags, None, None).await?;
            cache.set_monitors(cache_key.clone(), fresh_monitors).await;
            cache
                .get_or_fetch_monitors(&cache_key, || async { unreachable!("Just inserted") })
                .await?
        } else {
            cache
                .get_or_fetch_monitors(&cache_key, || async {
                    client.list_monitors(tags, monitor_tags, None, None).await
                })
                .await?
        };

        let monitors_slice = handler.paginate(&monitors, page, page_size);

        let data = json!(
            monitors_slice
                .iter()
                .map(|monitor| {
                    json!({
                        "id": monitor.id,
                        "name": monitor.name,
                        "type": monitor.monitor_type,
                        "query": monitor.query,
                        "status": monitor.overall_state,
                        "tags": monitor.tags,
                        "priority": monitor.priority
                    })
                })
                .collect::<Vec<_>>()
        );

        let pagination = handler.format_pagination(page, page_size, monitors.len());

        Ok(handler.format_list(data, Some(pagination), None))
    }

    pub async fn get(client: Arc<DatadogClient>, params: &Value) -> Result<Value> {
        let handler = MonitorsHandler;

        let monitor_id = params["monitor_id"].as_i64().ok_or_else(|| {
            crate::error::DatadogError::InvalidInput("Missing 'monitor_id' parameter".to_string())
        })?;

        let response = client.get_monitor(monitor_id).await?;

        let data = json!({
            "id": response.id,
            "name": response.name,
            "type": response.monitor_type,
            "query": response.query,
            "message": response.message,
            "tags": response.tags,
            "created": response.created,
            "modified": response.modified,
            "overall_state": response.overall_state,
            "priority": response.priority,
            "options": response.options.as_ref().map(|o| {
                let mut opts = json!({
                    "thresholds": o.thresholds,
                    "notify_no_data": o.notify_no_data,
                    "notify_audit": o.notify_audit,
                    "timeout_h": o.timeout_h
                });

                // Only include silenced if it has entries
                if let Some(ref silenced) = o.silenced
                    && let Some(obj) = silenced.as_object()
                    && !obj.is_empty()
                {
                    opts["silenced"] = json!(silenced);
                }

                opts
            })
        });

        Ok(handler.format_detail(data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_optional_tags_parameter() {
        let params_with = json!({"tags": "env:prod"});
        let params_without = json!({});

        assert_eq!(params_with["tags"].as_str(), Some("env:prod"));
        assert_eq!(params_without["tags"].as_str(), None);
    }

    #[test]
    fn test_optional_monitor_tags_parameter() {
        let params = json!({"monitor_tags": "service:web"});
        assert_eq!(params["monitor_tags"].as_str(), Some("service:web"));
    }

    #[test]
    fn test_pagination_defaults() {
        let handler = MonitorsHandler;
        let params = json!({});

        let (page, page_size) = handler.parse_pagination(&params);
        assert_eq!(page, 0);
        assert_eq!(page_size, 50);
    }

    #[test]
    fn test_pagination_custom() {
        let handler = MonitorsHandler;
        let params = json!({
            "page": 2,
            "page_size": 25
        });

        let (page, page_size) = handler.parse_pagination(&params);
        assert_eq!(page, 2);
        assert_eq!(page_size, 25);
    }

    #[test]
    fn test_get_missing_monitor_id() {
        let params = json!({});
        let monitor_id = params["monitor_id"].as_i64();
        assert_eq!(monitor_id, None);
    }

    #[test]
    fn test_get_valid_monitor_id() {
        let params = json!({"monitor_id": 12345});
        let monitor_id = params["monitor_id"].as_i64();
        assert_eq!(monitor_id, Some(12345));
    }

    #[test]
    fn test_paginator_trait() {
        let handler = MonitorsHandler;
        let data = vec![1, 2, 3, 4, 5];

        let page1 = handler.paginate(&data, 0, 2);
        assert_eq!(page1, &[1, 2]);

        let page2 = handler.paginate(&data, 1, 2);
        assert_eq!(page2, &[3, 4]);
    }

    #[test]
    fn test_response_formatter_list() {
        let handler = MonitorsHandler;
        let data = json!([{"id": 1}, {"id": 2}]);
        let pagination = json!({"page": 0, "page_size": 50});

        let response = handler.format_list(data, Some(pagination), None);
        assert!(response.get("data").is_some());
        assert!(response.get("pagination").is_some());
    }

    #[test]
    fn test_response_formatter_detail() {
        let handler = MonitorsHandler;
        let data = json!({"id": 123, "name": "Test Monitor"});

        let response = handler.format_detail(data.clone());
        assert_eq!(response["data"], data);
    }

    #[test]
    fn test_format_pagination() {
        let handler = MonitorsHandler;
        let pagination = handler.format_pagination(0, 50, 150);

        assert_eq!(pagination["page"], 0);
        assert_eq!(pagination["page_size"], 50);
        assert_eq!(pagination["total"], 150);
        assert_eq!(pagination["has_next"], true);
    }
}
