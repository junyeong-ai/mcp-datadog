# MCP Datadog Server

<div align="center">

**Datadog Integration for AI Agents - Optimized for Token Efficiency and Performance**

English | [한국어](./README.md)

[![CI](https://github.com/junyeong-ai/mcp-datadog/workflows/CI/badge.svg)](https://github.com/junyeong-ai/mcp-datadog/actions)
[![Lint](https://github.com/junyeong-ai/mcp-datadog/workflows/Lint/badge.svg)](https://github.com/junyeong-ai/mcp-datadog/actions)
[![codecov](https://codecov.io/gh/junyeong-ai/mcp-datadog/branch/main/graph/badge.svg)](https://codecov.io/gh/junyeong-ai/mcp-datadog)

[![Rust](https://img.shields.io/badge/rust-1.90%2B%20(2024%20edition)-orange?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![MCP](https://img.shields.io/badge/MCP-2024--11--05-blue?style=flat-square)](https://modelcontextprotocol.io)
[![Tools](https://img.shields.io/badge/MCP%20tools-12-blue?style=flat-square)](#%EF%B8%8F-available-tools-12)
[![License](https://img.shields.io/badge/license-MIT-green?style=flat-square)](LICENSE)
[![Datadog](https://img.shields.io/badge/Datadog-API%20v1%2Fv2-632CA6?style=flat-square)](https://docs.datadoghq.com/api/)

</div>

---

## 📑 Table of Contents

- [🎯 What is This?](#-what-is-this)
- [🚀 Quick Start (3 Minutes)](#-quick-start-3-minutes)
- [💡 Why Use This?](#-why-use-this)
- [🎯 Real-World Examples](#-real-world-examples)
- [🛠️ Available Tools (12)](#️-available-tools-12)
- [⚙️ Environment Variables Guide](#️-environment-variables-guide)
- [🏗️ Tech Stack & Architecture](#️-tech-stack--architecture)
- [🧪 Development & Testing](#-development--testing)

---

## 🎯 What is This?

> **"Find error logs from the last hour"** → Claude automatically searches Datadog
> Use Datadog by conversing with AI - no complex query syntax needed

**MCP Datadog Server** is a Model Context Protocol server that enables AI agents to control Datadog using natural language. Optimized for AI agents with **hundreds of times token reduction**, **client-side caching**, and **natural language time** support.

### Real Usage Examples

**"Show production CPU usage trends"** → AI generates charts automatically
**"Analyze yesterday's incident"** → AI analyzes logs and suggests solutions

---

## 🚀 Quick Start (3 Minutes)

### 📋 Prerequisites

- **Rust** 1.90+ (2024 edition) - [Install](https://rustup.rs/)
- **Claude Desktop** - [Download](https://claude.ai/download)
- **Datadog Account** - API keys required ([Free trial](https://www.datadoghq.com/))

### 1️⃣ Build (1 minute)

```bash
git clone https://github.com/junyeong-ai/mcp-datadog.git
cd mcp-datadog
cargo build --release
```

### 2️⃣ Configure (1 minute)

Open your Claude Desktop config file:

- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
- **Linux**: `~/.config/Claude/claude_desktop_config.json`

Copy and paste this configuration:

```json
{
  "mcpServers": {
    "datadog": {
      "command": "/absolute/path/to/mcp-datadog/target/release/mcp-datadog",
      "env": {
        "DD_API_KEY": "your_api_key_here",
        "DD_APP_KEY": "your_app_key_here",
        "DD_SITE": "datadoghq.com",
        "DD_TAG_FILTER": "env:,service:",
        "LOG_LEVEL": "warn"
      }
    }
  }
}
```

> 💡 **Set DD_TAG_FILTER!** Exclude unnecessary tags to significantly reduce response size.

### 3️⃣ Run (30 seconds)

Restart Claude Desktop and you're done! 🎉

Now ask Claude:
```
"Aggregate error logs from production in the last hour by service"
"Show payment-api CPU usage trends"
"Analyze 500 errors between 3pm and 5pm yesterday"
```

---

## 💡 Why Use This?

### 📊 Comparison

| Method | Setup Time | AI-Friendly | Token Efficiency | Natural Language |
|--------|-----------|-------------|-----------------|------------------|
| **Direct API** | - | ❌ Low | ❌ No filtering | ❌ |
| **Python SDK** | 10min+ | ⚠️ Medium | ⚠️ Medium | ❌ |
| **MCP Datadog** | **3min** | ✅ **Optimized** | ✅ **Hundreds of times** | ✅ |

### 🎯 3 Core Optimizations

**1. Automatic Rollup for Hundreds of Times Token Reduction**
```bash
# 30-day metric query with max_points compression
{"query": "avg:system.cpu.user{*}", "from": "30 days ago", "max_points": 100}
# → Compresses 43,200 points to 60 (measured 720x reduction!)
```
- **Dozens to hundreds of times** reduction depending on time range (7d: 120x, 30d: 720x)
- 9-tier interval auto-calculation (60s ~ 86400s)
- Auto-detects aggregation method (avg/max/min/sum)
- Preserves existing rollup

**2. Smart Caching for Large Datasets**
- 100+ monitors paginated seamlessly
- First request fetches, subsequent requests use cache (TTL 5min, LRU)
- Solves no-server-pagination problem

**3. Tag Filtering for Response Size Reduction**
```bash
# Select only necessary tags
DD_TAG_FILTER="env:,service:"
# Dozens~hundreds of tags → only what you need
```

### ⚡ Additional Benefits

- **Single Binary**: 5.3MB, no runtime dependencies
- **Natural Language Time**: "1 hour ago", "yesterday" supported
- **Auto Retry**: Exponential backoff (max 3 retries)
- **Read-Only**: Safe data queries only

> 💡 **For detailed technical information, see [Tech Stack & Architecture](#️-tech-stack--architecture) section.**

---

## ✨ Key Features

### 📊 Metrics & Infrastructure (2 tools)
- **datadog_metrics_query**: Time series metrics + automatic rollup (up to 700x+ reduction)
- **datadog_hosts_list**: Host listing with tag filtering

### 📝 Logs & Analytics (3 tools)
- **datadog_logs_search**: Log search + tag filtering
- **datadog_logs_aggregate**: Log aggregation (count/sum/avg/min/max/pc99)
- **datadog_logs_timeseries**: Time series analysis (custom intervals)

### 🔍 Monitoring & Events (3 tools)
- **datadog_monitors_list**: Monitor listing (client-side caching)
- **datadog_monitors_get**: Individual monitor details
- **datadog_events_query**: Event stream (client-side caching)

### 📈 Dashboards (2 tools)
- **datadog_dashboards_list**: Dashboard listing (client-side caching)
- **datadog_dashboards_get**: Dashboard details

### 🔬 APM & Tracing (2 tools)
- **datadog_spans_search**: APM span search + **70% size reduction** (stack trace truncation) + cursor pagination
- **datadog_services_list**: Service catalog + environment filtering

> 📖 **For detailed parameters and usage, see [Available Tools](#️-available-tools-12) section.**

---

## ⚙️ Environment Variables Guide

| Variable | Required | Default | Description | 💡 Optimization Tip |
|----------|----------|---------|-------------|-------------------|
| `DD_API_KEY` | ✅ | - | Datadog API key | [Create in Datadog](https://app.datadoghq.com/organization-settings/api-keys) |
| `DD_APP_KEY` | ✅ | - | Datadog Application key | [Create in Datadog](https://app.datadoghq.com/organization-settings/application-keys) |
| `DD_SITE` | ❌ | `datadoghq.com` | Datadog site | Set for your region (datadoghq.eu, us3, us5, etc.) |
| `DD_TAG_FILTER` | ❌ | `*` (all tags) | Tag filter | **`"env:,service:"` for significant response size reduction!** |
| `LOG_LEVEL` | ❌ | `warn` | Log level | Use `debug` for troubleshooting |

### 🎯 DD_TAG_FILTER Strategies

Tag filtering can **significantly reduce response size**:

```bash
# Strategy 1: Production environment only
DD_TAG_FILTER="env:production"

# Strategy 2: Specific services only
DD_TAG_FILTER="env:,service:payment,service:auth"

# Strategy 3: Core tags only (recommended!)
DD_TAG_FILTER="env:,service:,version:"

# Strategy 4: Include all tags (default)
DD_TAG_FILTER="*"

# Strategy 5: Exclude all tags
DD_TAG_FILTER=""
```

**Usage Example**:
```json
{
  "name": "datadog_logs_search",
  "arguments": {
    "query": "status:error",
    "from": "1 hour ago",
    "tag_filter": "env:,service:"  // Save tokens here!
  }
}
```

---

## 🎯 Real-World Examples

### Example 1: Production Error Monitoring

**Ask Claude**:
```
"Aggregate error logs from production in the last hour by service"
```

**AI Automatically**:
1. Uses `datadog_logs_aggregate` tool
2. Sets `query="status:error env:production"`
3. Applies `group_by=["@service"]`
4. Presents results in a table

### Example 2: Performance Analysis

**Ask Claude**:
```
"Show payment-api CPU usage trends for the last 24 hours"
```

**AI Automatically**:
1. Uses `datadog_metrics_query` tool
2. Creates `query="avg:system.cpu.user{service:payment-api}"`
3. Sets `from="24 hours ago"`
4. Visualizes with charts

### Example 3: Incident Investigation

**Ask Claude**:
```
"Find status:500 errors between 3pm and 5pm yesterday
 and tell me which endpoint had the most"
```

**AI Automatically**:
1. Searches logs with `datadog_logs_search`
2. Aggregates with `datadog_logs_aggregate`
3. Identifies most frequent endpoint
4. Provides root cause analysis and solutions

### Example 4: Resource Optimization

**Ask Claude**:
```
"Show me hosts with memory usage above 80%"
```

**AI Automatically**:
1. Uses `datadog_hosts_list` tool
2. Analyzes metric data
3. Filters hosts exceeding threshold
4. Provides optimization recommendations

---

## 🛠️ Available Tools (12)

<details>
<summary><b>📊 Metrics & Infrastructure (2)</b></summary>

### datadog_metrics_query
Query time series metrics (CPU, memory, network, etc.)

**🚀 Automatic Rollup**: Drastically reduce tokens with `max_points`! Auto-calculates optimal interval from time range and max_points, adding `.rollup(agg, interval)`

**Parameters**:
- `query` (required): Metrics query (e.g., `"avg:system.cpu.user{*}"`)
- `from` (optional): Start time (default: `"1 hour ago"`)
- `to` (optional): End time (default: `"now"`)
- `max_points` (optional): Maximum data points (e.g., 100) - Enables automatic rollup

**Example**:
```json
{
  "query": "avg:system.cpu.user{*}",
  "from": "7 days ago",
  "to": "now",
  "max_points": 100
}
// Auto-applies 2-hour rollup → 120x token reduction
```

### datadog_hosts_list
List infrastructure hosts with filtering

**Parameters**:
- `filter` (optional): Host filter query
- `from` (optional): Start time (default: `"1 hour ago"`)
- `count` (optional): Number of hosts (default: 100, max: 1000)
- `tag_filter` (optional): Tag filtering

</details>

<details>
<summary><b>📝 Logs & Analytics (3)</b></summary>

### datadog_logs_search
Powerful log search and filtering

**Parameters**:
- `query` (required): Log search query
- `from` (optional): Start time (default: `"1 hour ago"`)
- `to` (optional): End time (default: `"now"`)
- `limit` (optional): Maximum logs (default: 10)
- `tag_filter` (optional): Tag filtering (`"*"`, `""`, `"env:,service:"`)

### datadog_logs_aggregate
Aggregate log events and compute metrics

**Parameters**:
- `query` (optional): Log search query (default: `"*"`)
- `from` (optional): Start time
- `to` (optional): End time
- `compute` (optional): Aggregation operations array (count, sum, avg, min, max, pc99)
- `group_by` (optional): Grouping facets array

### datadog_logs_timeseries
Generate time series data from log aggregations

**Parameters**:
- `query` (optional): Log search query
- `from` (optional): Start time
- `to` (optional): End time
- `interval` (optional): Time interval (default: `"1h"`)
- `aggregation` (optional): Aggregation type (default: `"count"`)

</details>

<details>
<summary><b>🔍 Monitoring & Events (3)</b></summary>

### datadog_monitors_list
List all monitors (client-side caching)

**🎯 Client Caching Core**: Datadog API returns all monitors at once, risking token limits. This tool handles caching + pagination on the client side!

**Parameters**:
- `tags` (optional): Tag filter (comma-separated)
- `monitor_tags` (optional): Monitor tag filter
- `page` (optional): Page number (0-indexed)
  - **Page 0**: Always fetch fresh & cache
  - **Page 1+**: Slice from cache (5min TTL)
- `page_size` (optional): Monitors per page (default: 50)

**Benefits**:
- Browse 100+ monitors without token limits
- Fast response after first request (cache hit)
- Memory-efficient with LRU eviction

### datadog_monitors_get
Get detailed monitor information

**Parameters**:
- `monitor_id` (required): Monitor ID

### datadog_events_query
Query Datadog event stream

**🎯 Client Caching Core**: Efficiently handles massive event streams returned all at once via client-side caching!

**Parameters**:
- `from` (optional): Start time (default: `"1 hour ago"`)
- `to` (optional): End time (default: `"now"`)
- `priority` (optional): Priority filter (`"normal"`, `"low"`)
- `sources` (optional): Source filter
- `tags` (optional): Tag filter
- `page` (optional): Page number (default: 0)
  - **Page 0**: Fresh data & cache
  - **Page 1+**: Use cache (5min TTL)

</details>

<details>
<summary><b>📈 Dashboards (2)</b></summary>

### datadog_dashboards_list
List all dashboards

**🎯 Client Caching**: Efficiently handles no-pagination API on client side (5min TTL)

### datadog_dashboards_get
Get detailed dashboard information

**Parameters**:
- `dashboard_id` (required): Dashboard ID

</details>

<details>
<summary><b>🔬 APM & Tracing (3)</b></summary>

### datadog_spans_search
Search APM spans (advanced filtering)

**🎯 70% Response Size Reduction**: Stack traces truncated to 10 lines by default, empty fields removed!

**Parameters**:
- `query` (optional): Search query (default: `"*"`)
- `from` (required): Start time
- `to` (required): End time
- `limit` (optional): Maximum spans (default: 10)
- `cursor` (optional): Pagination cursor
- `tag_filter` (optional): Tag filtering
- `full_stack_trace` (optional): If true, include complete stack traces (default: false)

### datadog_services_list
List services from catalog

**Parameters**:
- `env` (optional): Environment filter
- `page` (optional): Page number (default: 0)
- `page_size` (optional): Items per page (default: 10)

</details>

---

## 🏗️ Tech Stack & Architecture

### Core Technologies

- **Language**: Rust 2024 Edition (1.90+)
- **Protocol**: Model Context Protocol (MCP 2024-11-05)
- **Communication**: JSON-RPC 2.0 over stdio
- **HTTP Client**: reqwest (HTTP/2, rustls-tls)
- **Async Runtime**: tokio (full features)
- **Time Parsing**: interim (natural language support)

### Performance Characteristics

| Metric | Value |
|--------|-------|
| **Binary Size** | ~5.3MB (LTO optimized) |
| **Cache TTL** | 5 minutes (configurable) |
| **Request Timeout** | 30 seconds |
| **Max Retries** | 3 times (exponential backoff) |

### Architecture

```
┌─────────────────────────────────────────────────┐
│         AI Agent (Claude, ChatGPT)              │
│           Natural language queries              │
└────────────────────┬────────────────────────────┘
                     │ JSON-RPC 2.0 (stdio)
┌────────────────────▼────────────────────────────┐
│            MCP Datadog Server (Rust)            │
│  ┌─────────────────────────────────────────┐   │
│  │  MCP Protocol Handler                   │   │
│  │  - JSON-RPC 2.0                         │   │
│  │  - Tool Schema (12 tools)               │   │
│  │  - Router                               │   │
│  └─────────────┬───────────────────────────┘   │
│                │                                 │
│  ┌─────────────▼───────────────────────────┐   │
│  │  Smart Cache (TTL: 5min)                │   │
│  │  - Page 0: Always fresh                 │   │
│  │  - Later pages: Use cache               │   │
│  └─────────────┬───────────────────────────┘   │
│                │                                 │
│  ┌─────────────▼───────────────────────────┐   │
│  │  Datadog Client (HTTP/2)                │   │
│  │  - Retry Logic (exponential backoff)    │   │
│  │  - Rate Limit Handling                  │   │
│  │  - Connection Pooling                   │   │
│  └─────────────┬───────────────────────────┘   │
└────────────────┼───────────────────────────────┘
                 │ HTTPS
┌────────────────▼───────────────────────────────┐
│              Datadog API (v1/v2)               │
│  Metrics, Logs, Monitors, Events, APM, etc.   │
└────────────────────────────────────────────────┘
```

---

## 📚 Time Format Support

Supports natural language, absolute time, and relative time:

### Relative Time (Natural Language)
```
"10 minutes ago"
"2 hours ago"
"3 days ago"
"1 week ago"
```

### Named Times
```
"now"
"today"
"yesterday"
"last week"
"last month"
```

### Absolute Time
```
ISO 8601: "2024-01-15T10:30:00Z"
Unix timestamp: 1704067200
```

---

## 🧪 Development & Testing

### Development Setup

```bash
# Clone repository
git clone https://github.com/junyeong-ai/mcp-datadog.git
cd mcp-datadog

# Set up environment variables
cp .env.example .env
# Add API keys to .env

# Run in debug mode
LOG_LEVEL=debug cargo run
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test --lib cache::tests
cargo test --lib handlers::metrics::tests

# Verbose output
cargo test -- --nocapture

# Test coverage
cargo install cargo-llvm-cov
cargo llvm-cov --all-features --lcov --output-path lcov.info
```

### MCP Protocol Testing

```bash
# Initialize
echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05"},"id":0}' | cargo run

# List tools
echo '{"jsonrpc":"2.0","method":"tools/list","id":1}' | cargo run

# Execute tool (metrics query)
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"datadog_metrics_query","arguments":{"query":"avg:system.cpu.user{*}","from":"1 hour ago"}},"id":2}' | cargo run
```

### Production Build

```bash
# Optimized build
cargo build --release

# Strip symbols (optional)
strip target/release/mcp-datadog

# Result: ~5.3MB binary
```

---

## 🔒 Security

- **Read-Only**: All operations are read-only (no data modification)
- **Credential Safety**: API keys are never logged
- **Input Validation**: All parameters are validated
- **Error Handling**: No internal information exposure

---

## 🤝 Contributing

Contributions welcome!

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Guidelines
- Format code with `cargo fmt`
- Check lints with `cargo clippy -- -D warnings`
- Pass all tests with `cargo test`
- Unit tests required for new features
- Zero warnings policy (enforced in CI)

---

## 📝 License

MIT License - see [LICENSE](LICENSE) file for details.

---

## 💬 Support

- **Issues**: Bug reports and feature requests on [GitHub Issues](https://github.com/junyeong-ai/mcp-datadog/issues)
- **Discussions**: Questions and discussions on [GitHub Discussions](https://github.com/junyeong-ai/mcp-datadog/discussions)
- **Documentation**: This README and [CLAUDE.md](./CLAUDE.md)

---

<div align="center">

**Made with ❤️ in Rust for the MCP ecosystem**

[⭐ Star this repo](https://github.com/junyeong-ai/mcp-datadog) | [🐛 Report Bug](https://github.com/junyeong-ai/mcp-datadog/issues) | [✨ Request Feature](https://github.com/junyeong-ai/mcp-datadog/issues)

</div>
