use super::protocol::{JsonRpcRequest, JsonRpcResponse, Server};
use crate::error::Result;
use crate::handlers;
use serde_json::json;

impl Server {
    pub async fn handle_tool_call(
        &self,
        request: &JsonRpcRequest,
    ) -> Result<Option<JsonRpcResponse>> {
        // Check if initialized
        {
            let initialized = self.initialized.read().await;
            if !*initialized {
                let error_response = Self::create_error_response(
                    -32002,
                    "Server not initialized".to_string(),
                    request.id.clone(),
                );
                return Ok(Some(error_response));
            }
        }

        let params = match request.params.as_ref() {
            Some(p) => p,
            None => {
                let error_response = Self::create_error_response(
                    -32602,
                    "Missing params".to_string(),
                    request.id.clone(),
                );
                return Ok(Some(error_response));
            }
        };

        let tool_name = match params["name"].as_str() {
            Some(name) => name,
            None => {
                let error_response = Self::create_error_response(
                    -32602,
                    "Missing tool name".to_string(),
                    request.id.clone(),
                );
                return Ok(Some(error_response));
            }
        };

        let arguments = &params["arguments"];

        let result = match tool_name {
            "datadog_metrics_query" => {
                handlers::metrics::MetricsHandler::query(self.client.clone(), arguments).await
            }
            "datadog_logs_search" => {
                handlers::logs::LogsHandler::search(self.client.clone(), arguments).await
            }
            "datadog_monitors_list" => {
                handlers::monitors::MonitorsHandler::list(
                    self.client.clone(),
                    self.cache.clone(),
                    arguments,
                )
                .await
            }
            "datadog_monitors_get" => {
                handlers::monitors::MonitorsHandler::get(self.client.clone(), arguments).await
            }
            "datadog_events_query" => {
                handlers::events::EventsHandler::query(
                    self.client.clone(),
                    self.cache.clone(),
                    arguments,
                )
                .await
            }
            "datadog_hosts_list" => {
                handlers::hosts::HostsHandler::list(self.client.clone(), arguments).await
            }
            "datadog_dashboards_list" => {
                handlers::dashboards::DashboardsHandler::list(
                    self.client.clone(),
                    self.cache.clone(),
                    arguments,
                )
                .await
            }
            "datadog_dashboards_get" => {
                handlers::dashboards::DashboardsHandler::get(self.client.clone(), arguments).await
            }
            "datadog_spans_search" => {
                handlers::spans::SpansHandler::list(self.client.clone(), arguments).await
            }
            "datadog_services_list" => {
                handlers::services::ServicesHandler::list(self.client.clone(), arguments).await
            }
            "datadog_logs_aggregate" => {
                handlers::logs_aggregate::LogsAggregateHandler::aggregate(
                    self.client.clone(),
                    arguments,
                )
                .await
            }
            "datadog_logs_timeseries" => {
                handlers::logs_timeseries::LogsTimeseriesHandler::timeseries(
                    self.client.clone(),
                    arguments,
                )
                .await
            }
            _ => {
                let error_response = Self::create_error_response(
                    -32602,
                    format!("Unknown tool: {}", tool_name),
                    request.id.clone(),
                );
                return Ok(Some(error_response));
            }
        };

        let result_content = match result {
            Ok(data) => json!({
                "content": [{
                    "type": "text",
                    "text": serde_json::to_string_pretty(&data)
                        .unwrap_or_else(|_| "Error formatting response".to_string())
                }]
            }),
            Err(e) => json!({
                "content": [{
                    "type": "text",
                    "text": format!("Error: {}", e)
                }],
                "isError": true
            }),
        };

        let response = Self::create_success_response(result_content, request.id.clone());
        Ok(Some(response))
    }
}
