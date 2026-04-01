mod config;
mod db;
mod routes;
mod state;
mod ws;

use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tracing_subscriber::EnvFilter;

use config::ServerConfig;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let config = ServerConfig::from_env();
    tracing::info!("starting stiglab server on {}:{}", config.host, config.port);
    tracing::info!("database: {}", config.database_url);

    let pool = db::init_pool(&config.database_url).await?;
    let state = AppState::new(pool);

    // API routes
    let api_routes = Router::new()
        .route("/api/health", get(routes::health::health))
        .route("/api/nodes", get(routes::nodes::list_nodes))
        .route("/api/tasks", post(routes::tasks::create_task))
        .route("/api/sessions", get(routes::sessions::list_sessions))
        .route("/api/sessions/{id}", get(routes::sessions::get_session))
        .route(
            "/api/sessions/{id}/logs",
            get(routes::sessions::session_logs),
        )
        .route("/agent/ws", get(ws::agent::agent_ws_handler));

    // Configure CORS: restrict to specified origin in production, permissive in dev
    let cors = if let Some(ref origin) = config.cors_origin {
        tracing::info!("CORS restricted to origin: {origin}");
        CorsLayer::new()
            .allow_origin(
                origin
                    .parse::<axum::http::HeaderValue>()
                    .expect("invalid CORS origin"),
            )
            .allow_methods(tower_http::cors::Any)
            .allow_headers(tower_http::cors::Any)
    } else {
        tracing::warn!("CORS is permissive (set STIGLAB_CORS_ORIGIN to restrict)");
        CorsLayer::permissive()
    };

    let mut app = api_routes.with_state(state).layer(cors);

    // Serve static UI files if configured
    if let Some(ref static_dir) = config.static_dir {
        tracing::info!("serving static files from {static_dir}");
        let index_file = format!("{static_dir}/index.html");
        app = app.fallback_service(ServeDir::new(static_dir).fallback(ServeFile::new(index_file)));
    }

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("listening on {addr}");

    axum::serve(listener, app).await?;

    Ok(())
}
