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

const VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_HTTP_ADDR: &str = "127.0.0.1:3100";

const HELP: &str = "\
mouseless {VERSION} — macOS desktop control over MCP

USAGE:
    mouseless                    Start in stdio mode (default)
    mouseless --http [ADDR]      Start HTTP server (default: 127.0.0.1:3100)
    mouseless --stdio            Explicit stdio mode (same as default)
    mouseless -h, --help         Show this help
    mouseless -V, --version      Show version

ENVIRONMENT:
    RUST_LOG                     Log filter (default: info)

macOS permissions required:
  • Accessibility        (System Settings > Privacy & Security > Accessibility)
  • Screen Recording     (System Settings > Privacy & Security > Screen Recording)
";

enum Mode {
    Stdio,
    Http(Option<String>),
}

fn parse_args() -> Result<Mode, String> {
    let mut args = std::env::args().skip(1);
    let mut mode = Mode::Stdio;
    while let Some(a) = args.next() {
        match a.as_str() {
            "-h" | "--help" => {
                println!("{}", HELP.replace("{VERSION}", VERSION));
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!("mouseless {VERSION}");
                std::process::exit(0);
            }
            "--stdio" | "stdio" => mode = Mode::Stdio,
            "--http" => {
                let addr = args.next();
                mode = Mode::Http(addr);
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    Ok(mode)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mode = match parse_args() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("error: {e}\n");
            eprintln!("{}", HELP.replace("{VERSION}", VERSION));
            std::process::exit(2);
        }
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_target(true)
        .with_thread_ids(true)
        .init();

    match mode {
        Mode::Stdio => {
            eprintln!("▸ mouseless {VERSION} — stdio mode");
            eprintln!("  listening on stdin/stdout. press ctrl+c to stop.");
            eprintln!(
                "  needs Accessibility + Screen Recording (System Settings > Privacy & Security)."
            );
            server::run_stdio().await
        }
        Mode::Http(addr) => {
            let addr_str = addr.as_deref().unwrap_or(DEFAULT_HTTP_ADDR);
            eprintln!("▸ mouseless {VERSION} — HTTP mode");
            eprintln!("  serving MCP at http://{addr_str}/mcp");
            eprintln!("  press ctrl+c to stop.");
            eprintln!(
                "  needs Accessibility + Screen Recording (System Settings > Privacy & Security)."
            );
            server::run_http(addr.as_deref()).await
        }
    }
}
