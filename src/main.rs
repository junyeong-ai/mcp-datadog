mod cache;
mod datadog;
mod error;
mod handlers;
mod server;
mod utils;

use dotenvy::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv().ok();

    // Initialize logging with LOG_LEVEL or RUST_LOG environment variable
    // Default to "warn" if neither is set
    env_logger::Builder::from_env(env_logger::Env::default().filter_or(
        "RUST_LOG",
        env::var("LOG_LEVEL").unwrap_or_else(|_| "warn".to_string()),
    ))
    .init();

    // Get API credentials from environment
    let api_key = env::var("DD_API_KEY").unwrap_or_else(|_| "DEMO_API_KEY".to_string());

    let app_key = env::var("DD_APP_KEY").unwrap_or_else(|_| "DEMO_APP_KEY".to_string());

    let site = env::var("DD_SITE").ok();

    // Create and run the server
    let server = server::Server::new(api_key, app_key, site)?;
    server.run().await?;

    Ok(())
}
