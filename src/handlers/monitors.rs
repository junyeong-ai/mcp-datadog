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
            cache.set_monitors(cache_key, fresh_monitors.clone()).await;
            fresh_monitors
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

        let pagination =
            handler.format_pagination(page, page_size, monitors.len());

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
            "options": response.options.as_ref().map(|o| json!({
                "thresholds": o.thresholds,
                "notify_no_data": o.notify_no_data,
                "notify_audit": o.notify_audit,
                "timeout_h": o.timeout_h,
                "silenced": o.silenced
            }))
        });

        Ok(handler.format_detail(data))
    }
}
