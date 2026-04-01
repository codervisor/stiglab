mod config;
mod connection;
mod session;

use clap::Parser;
use tracing_subscriber::EnvFilter;

use config::AgentConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let config = AgentConfig::parse();
    tracing::info!("stiglab agent starting");
    tracing::info!("  node name: {}", config.node_name());
    tracing::info!("  server: {}", config.server);
    tracing::info!("  max sessions: {}", config.max_sessions);

    // Connect with reconnection logic
    loop {
        match connection::connect_and_run(config.clone()).await {
            Ok(()) => {
                tracing::info!("connection closed, reconnecting in 5s...");
            }
            Err(e) => {
                tracing::error!("connection error: {e}, reconnecting in 5s...");
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}
