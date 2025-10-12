use serde_json::{Value, json};
use std::sync::Arc;

use crate::datadog::DatadogClient;
use crate::error::Result;
use crate::handlers::common::{ResponseFormatter, TimeHandler, TimeParams};

pub struct MetricsHandler;

impl TimeHandler for MetricsHandler {}
impl ResponseFormatter for MetricsHandler {}

impl MetricsHandler {
    // Calculate rollup interval based on time range and desired max_points
    fn calculate_rollup_interval(from_ts: i64, to_ts: i64, max_points: usize) -> i64 {
        let time_range = to_ts - from_ts;
        let interval = time_range / max_points as i64;

        // Round up to reasonable intervals: 60s, 300s (5m), 600s (10m), 3600s (1h), etc.
        if interval < 60 {
            60
        } else if interval < 300 {
            300
        } else if interval < 600 {
            600
        } else if interval < 1800 {
            1800
        } else if interval < 3600 {
            3600
        } else if interval < 7200 {
            7200
        } else if interval < 21600 {
            21600
        } else if interval < 43200 {
            43200
        } else {
            86400 // 1 day max
        }
    }

    // Add rollup to query if needed
    fn add_rollup_to_query(query: &str, interval: i64) -> String {
        // Check if query already has rollup
        if query.contains(".rollup(") {
            return query.to_string();
        }

        // Extract aggregation method from query (avg:, max:, min:, sum:)
        let agg = if query.starts_with("avg:") {
            "avg"
        } else if query.starts_with("max:") {
            "max"
        } else if query.starts_with("min:") {
            "min"
        } else if query.starts_with("sum:") {
            "sum"
        } else {
            "avg" // default
        };

        format!("{}.rollup({}, {})", query, agg, interval)
    }

    pub async fn query(client: Arc<DatadogClient>, params: &Value) -> Result<Value> {
        let handler = MetricsHandler;

        let mut query = params["query"].as_str().ok_or_else(|| {
            crate::error::DatadogError::InvalidInput("Missing 'query' parameter".to_string())
        })?.to_string();

        let time = handler.parse_time(params, 1)?; // v1 API

        let TimeParams::Timestamp {
            from: from_ts,
            to: to_ts,
        } = time;

        // Get max_points parameter and apply rollup at API level
        let max_points = params["max_points"].as_i64().map(|p| p as usize);
        let mut applied_rollup = false;

        if let Some(max) = max_points {
            let interval = Self::calculate_rollup_interval(from_ts, to_ts, max);
            query = Self::add_rollup_to_query(&query, interval);
            applied_rollup = true;
        }

        let response = client.query_metrics(&query, from_ts, to_ts).await?;

        let series = response.series.iter().map(|s| {
            let points_data = if let Some(ref pointlist) = s.pointlist {
                json!({
                    "count": pointlist.len(),
                    "data": pointlist.iter().map(|p| {
                        if p.len() >= 2 {
                            json!({
                                "timestamp": p[0].map(|t| crate::utils::format_timestamp(t as i64 / 1000))
                                    .unwrap_or_else(|| "N/A".to_string()),
                                "value": p[1]
                            })
                        } else {
                            json!({
                                "timestamp": "N/A",
                                "value": null
                            })
                        }
                    }).collect::<Vec<_>>()
                })
            } else {
                json!({
                    "count": 0,
                    "data": []
                })
            };

            json!({
                "metric": s.metric,
                "scope": s.scope,
                "points": points_data,
                "unit": s.unit,
                "aggr": s.aggr,
                "interval": s.interval
            })
        }).collect::<Vec<_>>();

        let mut meta = json!({
            "query": response.query,
            "status": response.status,
            "from": crate::utils::format_timestamp(from_ts),
            "to": crate::utils::format_timestamp(to_ts),
            "error": response.error
        });

        if applied_rollup {
            meta["rollup_applied"] = json!(true);
            if let Some(max) = max_points {
                meta["requested_max_points"] = json!(max);
            }
        }

        Ok(handler.format_list(json!(series), None, Some(meta)))
    }
}
