//! New Loka root launcher binary.
//!
//! Thin entry point that starts the HTTP server using environment
//! configuration (`NEWLOKA_BIND_ADDR`). This is the binary packaged in the
//! Docker image; the richer `newloka-cli` and `newloka-server` binaries remain
//! available for interactive use.

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let bind = std::env::var("NEWLOKA_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    newloka_server::run(&bind).await
}
