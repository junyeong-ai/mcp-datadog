use serde_json::{Value, json};
use std::sync::Arc;

use crate::datadog::DatadogClient;
use crate::error::Result;
use crate::handlers::common::{ResponseFormatter, TimeHandler, TimeParams};

pub struct MetricsHandler;

impl TimeHandler for MetricsHandler {}
impl ResponseFormatter for MetricsHandler {}

impl MetricsHandler {
    pub async fn query(client: Arc<DatadogClient>, params: &Value) -> Result<Value> {
        let handler = MetricsHandler;

        let query = params["query"].as_str().ok_or_else(|| {
            crate::error::DatadogError::InvalidInput("Missing 'query' parameter".to_string())
        })?;

        let time = handler.parse_time(params, 1)?; // v1 API

        let TimeParams::Timestamp {
            from: from_ts,
            to: to_ts,
        } = time;

        let response = client.query_metrics(query, from_ts, to_ts).await?;

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

        let meta = json!({
            "query": response.query,
            "status": response.status,
            "from": crate::utils::format_timestamp(from_ts),
            "to": crate::utils::format_timestamp(to_ts),
            "error": response.error
        });

        Ok(handler.format_list(json!(series), None, Some(meta)))
    }
}
