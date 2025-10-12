use std::sync::Arc;
use serde_json::{json, Value};

use crate::datadog::DatadogClient;
use crate::error::Result;
use crate::handlers::common::{TimeHandler, TimeParams, ResponseFormatter};

pub struct HostsHandler;

impl TimeHandler for HostsHandler {}
impl ResponseFormatter for HostsHandler {}

impl HostsHandler {
    pub async fn list(
        client: Arc<DatadogClient>,
        params: &Value,
    ) -> Result<Value> {
        let handler = HostsHandler;

        let filter = params["filter"]
            .as_str()
            .map(|s| s.to_string());

        let sort_field = params["sort_field"]
            .as_str()
            .map(|s| s.to_string());

        let sort_dir = params["sort_dir"]
            .as_str()
            .map(|s| s.to_string());

        let time = handler.parse_time(params, 1)?;
        let TimeParams::Timestamp { from, .. } = time;
        let from = Some(from);

        let start = params["start"]
            .as_i64()
            .map(|s| s as i32);

        let count = params["count"]
            .as_i64()
            .map(|c| c as i32)
            .or(Some(100));

        let response = client.list_hosts(filter, from, sort_field, sort_dir, start, count).await?;

        let data = json!(response.host_list.iter().map(|host| {
            json!({
                "name": host.name,
                "host_name": host.host_name,
                "up": host.up,
                "is_muted": host.is_muted,
                "last_reported": host.last_reported_time.map(crate::utils::format_timestamp),
                "aws_name": host.aws_name,
                "apps": host.apps,
                "sources": host.sources,
                "tags": host.tags_by_source
            })
        }).collect::<Vec<_>>());

        let meta = json!({
            "total_matching": response.total_matching,
            "total_returned": response.total_returned
        });

        Ok(handler.format_list(data, None, Some(meta)))
    }
}
