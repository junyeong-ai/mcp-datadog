use log::error;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::RwLock;

use crate::cache::DataCache;
use crate::datadog::DatadogClient;
use crate::error::Result;

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub method: String,
    pub params: Option<Value>,
    pub id: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct InitializeRequest {
    #[serde(alias = "protocolVersion")]
    pub protocol_version: String,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

pub struct Server {
    pub client: Arc<DatadogClient>,
    pub cache: Arc<DataCache>,
    pub initialized: Arc<RwLock<bool>>,
}

impl Server {
    /// Create a standardized error response
    pub fn create_error_response(code: i32, message: String, id: Option<Value>) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data: None,
            }),
            id,
        }
    }

    /// Create a standardized success response
    pub fn create_success_response(result: Value, id: Option<Value>) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    pub fn new(api_key: String, app_key: String, site: Option<String>) -> Result<Self> {
        let client = match DatadogClient::new(api_key, app_key, site) {
            Ok(c) => Arc::new(c),
            Err(e) => return Err(e),
        };
        let cache = Arc::new(DataCache::new(300)); // 5 minutes TTL
        Ok(Self {
            client,
            cache,
            initialized: Arc::new(RwLock::new(false)),
        })
    }

    pub async fn run(self) -> Result<()> {
        // Use async I/O for better compatibility
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut stdout = stdout;

        // Spawn background cache cleanup task
        let cache_clone = self.cache.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                let removed = cache_clone.cleanup_all_expired().await;
                if removed > 0 {
                    log::info!("Cache cleanup: removed {} expired entries", removed);
                }
            }
        });

        let mut buffer = String::new();
        let mut empty_reads = 0;

        loop {
            buffer.clear();

            // Read a line from stdin
            let line = match reader.read_line(&mut buffer).await {
                Ok(0) => {
                    empty_reads += 1;
                    if empty_reads > 3 {
                        break;
                    }
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    continue;
                }
                Ok(_) => {
                    empty_reads = 0; // Reset counter on successful read
                    buffer.trim()
                }
                Err(_) => continue,
            };

            if line.is_empty() {
                continue;
            }

            // Parse JSON-RPC request
            let request: JsonRpcRequest = match serde_json::from_str(line) {
                Ok(req) => req,
                Err(e) => {
                    // Send error response if we can extract an id
                    if let Ok(partial) = serde_json::from_str::<serde_json::Value>(line)
                        && let Some(id) = partial.get("id")
                    {
                        let mut error_response = Self::create_error_response(
                            -32700,
                            "Parse error".to_string(),
                            Some(id.clone()),
                        );
                        // Add details for parse errors
                        if let Some(error) = &mut error_response.error {
                            error.data = Some(json!({"details": e.to_string()}));
                        }
                        if let Ok(response_str) = serde_json::to_string(&error_response) {
                            let _ = stdout.write_all(response_str.as_bytes()).await;
                            let _ = stdout.write_all(b"\n").await;
                            let _ = stdout.flush().await;
                        }
                    }
                    continue;
                }
            };

            // Process the request
            match self.process_request(request).await {
                Ok(Some(response)) => {
                    let response_str = match serde_json::to_string(&response) {
                        Ok(s) => s,
                        Err(_) => continue,
                    };

                    // Try to write response, if it fails the client probably disconnected
                    if stdout.write_all(response_str.as_bytes()).await.is_err()
                        || stdout.write_all(b"\n").await.is_err()
                        || stdout.flush().await.is_err()
                    {
                        break;
                    }
                }
                Ok(None) => {
                    // This was a notification, no response needed
                }
                Err(e) => {
                    error!("Request processing error: {}", e);
                    // Send error response
                    let error_response = Self::create_error_response(-32603, e.to_string(), None);

                    if let Ok(response_str) = serde_json::to_string(&error_response) {
                        let _ = stdout.write_all(response_str.as_bytes()).await;
                        let _ = stdout.write_all(b"\n").await;
                        let _ = stdout.flush().await;
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn process_request(
        &self,
        request: JsonRpcRequest,
    ) -> Result<Option<JsonRpcResponse>> {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(&request).await,
            "initialized" | "notifications/initialized" => self.handle_initialized(&request).await,
            "tools/list" => self.handle_tools_list(&request).await,
            "tools/call" => self.handle_tool_call(&request).await,
            "prompts/list" => {
                let response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: Some(json!({
                        "prompts": []
                    })),
                    error: None,
                    id: request.id,
                };
                Ok(Some(response))
            }
            "resources/list" => {
                let response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: Some(json!({
                        "resources": []
                    })),
                    error: None,
                    id: request.id,
                };
                Ok(Some(response))
            }
            "shutdown" => {
                let response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: Some(json!({})),
                    error: None,
                    id: request.id,
                };
                Ok(Some(response))
            }
            "exit" => {
                // Exit is a notification, no response
                Ok(None)
            }
            "notifications/cancelled" | "notifications/progress" => {
                // Notifications don't get responses
                Ok(None)
            }
            _ => {
                let error = JsonRpcError {
                    code: -32601,
                    message: format!("Method not found: {}", request.method),
                    data: None,
                };
                let response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(error),
                    id: request.id,
                };
                Ok(Some(response))
            }
        }
    }

    pub async fn handle_initialize(
        &self,
        request: &JsonRpcRequest,
    ) -> Result<Option<JsonRpcResponse>> {
        // Parse initialize params
        let params: InitializeRequest = match &request.params {
            Some(p) => match serde_json::from_value(p.clone()) {
                Ok(params) => params,
                Err(e) => {
                    let error_response = JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32602,
                            message: format!("Invalid params: {}", e),
                            data: None,
                        }),
                        id: request.id.clone(),
                    };
                    return Ok(Some(error_response));
                }
            },
            None => {
                let error_response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Missing params".to_string(),
                        data: None,
                    }),
                    id: request.id.clone(),
                };
                return Ok(Some(error_response));
            }
        };

        // Return the same protocol version the client requested
        let protocol_version = params.protocol_version.clone();

        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(json!({
                "protocolVersion": protocol_version,
                "serverInfo": {
                    "name": "datadog-mcp-server",
                    "version": "0.1.0"
                },
                "capabilities": {
                    "tools": {}
                }
            })),
            error: None,
            id: request.id.clone(),
        };
        Ok(Some(response))
    }

    pub async fn handle_initialized(
        &self,
        _request: &JsonRpcRequest,
    ) -> Result<Option<JsonRpcResponse>> {
        // Set initialized state
        {
            let mut initialized = self.initialized.write().await;
            *initialized = true;
        }

        // Notifications don't get responses
        Ok(None)
    }
}
