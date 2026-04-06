use stiglab_server::{build_router, config::ServerConfig, db, state::AppState};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let config = ServerConfig::from_env();
    tracing::info!("starting stiglab server on {}:{}", config.host, config.port);
    tracing::info!("database configured");

    let pool = db::init_pool(&config.database_url).await?;
    let state = AppState::new(pool);
    let app = build_router(state, &config);

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("listening on {addr}");

    axum::serve(listener, app).await?;

    Ok(())
}
