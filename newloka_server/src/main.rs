//! New Loka Server binary
use clap::Parser;

#[derive(Parser)]
#[command(name = "newloka-server")]
#[command(about = "New Loka HTTP server")]
struct Args {
    /// Bind address
    #[arg(short, long, default_value = "127.0.0.1:8080")]
    bind: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    tracing_subscriber::fmt::init();
    newloka_server::run(&args.bind).await
}
