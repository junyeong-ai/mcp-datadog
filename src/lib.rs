// Library interface for MCP Datadog Server
// This exposes modules for testing and potential library usage

pub mod cache;
pub mod datadog;
pub mod error;
pub mod handlers;
pub mod server;
pub mod utils;

// Re-export commonly used types
pub use error::{DatadogError, Result};
pub use datadog::DatadogClient;
