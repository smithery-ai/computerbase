mod apps;
mod batch;
mod capture;
mod clipboard;
mod display;
mod error;
mod input;
mod server;
mod types;

use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Logs go to stderr — stdout is the MCP protocol channel
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("computer-use-mcp starting");

    server::run_stdio().await
}
