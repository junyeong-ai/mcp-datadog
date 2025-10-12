use super::protocol::{JsonRpcRequest, JsonRpcResponse, Server};
use crate::error::Result;
use serde_json::json;

impl Server {
    pub async fn handle_tools_list(
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

        // Get tag filter default from environment variable
        let tag_filter_default = self.client.get_tag_filter().unwrap_or("*");
        let tag_filter_desc = format!(
            "Comma-separated tag prefixes to include (e.g., 'env:,service:,version:'). Use '*' for all tags (default), '' (empty) to exclude all tags. Current default: '{}'",
            tag_filter_default
        );

        let tools_result = json!({
            "tools": [
                {
                    "name": "datadog_metrics_query",
                    "description": "Query time series metrics from Datadog. Returns metric data points with timestamps and values. Supports natural language time expressions ('1 hour ago'), ISO8601, and Unix timestamps.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "Metrics query (e.g., 'avg:system.cpu.user{*}')"
                            },
                            "from": {
                                "type": "string",
                                "description": "Start time (supports natural language like '1 hour ago', ISO8601 timestamps, or Unix timestamps)",
                                "default": "1 hour ago"
                            },
                            "to": {
                                "type": "string",
                                "description": "End time (supports natural language like 'now', ISO8601 timestamps, or Unix timestamps)",
                                "default": "now"
                            },
                            "max_points": {
                                "type": "integer",
                                "description": "Maximum number of data points to return (downsample if exceeded). Useful for large time ranges to reduce response size. If not specified, returns all points from API."
                            }
                        },
                        "required": ["query"]
                    }
                },
                {
                    "name": "datadog_logs_search",
                    "description": "Search log events in Datadog. Returns log entries with timestamps, messages, and metadata. Supports Datadog query syntax and natural language time expressions.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "Log search query"
                            },
                            "from": {
                                "type": "string",
                                "description": "Start time (supports natural language like '1 hour ago', ISO8601, or Unix timestamps)",
                                "default": "1 hour ago"
                            },
                            "to": {
                                "type": "string",
                                "description": "End time (supports natural language like 'now', ISO8601, or Unix timestamps)",
                                "default": "now"
                            },
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of logs to return",
                                "default": 10
                            },
                            "tag_filter": {
                                "type": "string",
                                "description": &tag_filter_desc
                            }
                        },
                        "required": ["query"]
                    }
                },
                {
                    "name": "datadog_monitors_list",
                    "description": "List all monitors from Datadog. Returns monitor names, types, queries, and states. Supports filtering by tags. Page 0 always fetches fresh data, subsequent pages use cache.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "tags": {
                                "type": "string",
                                "description": "Filter by tags (comma-separated)"
                            },
                            "monitor_tags": {
                                "type": "string",
                                "description": "Filter by monitor tags"
                            },
                            "page": {
                                "type": "integer",
                                "description": "Page number (0-based). Page 0 always fetches fresh data from Datadog API.",
                                "default": 0
                            },
                            "page_size": {
                                "type": "integer",
                                "description": "Number of monitors per page",
                                "default": 50
                            }
                        }
                    }
                },
                {
                    "name": "datadog_monitors_get",
                    "description": "Retrieve detailed information about a specific monitor by ID. Returns full monitor configuration, thresholds, notification settings, and current state.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "monitor_id": {
                                "type": "integer",
                                "description": "Monitor ID"
                            }
                        },
                        "required": ["monitor_id"]
                    }
                },
                {
                    "name": "datadog_events_query",
                    "description": "Query event stream from Datadog. Returns events with titles, text, timestamps, and alert types. Supports filtering by priority, sources, and tags. Page 0 fetches fresh data.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "from": {
                                "type": "string",
                                "description": "Start time (supports natural language like '1 hour ago', ISO8601, or Unix timestamps)",
                                "default": "1 hour ago"
                            },
                            "to": {
                                "type": "string",
                                "description": "End time (supports natural language like 'now', ISO8601, or Unix timestamps)",
                                "default": "now"
                            },
                            "priority": {
                                "type": "string",
                                "description": "Priority filter (normal, low)"
                            },
                            "sources": {
                                "type": "string",
                                "description": "Sources filter"
                            },
                            "tags": {
                                "type": "string",
                                "description": "Tags filter"
                            },
                            "page": {
                                "type": "integer",
                                "description": "Page number (0-based). Page 0 always fetches fresh data from Datadog API.",
                                "default": 0
                            },
                            "page_size": {
                                "type": "integer",
                                "description": "Number of events per page",
                                "default": 50
                            }
                        }
                    }
                },
                {
                    "name": "datadog_hosts_list",
                    "description": "List infrastructure hosts from Datadog. Returns host names, status, applications, sources, and tags. Supports filtering and sorting by various fields.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "filter": {
                                "type": "string",
                                "description": "Host filter query"
                            },
                            "from": {
                                "type": "string",
                                "description": "From time (supports natural language like '1 hour ago', ISO8601, or Unix timestamps)",
                                "default": "1 hour ago"
                            },
                            "sort_field": {
                                "type": "string",
                                "description": "Sort field"
                            },
                            "sort_dir": {
                                "type": "string",
                                "description": "Sort direction (asc, desc)"
                            },
                            "start": {
                                "type": "integer",
                                "description": "Starting index for pagination",
                                "default": 0
                            },
                            "count": {
                                "type": "integer",
                                "description": "Number of hosts to return (max 1000)",
                                "default": 100
                            },
                            "tag_filter": {
                                "type": "string",
                                "description": &tag_filter_desc
                            }
                        }
                    }
                },
                {
                    "name": "datadog_dashboards_list",
                    "description": "List all dashboards from Datadog. Returns dashboard IDs, titles, and descriptions. Page 0 fetches fresh data, subsequent pages use cache.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "page": {
                                "type": "integer",
                                "description": "Page number (0-based). Page 0 fetches fresh data from Datadog API.",
                                "default": 0
                            },
                            "page_size": {
                                "type": "integer",
                                "description": "Number of dashboards per page",
                                "default": 50
                            }
                        }
                    }
                },
                {
                    "name": "datadog_dashboards_get",
                    "description": "Retrieve full dashboard configuration by ID. Returns title, description, layout type, widgets, template variables, and author information.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "dashboard_id": {
                                "type": "string",
                                "description": "Dashboard ID"
                            }
                        },
                        "required": ["dashboard_id"]
                    }
                },
                {
                    "name": "datadog_spans_search",
                    "description": "Search APM trace spans from Datadog. Returns span details with timing, service information, and trace IDs. Supports cursor-based pagination and sorting.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "Spans search query",
                                "default": "*"
                            },
                            "from": {
                                "type": "string",
                                "description": "Start time (e.g., '1 hour ago', timestamp)"
                            },
                            "to": {
                                "type": "string",
                                "description": "End time (e.g., 'now', timestamp)"
                            },
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of spans to return",
                                "default": 10
                            },
                            "cursor": {
                                "type": "string",
                                "description": "Pagination cursor"
                            },
                            "sort": {
                                "type": "string",
                                "description": "Sort order (e.g., 'timestamp')"
                            },
                            "page": {
                                "type": "integer",
                                "description": "Page number (0-based, for client-side pagination)",
                                "default": 0
                            },
                            "page_size": {
                                "type": "integer",
                                "description": "Number of spans per page",
                                "default": 10
                            },
                            "tag_filter": {
                                "type": "string",
                                "description": &tag_filter_desc
                            }
                        },
                        "required": ["from", "to"]
                    }
                },
                {
                    "name": "datadog_services_list",
                    "description": "List services from APM service catalog. Returns service names, teams, repositories, integrations, and metadata. Supports environment filtering.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "env": {
                                "type": "string",
                                "description": "Filter by environment (e.g., 'production', 'staging')"
                            },
                            "page": {
                                "type": "integer",
                                "description": "Page number (0-based, for client-side pagination)",
                                "default": 0
                            },
                            "page_size": {
                                "type": "integer",
                                "description": "Number of services per page",
                                "default": 50
                            }
                        }
                    }
                },
                {
                    "name": "datadog_logs_aggregate",
                    "description": "Aggregate log events into buckets and compute metrics. Returns aggregated data with count, sum, avg, min, max, or percentiles. Supports grouping by log attributes.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "Log search query",
                                "default": "*"
                            },
                            "from": {
                                "type": "string",
                                "description": "Start time (e.g., '1 hour ago', timestamp)"
                            },
                            "to": {
                                "type": "string",
                                "description": "End time (e.g., 'now', timestamp)"
                            },
                            "compute": {
                                "type": "array",
                                "description": "Array of compute aggregations (count, sum, avg, min, max, pc99)",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "aggregation": {"type": "string"},
                                        "type": {"type": "string"},
                                        "interval": {"type": "string"},
                                        "metric": {"type": "string"}
                                    }
                                }
                            },
                            "group_by": {
                                "type": "array",
                                "description": "Array of fields to group by",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "facet": {"type": "string"},
                                        "limit": {"type": "integer"},
                                        "sort": {"type": "object"}
                                    }
                                }
                            },
                            "timezone": {
                                "type": "string",
                                "description": "Timezone for time-based operations (e.g., 'UTC', 'America/New_York')"
                            }
                        },
                        "required": ["from", "to"]
                    }
                },
                {
                    "name": "datadog_logs_timeseries",
                    "description": "Generate time series data from log events. Returns bucketed metrics over time with configurable intervals (1m, 5m, 1h). Supports count, sum, avg, and percentile aggregations.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "Log search query",
                                "default": "*"
                            },
                            "from": {
                                "type": "string",
                                "description": "Start time (e.g., '1 hour ago', timestamp)"
                            },
                            "to": {
                                "type": "string",
                                "description": "End time (e.g., 'now', timestamp)"
                            },
                            "interval": {
                                "type": "string",
                                "description": "Time interval for timeseries (e.g., '1m', '5m', '1h')",
                                "default": "1h"
                            },
                            "aggregation": {
                                "type": "string",
                                "description": "Aggregation type (count, sum, avg, min, max, pc99)",
                                "default": "count"
                            },
                            "metric": {
                                "type": "string",
                                "description": "Field to aggregate on (for non-count aggregations)"
                            },
                            "group_by": {
                                "type": "array",
                                "description": "Array of fields to group by",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "facet": {"type": "string"},
                                        "limit": {"type": "integer"}
                                    }
                                }
                            },
                            "timezone": {
                                "type": "string",
                                "description": "Timezone for time-based operations (e.g., 'UTC', 'America/New_York')"
                            }
                        },
                        "required": ["from", "to"]
                    }
                }
            ]
        });

        let response = Self::create_success_response(tools_result, request.id.clone());
        Ok(Some(response))
    }
}
