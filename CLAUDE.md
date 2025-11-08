# MCP Datadog Server - Development Guide

Clean, optimized AI assistant guide for maintaining and extending the MCP Datadog Server.

## Project Overview

High-performance Rust 2024 MCP server providing AI agents with Datadog observability access via JSON-RPC 2.0 over stdio.

**Stack:**
- Rust 2024 (1.90+), tokio async runtime, reqwest HTTP/2 client
- dotenvy (env), interim (time parsing), serde (serialization)
- **Policy:** Zero warnings, zero unsafe code, zero dead code

## Architecture

### Core Components

**MCP Server** (`src/server/`): JSON-RPC 2.0 protocol handler
- `protocol.rs`: I/O, request processing, initialization
- `schema.rs`: Tool schemas for AI comprehension
- `router.rs`: Route tools to handlers

**Datadog Client** (`src/datadog/client.rs`): HTTP/2 API client
- Connection pooling, 30s timeout, 3 retries with exponential backoff
- Multi-region support, automatic rate limit handling
- `retry.rs`: Backoff strategy (2^n seconds)

**Cache System** (`src/cache.rs`): Arc-based TTL cache
- **Returns `Arc<T>` instead of cloning** - 99.9% memory reduction
- 5-minute TTL, page 0 always fresh, LRU eviction
- Only for non-paginated APIs (monitors, events, dashboards)

**Handlers** (`src/handlers/`): Trait-based tool implementations
- `TimeHandler`: Unified time parsing (natural language, ISO8601, Unix)
- `TagFilter`: Unified tag filtering across logs/spans/hosts
- `ResponseFilter`: Response optimization (stack trace truncation, field filtering)
- `PaginationInfo`: Unified pagination structure (single_page, from_offset, from_cursor)
- `Paginator`: Client-side pagination logic
- `ResponseFormatter`: Consistent JSON response structure

### Data Flow

```
stdin → protocol.rs → router.rs → handler → client → Datadog API
                                     ↓
                                   cache (Arc<T>)
                                     ↓
stdout ← format_response ← handler ← Arc::clone (cheap)
```

## Key Patterns

### 1. Arc-Based Caching (Memory Optimized)

**Implementation:**
```rust
// Cache returns Arc<T> - only Arc pointer copied
pub async fn get(&self, key: &str) -> Option<Arc<T>>

// Handlers use Arc directly
let monitors = cache.get_or_fetch_monitors(&key, || async {
    client.list_monitors(...).await
}).await?;

// Paginate with Arc deref
let slice = handler.paginate(&*monitors, page, page_size);
```

**Performance:** 100 monitors = 8KB clone → 8 bytes Arc clone

### 2. Unified Tag Filtering

**TagFilter trait** (`handlers/common.rs`):
```rust
pub trait TagFilter {
    fn filter_tags(&self, tags: &[String], filter: &str) -> Vec<String>;
    fn filter_tags_map(&self, map: Option<&HashMap<...>>, filter: &str) -> ...;
}
```

**Usage in handlers:**
```rust
impl TagFilter for LogsHandler {}

let tags = attrs.and_then(|a| a.tags.as_ref())
    .map(|t| handler.filter_tags(t, tag_filter));
```

**Filter modes:** `"*"` (all), `""` (none), `"env:,service:"` (prefixes)

### 3. Time Handling

**TimeHandler trait:**
```rust
fn parse_time(&self, params: &Value, _api_version: u8) -> Result<TimeParams>;
fn timestamp_to_iso8601(&self, timestamp: i64) -> Result<String>;
```

**Flow:** User input → interim lib → Unix timestamp → API format (v1: timestamp, v2: ISO8601)

### 4. Response Optimization

**ResponseFilter trait** - 70% size reduction for Spans API:
```rust
pub trait ResponseFilter {
    fn should_truncate_stack_trace(&self, params: &Value) -> bool;
    fn truncate_stack_trace(&self, stack: &str, max_lines: usize) -> String;
    fn filter_http_verbose_fields(&self, http: &mut Value);
    fn truncate_long_string(&self, s: &str, max_len: usize) -> String;
}
```

**PaginationInfo struct** - Unified pagination:
```rust
pub struct PaginationInfo {
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
    pub has_next: bool,
    pub next_offset: Option<usize>,
}

// Three constructors for different API types
PaginationInfo::single_page(count, limit)      // Logs (heuristic)
PaginationInfo::from_offset(total, start, count)  // Hosts
PaginationInfo::from_cursor(total, size, has_cursor)  // Spans
```

**Usage:**
```rust
impl ResponseFilter for SpansHandler {}

// In handler
if handler.should_truncate_stack_trace(params) {
    truncated = handler.truncate_stack_trace(stack, 10);
}

let pagination = PaginationInfo::from_cursor(count, size, has_cursor);
Ok(json!({ "data": data, "pagination": pagination }))
```

### 5. Error Handling

**Comprehensive types** (`src/error.rs`):
```rust
pub enum DatadogError {
    ApiError(String),      // HTTP errors
    AuthError(String),     // 401/403
    RateLimitError,        // 429
    TimeoutError,          // 408
    NetworkError(reqwest::Error),
    JsonError(serde_json::Error),
    InvalidInput(String),
    DateParseError(String),
}
```

## Adding New Tools

**5-step process:**

1. **Schema** (`server/schema.rs`):
```rust
{
    "name": "datadog_resource_action",
    "description": "Action description with return data info",
    "inputSchema": {
        "type": "object",
        "properties": {
            "param": {"type": "string", "description": "..."},
            "optional": {"type": "integer", "default": 100}
        },
        "required": ["param"]
    }
}
```

2. **Handler** (`handlers/resource.rs`):
```rust
pub struct ResourceHandler;
impl TimeHandler for ResourceHandler {}
impl TagFilter for ResourceHandler {}      // If needed
impl ResponseFormatter for ResourceHandler {}

impl ResourceHandler {
    pub async fn action(client: Arc<DatadogClient>, params: &Value) -> Result<Value> {
        let handler = ResourceHandler;
        // Parse params, call client, format response
        Ok(handler.format_list(data, pagination, meta))
    }
}
```

3. **Route** (`server/router.rs`):
```rust
"datadog_resource_action" => handlers::resource::ResourceHandler::action(
    self.client.clone(), arguments
).await,
```

4. **Client method** (`datadog/client.rs`):
```rust
pub async fn resource_action(&self, param: &str) -> Result<Response> {
    self.request(Method::GET, "/api/v1/resource", Some(params), None::<()>).await
}
```

5. **Models** (`datadog/models.rs`):
```rust
#[derive(Debug, Serialize, Deserialize)]  // Clone only if cached
pub struct Response { pub data: Vec<Item> }
```

## Code Quality Standards

**Build Requirements:**
- ✅ `cargo build` with zero warnings
- ✅ Zero unsafe code blocks
- ✅ Zero dead code (`#[allow(dead_code)]` forbidden)
- ✅ All tests passing

**Design Principles:**
1. **YAGNI:** Build only what's needed now
2. **Self-documenting:** Clear naming over comments
3. **Trait-based:** Share logic via traits, not duplication
4. **Arc-optimized:** Use `Arc<T>` for shared data, avoid clones
5. **Type-safe:** Leverage Rust's type system fully

**Performance:**
- Connection pooling (reqwest automatic)
- Arc-based cache (not clone-based)
- Async/await throughout (tokio)
- Minimal allocations

## Testing

**Quick validation:**
```bash
cargo test                      # All tests (161)
cargo build --release           # Release build
cargo clippy                    # Lint check
```

**MCP protocol test:**
```bash
echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05"},"id":0}' | cargo run
```

**Debug mode:**
```bash
LOG_LEVEL=debug DD_API_KEY=xxx DD_APP_KEY=yyy cargo run
```

## Environment Variables

**Required:**
- `DD_API_KEY`: Datadog API key
- `DD_APP_KEY`: Datadog application key

**Optional:**
- `DD_SITE`: Region (default: datadoghq.com)
- `LOG_LEVEL`: Logging level (default: warn)
- `DD_TAG_FILTER`: Global tag filter (e.g., `"env:,service:"`)

## File Structure

```
src/
├── main.rs              # Entry point, env_logger setup
├── cache.rs             # Arc-based TTL cache
├── error.rs             # Comprehensive error types
├── utils.rs             # Time parsing (interim)
├── server/
│   ├── protocol.rs      # JSON-RPC 2.0 I/O
│   ├── schema.rs        # Tool definitions
│   └── router.rs        # Tool routing
├── datadog/
│   ├── client.rs        # HTTP client + API methods
│   ├── retry.rs         # Exponential backoff
│   └── models.rs        # Response types
└── handlers/
    ├── common.rs        # Shared traits (TimeHandler, TagFilter, etc.)
    ├── metrics.rs       # Metrics query
    ├── logs.rs          # Log search
    ├── logs_aggregate.rs   # Log aggregation
    ├── logs_timeseries.rs  # Log timeseries
    ├── monitors.rs      # Monitors
    ├── events.rs        # Events
    ├── hosts.rs         # Infrastructure
    ├── dashboards.rs    # Dashboards
    ├── spans.rs         # APM spans
    ├── services.rs      # Service catalog
    └── rum.rs           # RUM events
```

## Common Issues

**Arc dereference needed:**
```rust
// Cache returns Arc<Vec<T>>
let data = cache.get(...).await?;
let slice = handler.paginate(&*data, page, size);  // ← Deref with &*
```

**Tag filtering:**
```rust
// Always use TagFilter trait, never inline logic
let tags = handler.filter_tags(&tags_vec, filter_str);
```

**Time conversion:**
```rust
// Use TimeHandler helper
let iso = handler.timestamp_to_iso8601(unix_ts)?;
```

## Development Commands

```bash
# Build & test
cargo build && cargo test

# Watch mode
cargo watch -x build
cargo watch -x test

# Release
cargo build --release

# Format
cargo fmt

# Lint
cargo clippy
```

## Response Structure

**Optimized structure (70% smaller for Spans):**
```json
{
  "data": [ /* Clean data - no null/empty fields */ ],
  "pagination": {
    "total": 100,
    "page": 0,
    "page_size": 10,
    "has_next": true,
    "next_offset": 100  // Only for offset-based APIs
  }
}
```

**Key optimizations:**
- Stack traces truncated to 10 lines (use `full_stack_trace: true` for complete)
- Null/empty fields removed
- Single `pagination` object (meta removed)
- Consistent across all APIs

**Design:** Minimal, consistent, AI-agent optimized.

---

**Philosophy:** Clean code, zero waste, maximum performance. Every line serves a purpose.
