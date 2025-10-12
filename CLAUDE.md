# MCP Datadog Server - AI Assistant Development Guide

This comprehensive guide is for AI assistants working with the MCP Datadog Server codebase. It covers the complete architecture, implementation patterns, and development practices.

## Project Overview

The MCP Datadog Server is a high-performance Rust application (Rust 2024 edition) that bridges AI assistants with Datadog's observability platform using the Model Context Protocol (MCP). It provides comprehensive access to metrics, logs, analytics, monitoring, infrastructure data, dashboards, APM spans, and service catalogs through a clean JSON-RPC 2.0 interface over stdio.

**Technology Stack:**
- **Rust Edition:** 2024 (minimum version 1.90)
- **Environment:** dotenvy for secure environment variable loading
- **HTTP Client:** reqwest with rustls-tls
- **Async Runtime:** tokio with full features
- **Time Parsing:** interim library (supports natural language expressions)
- **Build Policy:** Zero warnings, no dead code tolerance, no unused dependencies

## Core Architecture

### System Components

1. **MCP Server (`src/server/`)**
   - Implements complete MCP protocol specification (2024-11-05)
   - Split into focused modules: protocol, schema, and router
   - `protocol.rs`: Server struct, JSON-RPC 2.0 handling, and I/O operations
   - `schema.rs`: Tool schema definitions optimized for AI agent comprehension
   - `router.rs`: Tool routing to specialized handlers
   - Manages JSON-RPC 2.0 communication over stdin/stdout
   - Provides comprehensive error handling and response formatting

2. **Datadog Client (`src/datadog/`)**
   - `client.rs`: HTTP/2 client with connection pooling and comprehensive API methods
   - `retry.rs`: Exponential backoff retry strategy (up to 3 retries, 2^n second delays)
   - 30-second timeout per request with automatic rate limit detection
   - Authentication management with API/APP key support
   - Multi-region support (datadoghq.com, datadoghq.eu, us3/us5)
   - Request/response debugging with configurable logging

3. **Intelligent Cache System (`src/cache.rs`)**
   - TTL-based caching with 5-minute expiration
   - Smart pagination: page 0 always fresh, subsequent pages cached
   - Separate cache domains for monitors and events (only APIs without server pagination)
   - Automatic cache invalidation on page 0 requests
   - Memory-efficient with configurable size limits

4. **Handler Architecture (`src/handlers/`)**
   - Trait-based design with `TimeHandler`, `Paginator`, and `ResponseFormatter`
   - Unified time parsing supporting natural language, ISO8601, and Unix timestamps
   - Consistent pagination patterns across all APIs
   - Specialized handlers for each Datadog API domain
   - Separate handlers for logs aggregation and timeseries analytics

### Data Models (`src/datadog/models.rs`)

Comprehensive type-safe models covering:
- **Metrics**: Series data with pointlist arrays and metadata
- **Logs**: Search results with attributes and pagination cursors  
- **Analytics**: Aggregation buckets, timeseries, compute operations
- **Monitors**: Monitor definitions with state and configuration
- **Events**: Event stream with priority, sources, and tags
- **Hosts**: Infrastructure data with metrics and metadata
- **Dashboards**: Dashboard definitions and widget configurations
- **Spans**: APM trace data with timing and service information
- **Services**: Service catalog with teams, links, and integrations

## Tool Implementation Details

### Current Tools (12 total)

**All tools follow the naming pattern**: `datadog_{resource}_{action}` ✅

**Tool descriptions optimized for AI agent comprehension** - Each includes action, return data, format support, and key capabilities.

1. **datadog_metrics_query**: Time series metrics with natural language time support
2. **datadog_logs_search**: Log search with filtering and limit controls
3. **datadog_logs_aggregate**: Aggregation engine with buckets and computations
4. **datadog_logs_timeseries**: Time series generation from log aggregations
5. **datadog_monitors_list**: Monitor listing with smart caching and pagination
6. **datadog_monitors_get**: Individual monitor retrieval by ID
7. **datadog_events_query**: Event stream with caching and comprehensive filtering
8. **datadog_hosts_list**: Infrastructure host listing with filtering and sorting
9. **datadog_dashboards_list**: Dashboard catalog access
10. **datadog_dashboards_get**: Individual dashboard retrieval
11. **datadog_spans_search**: APM spans with advanced filtering and cursor pagination
12. **datadog_services_list**: Service catalog with environment filtering and pagination

### Tool Naming & MCP Compliance

#### Naming Convention Decision

**Official Guidance**: The `datadog_` prefix is **RETAINED** for all tools, following Anthropic's official MCP best practices.

**Rationale** (from research and Anthropic engineering blog):
- **Conflict Prevention**: Prevents naming collisions when multiple MCP servers are active (e.g., other monitoring tools)
- **AI Agent Discovery**: The prefix enables agents to easily identify "all Datadog tools" via pattern matching
- **Industry Alignment**: Matches patterns from Anthropic's official Slack MCP server (`slack_*`) and other major providers (Firebase `firebase_*`, Atlassian `jira_*`, `confluence_*`)
- **Token Efficiency**: Despite 1-2 tokens per prefix, it saves 20-50 tokens in agent reasoning by providing immediate context
- **MCP Specification**: Complies with tool name regex `^[a-zA-Z0-9_-]{1,64}$`

**Naming Pattern**: `datadog_{resource}_{action}`
- Examples: `datadog_metrics_query`, `datadog_logs_search`, `datadog_monitors_list`
- Resource: The Datadog API domain (metrics, logs, monitors, etc.)
- Action: The operation (query, search, list, get, aggregate, etc.)

#### MCP Protocol Compliance

- **Tool Names**: Must match pattern `^[a-zA-Z0-9_-]{1,64}$` (underscores, not dots)
- **Capability Fields**: Always return empty objects `{}`, never `null`
- **Response Format**: All tool responses wrapped in content array with type "text"
- **JSON-RPC 2.0**: Strict compliance with id handling and error codes
- **Schema Validation**: Input schemas define required/optional parameters with defaults

### Time Handling Architecture

Unified time parsing through `TimeHandler` trait:

```rust
// Natural language parsing chain (using interim library)
"1 hour ago" → interim → DateTime → Unix timestamp
"yesterday" → interim → DateTime → Unix timestamp
"2024-01-15T10:30:00Z" → ISO8601 parse → Unix timestamp

// API-specific conversion
v1 APIs: Use timestamps directly
v2 APIs: Convert to ISO8601 strings or millisecond strings
```

### Caching Strategy Implementation

```rust
// Smart caching logic
if page == 0 || force_refresh {
    // Always fetch fresh data for first page
    let fresh_data = api_call().await?;
    cache.set(key, fresh_data.clone()).await;
    fresh_data
} else {
    // Use cache for subsequent pages
    match cache.get(&key).await {
        Some(cached) => cached,
        None => {
            let fresh_data = api_call().await?;
            cache.set(key, fresh_data.clone()).await;
            fresh_data
        }
    }
}
```

### Tag Filtering System

Unified tag filtering across logs, spans, and hosts to reduce response sizes.

**Three Filter Modes**:
1. `"*"` (default) - Return all tags
2. `""` (empty) - Return no tags
3. `"env:,service:"` - Return only tags with matching prefixes (comma-separated)

**Priority Chain**:
```rust
let tag_filter = params["tag_filter"]           // 1. Request parameter
    .as_str()
    .or_else(|| client.get_tag_filter())        // 2. DD_TAG_FILTER env var
    .unwrap_or("*");                            // 3. Default (all tags)
```

**Implementation Pattern**:
```rust
let tag_filter = params["tag_filter"]
    .as_str()
    .or_else(|| client.get_tag_filter())
    .unwrap_or("*");

let filtered_tags = match tag_filter {
    "*" => tags.clone(),
    "" => vec![],
    filter => {
        let prefixes: Vec<&str> = filter.split(',').map(str::trim).collect();
        tags.iter()
            .filter(|tag| prefixes.iter().any(|p| tag.starts_with(p)))
            .cloned()
            .collect()
    }
};
```

**Supported Tools**:
- `datadog_logs_search` - Filters tags array
- `datadog_spans_search` - Filters attributes.tags array
- `datadog_hosts_list` - Filters tags_by_source map

**Environment Variable**:
```bash
DD_TAG_FILTER="env:,service:,kube_namespace:"  # Global default
```

**Usage Example**:
```json
{
  "name": "datadog_spans_search",
  "arguments": {
    "from": "1 hour ago",
    "to": "now",
    "tag_filter": "env:,service:"
  }
}
```

## Development Patterns

### Adding New Tools

1. **Define Schema** in `src/server/schema.rs::handle_tools_list()`:
```rust
{
    "name": "datadog_new_tool",
    "description": "Clear, concise description",
    "inputSchema": {
        "type": "object",
        "properties": {
            "required_param": {
                "type": "string",
                "description": "Parameter description"
            },
            "optional_param": {
                "type": "integer", 
                "description": "Optional parameter",
                "default": 100
            }
        },
        "required": ["required_param"]
    }
}
```

2. **Create Handler** in `src/handlers/new_tool.rs`:
```rust
use std::sync::Arc;
use serde_json::{json, Value};
use crate::datadog::DatadogClient;
use crate::error::Result;
use crate::handlers::common::{TimeHandler, Paginator, ResponseFormatter};

pub struct NewToolHandler;

impl TimeHandler for NewToolHandler {}
impl Paginator for NewToolHandler {}  
impl ResponseFormatter for NewToolHandler {}

impl NewToolHandler {
    pub async fn execute(
        client: Arc<DatadogClient>,
        params: &Value,
    ) -> Result<Value> {
        let handler = NewToolHandler;
        
        // Parse required parameters
        let required_param = params["required_param"]
            .as_str()
            .ok_or_else(|| crate::error::DatadogError::InvalidInput("Missing required_param".to_string()))?;
            
        // Parse optional parameters with defaults
        let optional_param = params["optional_param"]
            .as_i64()
            .unwrap_or(100);
            
        // Time parsing if needed
        let time = handler.parse_time(params, 1)?; // 1 for v1 API, 2 for v2
        let TimeParams::Timestamp { from, to } = time;
        
        // API call
        let response = client.new_api_call(required_param, optional_param).await?;
        
        // Response formatting
        let data = json!(/* format response data */);
        let meta = json!(/* format metadata */);
        
        Ok(handler.format_list(data, None, Some(meta)))
    }
}
```

3. **Register Handler** in `src/server/router.rs::handle_tool_call()`:
```rust
"datadog_new_tool" => handlers::new_tool::NewToolHandler::execute(self.client.clone(), arguments).await,
```

4. **Add Client Method** in `src/datadog/client.rs`:
```rust
pub async fn new_api_call(&self, param: &str, limit: i64) -> Result<NewApiResponse> {
    let params = vec![
        ("param", param.to_string()),
        ("limit", limit.to_string()),
    ];

    self.request(
        reqwest::Method::GET,
        "/api/v1/new-endpoint",
        Some(params),
        None::<()>,
    ).await
}
```

5. **Define Models** in `src/datadog/models.rs`:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewApiResponse {
    pub data: Vec<NewDataItem>,
    pub meta: Option<NewMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]  
pub struct NewDataItem {
    pub id: Option<String>,
    pub name: Option<String>,
    // Add other fields as needed
}
```

### Client Implementation Patterns

All Datadog API calls follow this pattern:

```rust
pub async fn api_method(
    &self,
    param1: &str,
    param2: Option<i64>,
) -> Result<ResponseType> {
    let mut params = vec![
        ("required_param", param1.to_string()),
    ];
    
    if let Some(optional) = param2 {
        params.push(("optional_param", optional.to_string()));
    }

    self.request(
        reqwest::Method::GET, // or POST
        "/api/v1/endpoint",
        if params.is_empty() { None } else { Some(params) },
        None::<()>, // or Some(body) for POST
    ).await
}
```

### Error Handling Patterns

Use comprehensive error types from `src/error.rs`:

```rust
#[derive(Error, Debug)]
pub enum DatadogError {
    #[error("API request failed: {0}")]
    ApiError(String),
    
    #[error("Authentication failed: {0}")]
    AuthError(String),
    
    #[error("Invalid date format: {0}")]
    DateParseError(String),
    
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Rate limit exceeded")]
    RateLimitError,
    
    #[error("Timeout occurred")]
    TimeoutError,
}
```

## Testing & Validation

### Manual Testing Protocol

```bash
# 1. MCP Protocol Compliance
echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05"},"id":0}' | cargo run

# 2. Tool Listing
echo '{"jsonrpc":"2.0","method":"tools/list","id":1}' | cargo run

# 3. Tool Execution (metrics example)
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"datadog_metrics_query","arguments":{"query":"avg:system.cpu.user{*}","from":"1 hour ago"}},"id":2}' | cargo run

# 4. Error Handling
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"datadog_metrics_query","arguments":{}},"id":3}' | cargo run
```

### Development Testing

```bash
# Build and test cycle
cargo build && cargo test

# Lint and format checking  
cargo clippy
cargo fmt --check

# Release build testing
cargo build --release
./target/release/mcp-datadog

# Debug logging for development
LOG_LEVEL=debug DD_API_KEY=xxx DD_APP_KEY=yyy cargo run
```

### API Response Debugging

Enable debug logging to inspect API interactions:

```bash
LOG_LEVEL=debug cargo run
```

This shows:
- Request URLs and parameters
- Response status codes and headers
- Full JSON response bodies
- Cache hit/miss statistics
- Time parsing results

## Code Quality Standards

### Rust Best Practices

1. **Error Handling**: Use `Result<T>` for all fallible operations
2. **Memory Safety**: Leverage Arc<> for shared client instances
3. **Async Patterns**: Full async/await throughout with tokio runtime
4. **Type Safety**: Strongly typed API models with serde serialization
5. **Performance**: Connection pooling, caching, and efficient data structures

### Code Style Guidelines

1. **Zero-Warning Policy**: Build must complete with 0 warnings (`cargo build` clean output)
2. **No Dead Code**: Remove all unused code immediately, no `#[allow(dead_code)]` exceptions
3. **YAGNI Principle**: Don't build infrastructure for future use, build only what's needed now
4. **No Comments**: Code should be self-documenting through clear naming
5. **Minimal Logging**: Only log errors and debug info, default to `warn` level
6. **Clean Responses**: No debug fields, no duplicate data, no request echo fields in production responses
7. **Consistent Formatting**: Use `cargo fmt` for all code
8. **Trait-based Design**: Common functionality through shared traits

### Security Requirements

1. **Credential Safety**: Never log API keys or sensitive data
2. **Read-Only Operations**: All operations must be strictly read-only
3. **Input Validation**: Validate all parameters before API calls
4. **Error Messages**: Don't expose internal details in error messages

## Performance Optimization

### Memory Management

- **Connection Pooling**: Reuse HTTP connections efficiently via reqwest
- **Cache Design**: TTL-based with automatic expiration (5 minutes)
- **Async Architecture**: Tokio runtime with Arc/RwLock for shared state
- **Type Safety**: Strong typing prevents runtime allocation errors

### Network Optimization

- **HTTP/2**: Use modern protocol features
- **Compression**: Enable gzip/brotli compression
- **Retries**: Exponential backoff with jitter
- **Rate Limiting**: Respect Datadog's rate limits automatically

### Caching Effectiveness

- **TTL Strategy**: 5-minute TTL balances freshness with performance
- **Selective Caching**: Only cache APIs without server-side pagination (monitors, events)
- **Cache Invalidation**: Page 0 always fetches fresh data
- **Async Operations**: Non-blocking cache operations with RwLock

## Response Structure

All tool responses follow a consistent, optimized structure with essential data only:

```json
{
  "data": { /* Core response data */ },
  "meta": { /* Essential metadata: query, from, to, counts, etc. */ },
  "pagination": { /* Only for paginated endpoints */ }
}
```

**Design Principles:**
- **Essential Data Only**: Responses contain only data needed by AI agents
- **Consistent Structure**: All tools use the same response format
- **Efficient Transport**: Optimized payloads for MCP protocol communication
- **Clean Metadata**: Essential metadata fields (query, time range, counts) included in meta object

**Example Response (logs_aggregate):**
```json
{
  "data": {
    "buckets": [
      {
        "by": {},
        "computes": { "c0": 11519456 }
      }
    ]
  },
  "meta": {
    "buckets_count": 1,
    "from": "1760254364000",
    "to": "1760257964000",
    "query": "*",
    "timezone": null
  }
}
```

## Troubleshooting Common Issues

### "Server transport closed unexpectedly"

**Cause**: MCP protocol compliance issues
**Solution**: 
- Verify capability fields return `{}` not `null`
- Check tool names match pattern `^[a-zA-Z0-9_-]{1,64}$`
- Ensure all JSON-RPC responses are valid
- Validate tool schema definitions

### API Response Parsing Errors

**Cause**: Model definitions don't match actual API responses
**Solution**:
- Update models to match real API responses
- Make fields `Option<T>` for nullable/optional data
- Add `#[serde(default)]` for missing fields
- Use `#[serde(flatten)]` for dynamic fields

### Rate Limit Errors

**Cause**: Exceeding Datadog API rate limits
**Solution**:
- Client automatically handles retries with exponential backoff
- Monitor rate limit headers in debug logs
- Implement request queuing for high-volume scenarios
- Use caching more aggressively

### Large Response Issues

**Cause**: API responses too large for MCP transport
**Solution**:
- Implement proper pagination with reasonable page sizes
- Limit response data to essential fields only
- Use streaming for large datasets
- Add response size warnings in debug logs

### Time Parsing Failures

**Cause**: Unsupported time format or invalid time expression
**Solution**:
- Support natural language via interim library
- Handle ISO8601, Unix timestamps, and relative expressions
- Provide clear error messages with format examples
- Add fallback to default times (e.g., "1 hour ago")

## Environment Configuration

### Required Variables
```bash
DD_API_KEY=your_datadog_api_key       # Required for authentication
DD_APP_KEY=your_datadog_app_key       # Required for API access
```

### Optional Variables
```bash
DD_SITE=datadoghq.com                 # Default site, supports all regions
LOG_LEVEL=warn                        # Logging level (trace/debug/info/warn/error)
DD_TAG_FILTER="env:,service:"         # Global tag filter (comma-separated prefixes)
```

**Tag Filter Options**:
- `"*"` or unset - Include all tags (default)
- `""` - Exclude all tags
- `"env:,service:,kube_namespace:"` - Include only matching prefixes

### Runtime Configuration
- **Timeout**: 30 seconds per API request
- **Retries**: Maximum 3 retries with exponential backoff
- **Cache TTL**: 300 seconds (5 minutes)
- **Page Size**: 50 items default for paginated APIs

## File Structure Reference

```
src/
├── main.rs                    # Entry point with tokio runtime setup
├── cache.rs                   # TTL cache with smart invalidation
├── error.rs                   # Comprehensive error types
├── utils.rs                   # Time parsing (interim library)
├── server/
│   ├── mod.rs                 # Server module exports
│   ├── protocol.rs            # MCP protocol implementation and I/O
│   ├── schema.rs              # Tool schema definitions
│   └── router.rs              # Tool routing to handlers
├── datadog/
│   ├── mod.rs                 # Module exports
│   ├── client.rs              # HTTP client with API methods
│   ├── retry.rs               # Retry strategy implementation
│   └── models.rs              # API response models
└── handlers/
    ├── mod.rs                 # Handler module exports
    ├── common.rs              # Shared traits and utilities
    ├── metrics.rs             # Metrics query handler
    ├── logs.rs                # Log search handler
    ├── logs_aggregate.rs      # Log aggregation handler
    ├── logs_timeseries.rs     # Log timeseries handler
    ├── monitors.rs            # Monitor management handler
    ├── events.rs              # Event stream handler
    ├── hosts.rs               # Infrastructure handler
    ├── dashboards.rs          # Dashboard access handler
    ├── spans.rs               # APM spans handler
    └── services.rs            # Service catalog handler
```

## Build and Release Process

### Development Build
```bash
cargo build                           # Debug build with symbols
cargo build --release                 # Optimized release build
```

### Quality Assurance
```bash
cargo test                            # Run unit tests
cargo clippy                          # Lint checking  
cargo fmt --check                     # Format checking
```

### Production Deployment
```bash
cargo build --release                 # Create optimized binary
strip target/release/mcp-datadog      # Remove symbols (optional)
```

### Binary Size Optimization
- **Release Build**: ~5.3MB (with LTO enabled)
- **Strip Symbols**: Use `strip` command for further reduction
- **Cargo Profile**: LTO enabled, codegen-units=1, opt-level=3
- **Dependencies**: Minimal dependency tree with feature flags

## AI Assistant Notes

When working on this codebase:

1. **Maintain MCP Compatibility**: Always test protocol compliance after changes
2. **Preserve API Contracts**: Tool interfaces should remain backwards compatible
3. **Focus on Performance**: Caching and pagination are critical for user experience
4. **Test with Real APIs**: Always validate against actual Datadog APIs, not mocks
5. **Security First**: Never log credentials or expose sensitive data
6. **Documentation**: Update tool schemas when adding new parameters
7. **Error Handling**: Provide clear, actionable error messages to users
8. **Type Safety**: Use Rust's type system to prevent runtime errors
9. **Async Patterns**: Maintain non-blocking operations throughout
10. **Resource Management**: Be mindful of memory usage and connection limits

## Common Development Commands

```bash
# Development workflow
cargo watch -x 'build'                # Auto-rebuild on changes
cargo watch -x 'test'                 # Auto-test on changes

# Testing with real API
DD_API_KEY=xxx DD_APP_KEY=yyy LOG_LEVEL=debug cargo run

# Protocol testing  
echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05"},"id":0}' | cargo run

# Performance profiling
cargo build --release && time ./target/release/mcp-datadog

# Memory usage analysis
valgrind --tool=massif ./target/release/mcp-datadog

# Binary analysis
objdump -t target/release/mcp-datadog | grep -E "(metrics|logs|monitors)"
```

This guide provides comprehensive coverage of the MCP Datadog Server architecture and development practices. It should serve as the definitive reference for AI assistants working on this codebase.
