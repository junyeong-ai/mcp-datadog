use serde_json::{Value, json};
use std::sync::Arc;

use crate::cache::DataCache;
use crate::datadog::DatadogClient;
use crate::error::Result;
use crate::handlers::common::{Paginator, ResponseFormatter, TimeHandler, TimeParams};

pub struct EventsHandler;

impl TimeHandler for EventsHandler {}
impl Paginator for EventsHandler {}
impl ResponseFormatter for EventsHandler {}

impl EventsHandler {
    pub async fn query(
        client: Arc<DatadogClient>,
        cache: Arc<DataCache>,
        params: &Value,
    ) -> Result<Value> {
        let handler = EventsHandler;

        let priority = params["priority"].as_str().map(|s| s.to_string());

        let sources = params["sources"].as_str().map(|s| s.to_string());

        let tags = params["tags"].as_str().map(|s| s.to_string());

        let time = handler.parse_time(params, 1)?; // v1 API

        let TimeParams::Timestamp {
            from: start,
            to: end,
        } = time;

        let (page, page_size) = handler.parse_pagination(params);

        let cache_key = crate::cache::create_cache_key(
            "events",
            &json!({
                "start": start,
                "end": end,
                "priority": priority,
                "sources": sources,
                "tags": tags
            }),
        );

        let events = if page == 0 {
            let response = client
                .query_events(start, end, priority.clone(), sources.clone(), tags.clone())
                .await?;
            let events = response.events.unwrap_or_default();
            cache.set_events(cache_key, events.clone()).await;
            events
        } else {
            cache
                .get_or_fetch_events(&cache_key, || async {
                    let response = client
                        .query_events(start, end, priority, sources, tags)
                        .await?;
                    Ok(response.events.unwrap_or_default())
                })
                .await?
        };

        let events_slice = handler.paginate(&events, page, page_size);

        let data = json!(
            events_slice
                .iter()
                .map(|event| {
                    json!({
                        "id": event.id,
                        "title": event.title,
                        "text": event.text,
                        "date": event.date_happened.map(crate::utils::format_timestamp),
                        "priority": event.priority,
                        "host": event.host,
                        "source": event.source,
                        "alert_type": event.alert_type
                    })
                })
                .collect::<Vec<_>>()
        );

        let pagination = handler.format_pagination(page, page_size, events.len());
        let meta = json!({
            "from": crate::utils::format_timestamp(start),
            "to": crate::utils::format_timestamp(end)
        });

        Ok(handler.format_list(data, Some(pagination), Some(meta)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_optional_priority_parameter() {
        let params = json!({"priority": "normal"});
        assert_eq!(params["priority"].as_str(), Some("normal"));
    }

    #[test]
    fn test_optional_sources_parameter() {
        let params = json!({"sources": "my_app"});
        assert_eq!(params["sources"].as_str(), Some("my_app"));
    }

    #[test]
    fn test_optional_tags_parameter() {
        let params = json!({"tags": "env:prod,service:api"});
        assert_eq!(params["tags"].as_str(), Some("env:prod,service:api"));
    }

    #[test]
    fn test_pagination_parameters() {
        let handler = EventsHandler;
        let params = json!({"page": 1, "page_size": 100});

        let (page, page_size) = handler.parse_pagination(&params);
        assert_eq!(page, 1);
        assert_eq!(page_size, 100);
    }

    #[test]
    fn test_time_handler_trait() {
        let handler = EventsHandler;
        let params = json!({
            "from": "2 hours ago",
            "to": "now"
        });

        let result = handler.parse_time(&params, 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_paginator_trait() {
        let handler = EventsHandler;
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        let page = handler.paginate(&data, 0, 3);
        assert_eq!(page, &[1, 2, 3]);
    }

    #[test]
    fn test_response_formatter_trait() {
        let handler = EventsHandler;
        let data = json!([{"id": 1}]);
        let pagination = json!({"page": 0});
        let meta = json!({"from": "timestamp"});

        let response = handler.format_list(data, Some(pagination), Some(meta));
        assert!(response.get("data").is_some());
        assert!(response.get("pagination").is_some());
        assert!(response.get("meta").is_some());
    }
}
