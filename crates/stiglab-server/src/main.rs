use stiglab_server::{
    build_router, config::ServerConfig, db, spine::SpineEmitter, state::AppState,
};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let config = ServerConfig::from_env();
    tracing::info!("starting stiglab server on {}:{}", config.host, config.port);

    tracing::info!("connecting to database...");
    let pool = db::init_pool(&config.database_url).await?;
    tracing::info!("database connected");

    // Connect to Onsager event spine if configured
    let spine = if let Ok(url) = std::env::var("ONSAGER_DATABASE_URL") {
        tracing::info!("connecting to onsager event spine...");
        match SpineEmitter::connect(&url).await {
            Ok(emitter) => {
                tracing::info!("onsager event spine connected");
                Some(emitter)
            }
            Err(e) => {
                tracing::warn!("failed to connect to onsager event spine: {e}");
                None
            }
        }
    } else {
        tracing::info!("ONSAGER_DATABASE_URL not set, spine events disabled");
        None
    };

    let state = AppState::new(pool, config.clone(), spine);
    let app = build_router(state, &config);

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("listening on {addr}");

    axum::serve(listener, app).await?;

    Ok(())
}
