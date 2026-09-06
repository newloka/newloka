//! New Loka Server binary
use clap::Parser;

#[derive(Parser)]
#[command(name = "newloka-server")]
#[command(about = "New Loka HTTP server")]
struct Args {
    /// Bind address
    #[arg(short, long, default_value = "127.0.0.1:8080")]
    bind: String,

    /// Deployment tier (e.g. T0, T1, T2, T3, T4)
    #[arg(short, long)]
    tier: Option<String>,

    /// Database path or SQLite connection string (e.g. ./clinic.db, /path/to/clinic.db, or :memory:)
    #[arg(short, long)]
    db: Option<String>,

    /// Node ID identifier
    #[arg(short, long)]
    node_id: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if let Some(tier) = args.tier {
        std::env::set_var("NEWLOKA_TIER", tier);
    }
    if let Some(db) = args.db {
        std::env::set_var("NEWLOKA_DB_PATH", db);
    }
    if let Some(node) = args.node_id {
        std::env::set_var("NEWLOKA_NODE_ID", node);
    }
    tracing_subscriber::fmt::init();
    newloka_server::run(&args.bind).await
}
