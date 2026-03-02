mod api;
mod config;
mod db;
mod error;

use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

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
