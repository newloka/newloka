#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let bind = std::env::var("NEWLOKA_BIND").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    tracing_subscriber::fmt::init();
    newloka_server::run(&bind).await
}
