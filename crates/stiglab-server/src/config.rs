use std::env;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub static_dir: Option<String>,
    pub cors_origin: Option<String>,
}

impl ServerConfig {
    pub fn from_env() -> Self {
        let host = env::var("STIGLAB_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .or_else(|| env::var("STIGLAB_PORT").ok().and_then(|p| p.parse().ok()))
            .unwrap_or(3000);
        let database_url =
            env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://./data/stiglab.db".to_string());
        let static_dir = env::var("STIGLAB_STATIC_DIR").ok();
        let cors_origin = env::var("STIGLAB_CORS_ORIGIN").ok();

        ServerConfig {
            host,
            port,
            database_url,
            static_dir,
            cors_origin,
        }
    }
}
