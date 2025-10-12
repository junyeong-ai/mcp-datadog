use serde_json::{Value, json};
use std::sync::Arc;

use crate::datadog::{
    DatadogClient,
    models::{LogsCompute, LogsGroupBy},
};
use crate::error::Result;
use crate::handlers::common::{ResponseFormatter, TimeHandler, TimeParams};

pub struct LogsTimeseriesHandler;

impl TimeHandler for LogsTimeseriesHandler {}
impl ResponseFormatter for LogsTimeseriesHandler {}

impl LogsTimeseriesHandler {
    pub async fn timeseries(client: Arc<DatadogClient>, params: &Value) -> Result<Value> {
        let handler = LogsTimeseriesHandler;

        // Use v1 API time parsing to get timestamps, then convert to milliseconds strings
        let time = handler.parse_time(params, 1)?; // Parse as v1 to get timestamps
        let TimeParams::Timestamp {
            from: from_ts,
            to: to_ts,
        } = time;

        // Convert to milliseconds strings (Datadog expects string format for v2)
        let from = (from_ts * 1000).to_string();
        let to = (to_ts * 1000).to_string();

        let query = params["query"].as_str().unwrap_or("*").to_string();

        let interval = params["interval"].as_str().unwrap_or("1h");
        let metric = params["metric"].as_str();
        let aggregation = params["aggregation"].as_str().unwrap_or("count");

        // Create timeseries compute with required type field
        let compute = vec![LogsCompute {
            aggregation: aggregation.to_string(),
            compute_type: Some("timeseries".to_string()), // Required
            interval: Some(interval.to_string()),
            metric: metric.map(|s| s.to_string()),
        }];

        // Parse group_by if provided with required type field
        let group_by = params["group_by"].as_array().map(|group_by_params| {
            group_by_params
                .iter()
                .map(|g| LogsGroupBy {
                    facet: g["facet"].as_str().unwrap_or("status").to_string(),
                    limit: g["limit"].as_i64().map(|l| l as i32),
                    sort: None, // Timeseries typically don't use sort
                    group_type: Some(g["type"].as_str().unwrap_or("facet").to_string()), // Required
                })
                .collect::<Vec<_>>()
        });

        let timezone = params["timezone"].as_str().map(|s| s.to_string());

        let response = client
            .aggregate_logs(
                &query,
                &from,
                &to,
                Some(compute),
                group_by,
                timezone.clone(),
            )
            .await?;

        let data = response["data"].clone();
        let buckets_count = data
            .get("buckets")
            .and_then(|b| b.as_array())
            .map(|b| b.len())
            .unwrap_or(0);

        let meta = json!({
            "query": query,
            "from": from,
            "to": to,
            "interval": interval,
            "aggregation": aggregation,
            "metric": metric,
            "buckets_count": buckets_count,
            "timezone": timezone
        });

        Ok(handler.format_list(data, None, Some(meta)))
    }
}
