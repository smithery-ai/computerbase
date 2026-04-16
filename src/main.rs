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
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    tracing::info!("computer-use-mcp starting");

    let bind_addr = std::env::args().nth(1);
    server::run_http(bind_addr.as_deref()).await
}
