mod api;
mod config;
mod db;
mod error;
mod model;
mod service;
mod utils;

/// Shared test infrastructure. Only compiled during `cargo test`.
#[cfg(test)]
pub(crate) mod test_support {
    /// Serialize every test that reads or writes process environment variables.
    /// Acquire this lock at the start of any test using `std::env::set_var`,
    /// `remove_var`, `var`, or `dotenvy` calls so they don't race each other.
    pub static ENV_MUTEX: std::sync::LazyLock<std::sync::Mutex<()>> =
        std::sync::LazyLock::new(|| std::sync::Mutex::new(()));
}

use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Load .env file before reading config (ignored if file doesn't exist)
    let _ = dotenvy::dotenv();

    let config = config::AppConfig::from_env().expect("Failed to load configuration");

    let pool = db::create_pool(&config.database)
        .await
        .expect("Failed to connect to database");

    db::run_migrations(&pool)
        .await
        .expect("Failed to run database migrations");

    let addr = SocketAddr::new(config.server.host.parse().expect("Invalid host"), config.server.port);
    let app = api::create_router(pool);

    tracing::info!("Listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.expect("Failed to bind");
    axum::serve(listener, app).await.expect("Server error");
}

#[cfg(test)]
mod tests {
    #[test]
    fn project_compiles() {
        assert!(true);
    }
}
