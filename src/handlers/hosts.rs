use serde_json::{Value, json};
use std::sync::Arc;

use crate::datadog::DatadogClient;
use crate::error::Result;
use crate::handlers::common::{ResponseFormatter, TimeHandler, TimeParams};

pub struct HostsHandler;

impl TimeHandler for HostsHandler {}
impl ResponseFormatter for HostsHandler {}

impl HostsHandler {
    pub async fn list(client: Arc<DatadogClient>, params: &Value) -> Result<Value> {
        let handler = HostsHandler;

        let filter = params["filter"].as_str().map(|s| s.to_string());

        let sort_field = params["sort_field"].as_str().map(|s| s.to_string());

        let sort_dir = params["sort_dir"].as_str().map(|s| s.to_string());

        let time = handler.parse_time(params, 1)?;
        let TimeParams::Timestamp { from, .. } = time;
        let from = Some(from);

        let start = params["start"].as_i64().map(|s| s as i32);

        let count = params["count"].as_i64().map(|c| c as i32).or(Some(100));

        let response = client
            .list_hosts(filter, from, sort_field, sort_dir, start, count)
            .await?;

        // Get tag filter (same pattern as logs/spans)
        let tag_filter = params["tag_filter"]
            .as_str()
            .or_else(|| client.get_tag_filter())
            .unwrap_or("*");

        let data = json!(response.host_list.iter().map(|host| {
            // Apply tag filtering to tags_by_source
            let filtered_tags_by_source = match tag_filter {
                "*" => host.tags_by_source.clone(),
                "" => None,
                filter => {
                    host.tags_by_source.as_ref().map(|tags_map| {
                        let prefixes: Vec<&str> = filter.split(',').map(str::trim).collect();
                        let mut filtered_map = std::collections::HashMap::new();

                        for (source, tags) in tags_map.iter() {
                            let filtered_tags: Vec<String> = tags
                                .iter()
                                .filter(|tag| prefixes.iter().any(|p| tag.starts_with(p)))
                                .cloned()
                                .collect();

                            if !filtered_tags.is_empty() {
                                filtered_map.insert(source.clone(), filtered_tags);
                            }
                        }

                        filtered_map
                    })
                }
            };

            json!({
                "name": host.name,
                "host_name": host.host_name,
                "up": host.up,
                "is_muted": host.is_muted,
                "last_reported": host.last_reported_time.map(crate::utils::format_timestamp),
                "aws_name": host.aws_name,
                "apps": host.apps,
                "sources": host.sources,
                "tags": filtered_tags_by_source
            })
        }).collect::<Vec<_>>());

        let meta = json!({
            "total_matching": response.total_matching,
            "total_returned": response.total_returned
        });

        Ok(handler.format_list(data, None, Some(meta)))
    }
}
