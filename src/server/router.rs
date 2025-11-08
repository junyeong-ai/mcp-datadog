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
            "datadog_rum_events_search" => {
                handlers::rum::RumHandler::search_events(self.client.clone(), arguments).await
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::DataCache;
    use crate::datadog::DatadogClient;
    use serde_json::json;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn create_test_server() -> Server {
        let client =
            DatadogClient::new("test_key".to_string(), "test_app_key".to_string(), None).unwrap();
        let cache = Arc::new(DataCache::new(300));
        Server {
            client: Arc::new(client),
            cache,
            initialized: Arc::new(RwLock::new(true)),
        }
    }

    #[tokio::test]
    async fn test_route_without_initialization() {
        let mut server = create_test_server();
        server.initialized = Arc::new(RwLock::new(false));

        let request = JsonRpcRequest {
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "datadog_metrics_query",
                "arguments": {
                    "query": "avg:cpu{*}",
                    "from": "1 hour ago",
                    "to": "now"
                }
            })),
            id: Some(json!(1)),
        };

        let response = server.handle_tool_call(&request).await.unwrap();
        assert!(response.is_some());

        let resp = response.unwrap();
        assert!(resp.error.is_some());
        let error = resp.error.unwrap();
        assert_eq!(error.code, -32002);
        assert!(error.message.contains("not initialized"));
    }

    #[tokio::test]
    async fn test_route_missing_params() {
        let server = create_test_server();

        let request = JsonRpcRequest {
            method: "tools/call".to_string(),
            params: None,
            id: Some(json!(1)),
        };

        let response = server.handle_tool_call(&request).await.unwrap();
        assert!(response.is_some());

        let resp = response.unwrap();
        assert!(resp.error.is_some());
        let error = resp.error.unwrap();
        assert_eq!(error.code, -32602);
        assert!(error.message.contains("Missing params"));
    }

    #[tokio::test]
    async fn test_route_missing_tool_name() {
        let server = create_test_server();

        let request = JsonRpcRequest {
            method: "tools/call".to_string(),
            params: Some(json!({
                "arguments": {}
            })),
            id: Some(json!(1)),
        };

        let response = server.handle_tool_call(&request).await.unwrap();
        assert!(response.is_some());

        let resp = response.unwrap();
        assert!(resp.error.is_some());
        let error = resp.error.unwrap();
        assert_eq!(error.code, -32602);
        assert!(error.message.contains("Missing tool name"));
    }

    #[tokio::test]
    async fn test_route_unknown_tool_error() {
        let server = create_test_server();

        let request = JsonRpcRequest {
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "datadog_unknown_tool",
                "arguments": {}
            })),
            id: Some(json!(1)),
        };

        let response = server.handle_tool_call(&request).await.unwrap();
        assert!(response.is_some());

        let resp = response.unwrap();
        assert!(resp.error.is_some());
        let error = resp.error.unwrap();
        assert_eq!(error.code, -32602);
        assert!(error.message.contains("Unknown tool"));
        assert!(error.message.contains("datadog_unknown_tool"));
    }

    #[tokio::test]
    async fn test_route_with_missing_required_argument() {
        let server = create_test_server();

        let request = JsonRpcRequest {
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "datadog_metrics_query",
                "arguments": {
                    "from": "1 hour ago",
                    "to": "now"
                }
            })),
            id: Some(json!(1)),
        };

        let response = server.handle_tool_call(&request).await.unwrap();
        assert!(response.is_some());

        let resp = response.unwrap();
        assert!(resp.result.is_some());
        let result = resp.result.unwrap();
        assert!(result.get("content").is_some());

        let content = &result["content"][0]["text"];
        let text = content.as_str().unwrap();
        assert!(text.contains("Error") || text.contains("query"));
    }

    #[tokio::test]
    async fn test_route_response_format() {
        let server = create_test_server();

        let request = JsonRpcRequest {
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "datadog_unknown_tool",
                "arguments": {}
            })),
            id: Some(json!(42)),
        };

        let response = server.handle_tool_call(&request).await.unwrap();
        assert!(response.is_some());

        let resp = response.unwrap();
        assert_eq!(resp.jsonrpc, "2.0");
        assert_eq!(resp.id, Some(json!(42)));
    }
}
