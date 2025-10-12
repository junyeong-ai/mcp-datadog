# MCP Datadog Server

[![Rust](https://img.shields.io/badge/rust-1.90%2B-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![MCP](https://img.shields.io/badge/MCP-2024--11--05-green.svg)](https://modelcontextprotocol.io)
[![Datadog](https://img.shields.io/badge/Datadog-API%20v1%2Fv2-632CA6.svg)](https://docs.datadoghq.com/api/)
[![GitHub](https://img.shields.io/github/stars/junyeong-ai/mcp-datadog?style=social)](https://github.com/junyeong-ai/mcp-datadog)

A high-performance Model Context Protocol (MCP) server that provides AI agents with comprehensive access to Datadog's observability platform. Query metrics, search logs, analyze aggregations, monitor infrastructure, and manage dashboards through natural language with intelligent caching and optimized pagination.

## Features

- **Comprehensive Datadog API Coverage**: Access metrics, logs, analytics, monitors, events, hosts, dashboards, spans, and services for core monitoring use cases
- **Natural Language Time Expressions**: Use intuitive expressions like "1 hour ago", "yesterday", or "last week"  
- **Intelligent Caching & Pagination**: Smart caching strategy with page 0 always fresh, 5-minute TTL for optimal performance
- **Advanced Log Analytics**: Aggregations and timeseries analysis with proper v2 API compliance
- **APM Integration**: Full spans search and services catalog access
- **AI Agent Optimized**: Tool descriptions designed for optimal AI agent comprehension
- **Optimized Response Payloads**: Clean responses with essential data only, no duplicate fields
- **Resilient Network Layer**: Built-in exponential backoff and automatic retry logic
- **Direct stdio Communication**: Efficient JSON-RPC 2.0 protocol over stdin/stdout

## Prerequisites

- Rust 1.90 or higher (Rust 2024 edition)
- Datadog API Key and Application Key
- Claude Desktop (for MCP integration)

## Installation

### Build from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/mcp-datadog.git
cd mcp-datadog

# Build the project
cargo build --release

# The binary will be available at ./target/release/mcp-datadog
```

## Configuration

### Environment Variables

Create a `.env` file in the project root:

```env
DD_API_KEY=your_api_key_here
DD_APP_KEY=your_app_key_here
DD_SITE=datadoghq.com  # Optional: datadoghq.eu, us3.datadoghq.com, us5.datadoghq.com
DD_TAG_FILTER=env:,service:,version:,host:  # Optional: Default tag filter for logs (comma-separated prefixes)
LOG_LEVEL=warn  # Optional: trace, debug, info, warn, error (default: warn)
```

### Claude Desktop Integration

Add the server to your Claude Desktop configuration:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows**: `%APPDATA%\\Claude\\claude_desktop_config.json`  
**Linux**: `~/.config/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "datadog": {
      "command": "/absolute/path/to/mcp-datadog",
      "env": {
        "DD_API_KEY": "your_api_key_here",
        "DD_APP_KEY": "your_app_key_here",
        "DD_SITE": "datadoghq.com",
        "DD_TAG_FILTER": "env:,service:,version:,host:",
        "LOG_LEVEL": "warn"
      }
    }
  }
}
```

## Available Tools

### Metrics & Infrastructure

#### datadog_metrics_query
Query time series metrics from Datadog.

**Parameters:**
- `query` (required): Metrics query string (e.g., "avg:system.cpu.user{*}")
- `from` (optional): Start time - defaults to "1 hour ago"
- `to` (optional): End time - defaults to "now"
- `max_points` (optional): Maximum data points to return (API applies rollup aggregation)

**Example:**
```json
{
  "query": "avg:system.cpu.user{host:production-*}",
  "from": "24 hours ago",
  "to": "now",
  "max_points": 50
}
```

#### datadog_hosts_list
List infrastructure hosts with filtering and pagination.

**Parameters:**
- `filter` (optional): Host filter query
- `from` (optional): From time - defaults to "1 hour ago"
- `sort_field` (optional): Field to sort by
- `sort_dir` (optional): Sort direction ("asc" or "desc")
- `start` (optional): Starting index - defaults to 0
- `count` (optional): Number of hosts (max 1000) - defaults to 100
- `tag_filter` (optional): Tag filtering (same as logs)

### Logs & Analytics

#### datadog_logs_search
Search through Datadog logs with powerful filtering.

**Parameters:**
- `query` (required): Log search query
- `from` (optional): Start time - defaults to "1 hour ago"
- `to` (optional): End time - defaults to "now"
- `limit` (optional): Maximum number of logs - defaults to 10
- `tag_filter` (optional): Tag filtering control with explicit keywords:
  - `"*"` - Return all tags (no filtering)
  - `""` - Exclude all tags (empty response)
  - `"env:,service:,..."` - Include only tags with specified prefixes
  - Default determined by `DD_TAG_FILTER` env var, or `"*"` if not set

**Examples:**
```json
{
  "query": "status:error service:payment-api",
  "from": "30 minutes ago",
  "limit": 20
}
```

**Tag Filtering Examples:**
```json
// Return all tags explicitly
{
  "query": "*",
  "tag_filter": "*"
}

// Exclude all tags
{
  "query": "*",
  "tag_filter": ""
}

// Filter to specific tag prefixes
{
  "query": "*",
  "tag_filter": "env:,service:,version:"
}

// Use environment variable default (omit tag_filter)
{
  "query": "*"
}
```

#### datadog_logs_aggregate
Aggregate log events into buckets and compute metrics.

**Parameters:**
- `query` (optional): Log search query - defaults to "*"
- `from` (optional): Start time - defaults to "1 hour ago"
- `to` (optional): End time - defaults to "now"
- `compute` (optional): Array of aggregation computations
- `group_by` (optional): Array of grouping facets
- `timezone` (optional): Timezone for results

**Example:**
```json
{
  "query": "service:web-app status:error",
  "from": "2 hours ago",
  "compute": [
    {
      "type": "total",
      "aggregation": "count"
    }
  ],
  "group_by": [
    {
      "facet": "@http.status_code",
      "type": "facet"
    }
  ]
}
```

#### datadog_logs_timeseries  
Generate timeseries data from log aggregations.

**Parameters:**
- `query` (optional): Log search query - defaults to "*"
- `from` (optional): Start time - defaults to "1 hour ago"
- `to` (optional): End time - defaults to "now"
- `interval` (optional): Time interval - defaults to "1h"
- `aggregation` (optional): Aggregation type - defaults to "count"
- `metric` (optional): Metric field for aggregation
- `group_by` (optional): Array of grouping facets
- `timezone` (optional): Timezone for results

### Monitoring & Events

#### datadog_monitors_list
List all configured monitors with smart caching and pagination.

**Parameters:**
- `tags` (optional): Filter by tags (comma-separated)
- `monitor_tags` (optional): Filter by monitor-specific tags
- `page` (optional): Page number (0-based, page 0 always fresh) - defaults to 0
- `page_size` (optional): Number of monitors per page - defaults to 50

#### datadog_monitors_get
Retrieve detailed information about a specific monitor.

**Parameters:**
- `monitor_id` (required): The monitor ID

#### datadog_events_query
Query the Datadog event stream with smart caching and pagination.

**Parameters:**
- `from` (optional): Start time - defaults to "1 hour ago"
- `to` (optional): End time - defaults to "now"
- `priority` (optional): Filter by priority ("normal" or "low")
- `sources` (optional): Filter by sources
- `tags` (optional): Filter by tags
- `page` (optional): Page number (0-based, page 0 always fresh) - defaults to 0
- `page_size` (optional): Number of events per page - defaults to 50

### Dashboards

#### datadog_dashboards_list
List all available dashboards.

#### datadog_dashboards_get
Get detailed information about a specific dashboard.

**Parameters:**
- `dashboard_id` (required): The dashboard ID

### APM & Tracing

#### datadog_spans_search
Search APM spans with advanced filtering.

**Parameters:**
- `query` (optional): Search query - defaults to "*"
- `from` (required): Start time
- `to` (required): End time
- `limit` (optional): Maximum spans to return - defaults to 10
- `cursor` (optional): Pagination cursor
- `sort` (optional): Sort order
- `tag_filter` (optional): Tag filtering (same as logs)

#### datadog_services_list
List services from the Datadog service catalog.

**Parameters:**
- `env` (optional): Filter by environment
- `page` (optional): Page number - defaults to 0
- `page_size` (optional): Items per page - defaults to 10

## Caching & Performance Strategy

The server implements intelligent caching for optimal performance:

- **Page 0**: Always fetches fresh data from API
- **Subsequent Pages**: Uses cached data if available (5-minute TTL)
- **Smart Invalidation**: Cache automatically refreshes when page 0 is requested
- **API Types**: Only monitors and events use caching (no server-side pagination)

This ensures real-time data visibility while maintaining fast pagination performance.

## Time Format Support

Flexible time parsing powered by the `interim` library supports various formats:

### Relative Time
- `"10 minutes ago"`
- `"2 hours ago"`
- `"3 days ago"`
- `"1 week ago"`

### Named Times
- `"now"`
- `"today"`
- `"yesterday"`
- `"last week"`
- `"last month"`

### Absolute Formats
- ISO 8601: `"2024-01-15T10:30:00Z"`
- Unix timestamp: `1704067200`

## Project Architecture

```
mcp-datadog/
├── src/
│   ├── main.rs              # Application entry point
│   ├── cache.rs             # TTL-based caching system
│   ├── error.rs             # Error types and handling
│   ├── utils.rs             # Time parsing (interim library)
│   ├── server/
│   │   ├── mod.rs           # Server module exports
│   │   ├── protocol.rs      # MCP protocol & JSON-RPC handling
│   │   ├── schema.rs        # Tool schema definitions
│   │   └── router.rs        # Tool routing to handlers
│   ├── datadog/
│   │   ├── client.rs        # Datadog API client
│   │   ├── retry.rs         # Retry logic with exponential backoff
│   │   ├── models.rs        # API response models and types
│   │   └── mod.rs           # Module definitions
│   └── handlers/            # Tool implementations
│       ├── common.rs        # Shared traits and utilities
│       ├── metrics.rs       # Metrics queries
│       ├── logs.rs          # Log search
│       ├── logs_aggregate.rs   # Log aggregations
│       ├── logs_timeseries.rs  # Log timeseries
│       ├── monitors.rs      # Monitor management
│       ├── events.rs        # Event stream queries
│       ├── hosts.rs         # Infrastructure hosts
│       ├── dashboards.rs    # Dashboard access
│       ├── spans.rs         # APM spans search
│       ├── services.rs      # Service catalog
│       └── mod.rs           # Handler module exports
├── Cargo.toml               # Dependencies (Rust 2024, reqwest, tokio, dotenvy)
├── .env.example             # Environment template
└── README.md                # This documentation
```

## Development

### Running in Development

```bash
# Set up environment variables
cp .env.example .env
# Edit .env with your credentials

# Run with different log levels
LOG_LEVEL=debug cargo run
LOG_LEVEL=info cargo run
```

### Testing

```bash
# Build and run tests
cargo test

# Test MCP protocol compliance
echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05"},"id":0}' | cargo run

# Test tool listing
echo '{"jsonrpc":"2.0","method":"tools/list","id":1}' | cargo run
```

### Building for Production

```bash
# Create optimized build
cargo build --release

# Strip symbols for smaller binary (optional)
strip target/release/mcp-datadog
```

## API Rate Limits & Resilience

The server implements resilient API communication:

- **Automatic Retry**: Up to 3 retries with exponential backoff (2^n seconds)
- **Timeout Handling**: 30-second timeout per request
- **Rate Limit Detection**: Automatically detects 429 responses and retries
- **Network Resilience**: Connection pooling and error recovery

## Error Handling

Comprehensive error handling provides clear feedback:

- **Authentication Errors**: Clear API/APP key validation messages
- **Rate Limiting**: Automatic retry with exponential backoff
- **Invalid Queries**: Detailed syntax error descriptions
- **Network Issues**: Connection and timeout error details
- **Time Format Errors**: Suggestions for correct formats

## Logging & Debugging

Control log output with the `LOG_LEVEL` environment variable:

- `trace`: Detailed trace information for debugging
- `debug`: Debug information and API request details
- `info`: General informational messages
- `warn`: Warnings and errors only (default)
- `error`: Errors only

### Debug Mode
```bash
LOG_LEVEL=debug ./mcp-datadog
```

## Troubleshooting

### Server Won't Start
- Verify `DD_API_KEY` and `DD_APP_KEY` environment variables
- Check binary has execute permissions: `chmod +x mcp-datadog`
- Validate Datadog credentials with a simple API call

### No Data Returned
- Verify your `DD_SITE` matches your Datadog instance
- Check time range includes actual data
- Validate query syntax for the specific API
- Ensure proper permissions for API keys

### Performance Issues
- Monitor cache hit rates in debug logs
- Check network connectivity to Datadog
- Verify pagination parameters are reasonable
- Consider using more specific time ranges

### API Errors
- Enable debug logging to see request/response details
- Check Datadog API status page
- Verify query syntax in Datadog UI first
- Ensure API keys have required permissions

## Security Considerations

- **Credential Security**: API keys are never logged or exposed in responses
- **Read-Only Operations**: All operations are strictly read-only 
- **No Data Modification**: Server has no capability to modify Datadog data
- **Secure Storage**: Store credentials in environment variables or secure config files

## Performance Characteristics

- **Binary Size**: ~5.3MB (release build with LTO, optimized dependencies)
- **CPU Usage**: Minimal CPU usage, I/O bound operations
- **Network**: Efficient connection reuse with HTTP/2 support
- **Caching**: 5-minute TTL for paginated endpoints (monitors, events)
- **Concurrency**: Full async/await support for concurrent requests

## Contributing

We welcome contributions! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Ensure all tests pass: `cargo test`
5. Submit a pull request

### Development Guidelines
- Follow Rust formatting: `cargo fmt`
- Check for issues: `cargo clippy`
- Maintain API compatibility
- Add tests for new features
- Update documentation as needed

## License

MIT License - see LICENSE file for details.

## Support

- **Issues**: Report bugs and feature requests on GitHub
- **Documentation**: Comprehensive docs in this README
- **Community**: Discussions and questions welcome in GitHub issues

---

Built with ❤️ in Rust for the MCP ecosystem.