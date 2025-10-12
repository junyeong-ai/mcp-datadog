use std::sync::Arc;
use serde_json::{json, Value};

use crate::datadog::{DatadogClient, models::{LogsCompute, LogsGroupBy, LogsGroupBySort}};
use crate::error::Result;
use crate::handlers::common::{TimeHandler, TimeParams, ResponseFormatter};

pub struct LogsAggregateHandler;

impl TimeHandler for LogsAggregateHandler {}
impl ResponseFormatter for LogsAggregateHandler {}

impl LogsAggregateHandler {
    pub async fn aggregate(
        client: Arc<DatadogClient>,
        params: &Value,
    ) -> Result<Value> {
        let handler = LogsAggregateHandler;

        // Use v1 API time parsing to get timestamps, then convert to milliseconds strings
        let time = handler.parse_time(params, 1)?; // Parse as v1 to get timestamps
        let TimeParams::Timestamp { from: from_ts, to: to_ts } = time;

        // Convert to milliseconds strings (Datadog expects string format for v2)
        let from = (from_ts * 1000).to_string();
        let to = (to_ts * 1000).to_string();

        let query = params["query"]
            .as_str()
            .unwrap_or("*")
            .to_string();

    // Parse compute parameters - MUST have type field
    let compute = if let Some(compute_params) = params["compute"].as_array() {
        if compute_params.is_empty() {
            // Empty array: use default compute to avoid Datadog API error
            Some(vec![LogsCompute {
                aggregation: "count".to_string(),
                compute_type: Some("total".to_string()),
                interval: None,
                metric: None,
            }])
        } else {
            Some(compute_params.iter().map(|c| {
                LogsCompute {
                    aggregation: c["aggregation"].as_str().unwrap_or("count").to_string(),
                    compute_type: Some(c["type"].as_str().unwrap_or("total").to_string()),
                    interval: c["interval"].as_str().map(|s| s.to_string()),
                    metric: c["metric"].as_str().map(|s| s.to_string()),
                }
            }).collect::<Vec<_>>())
        }
    } else {
        // Missing compute parameter: use default
        Some(vec![LogsCompute {
            aggregation: "count".to_string(),
            compute_type: Some("total".to_string()),
            interval: None,
            metric: None,
        }])
    };

    // Parse group_by parameters with required type field
        let group_by = params["group_by"].as_array().map(|group_by_params| {
            group_by_params.iter().map(|g| {
                let sort = g["sort"].as_object().map(|sort_params| LogsGroupBySort {
                    order: sort_params["order"].as_str().map(|s| s.to_string()),
                    sort_type: Some(sort_params["type"].as_str().unwrap_or("measure").to_string()), // Required
                    aggregation: sort_params["aggregation"].as_str().map(|s| s.to_string()),
                    metric: sort_params["metric"].as_str().map(|s| s.to_string()),
                });

                LogsGroupBy {
                    facet: g["facet"].as_str().unwrap_or("status").to_string(),
                    limit: g["limit"].as_i64().map(|l| l as i32),
                    sort,
                    group_type: Some(g["type"].as_str().unwrap_or("facet").to_string()), // Required
                }
            }).collect::<Vec<_>>()
        });

        let timezone = params["timezone"].as_str().map(|s| s.to_string());

        let response = client.aggregate_logs(
            &query,
            &from,
            &to,
            compute.clone(),
            group_by.clone(),
            timezone.clone(),
        ).await?;

        let data = response["data"].clone();
        let buckets_count = data.get("buckets").and_then(|b| b.as_array()).map(|b| b.len()).unwrap_or(0);

        let meta = json!({
            "query": query,
            "from": from,
            "to": to,
            "raw_response": response,
            "buckets_count": buckets_count,
            "timezone": timezone,
            "request_compute": compute,
            "request_group_by": group_by
        });

        Ok(handler.format_list(data, None, Some(meta)))
    }
}
