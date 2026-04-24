mod apps;
mod batch;
mod capture;
mod clipboard;
mod display;
mod error;
mod input;
mod server;
mod types;

use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_target(true)
        .with_thread_ids(true)
        .init();

    tracing::info!("mouseless starting");

    let arg = std::env::args().nth(1);
    match arg.as_deref() {
        Some("--stdio") | Some("stdio") => server::run_stdio().await,
        other => server::run_http(other).await,
    }
}
