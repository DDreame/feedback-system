use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::config::DatabaseConfig;
use crate::error::AppError;

pub async fn create_pool(config: &DatabaseConfig) -> Result<PgPool, AppError> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.url)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to connect to database: {e}")))
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), AppError> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to run database migrations: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DatabaseConfig;

    #[tokio::test]
    async fn create_pool_with_invalid_url_returns_error() {
        let config = DatabaseConfig {
            url: "not_a_valid_postgres_url".to_string(),
            max_connections: 5,
        };

        let result = create_pool(&config).await;
        assert!(result.is_err());
    }

    fn db_url_from_env() -> String {
        // Hold ENV_MUTEX while loading the .env file and reading the URL so this
        // doesn't race with config tests that temporarily mutate env vars.
        let _guard = crate::test_support::ENV_MUTEX.lock().unwrap();
        let _ = dotenvy::dotenv_override();
        std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set in backend/.env")
    }

    #[tokio::test]
    #[ignore = "requires a running PostgreSQL instance (set DATABASE_URL in backend/.env)"]
    async fn create_pool_with_valid_url_succeeds() {
        let config = DatabaseConfig { url: db_url_from_env(), max_connections: 5 };

        let pool = create_pool(&config).await.expect("Pool creation should succeed");
        // SELECT 1 returns INT4 in PostgreSQL, so decode as i32.
        let row: (i32,) = sqlx::query_as("SELECT 1").fetch_one(&pool).await.expect("Query should succeed");
        assert_eq!(row.0, 1);
    }

    #[tokio::test]
    #[ignore = "requires a running PostgreSQL instance (set DATABASE_URL in backend/.env)"]
    async fn run_migrations_succeeds_on_fresh_db() {
        let url = db_url_from_env();
        let config = DatabaseConfig { url, max_connections: 5 };
        let pool = create_pool(&config).await.expect("Pool should be created");

        run_migrations(&pool).await.expect("Migrations should run without error");

        // Running migrations again is idempotent
        run_migrations(&pool).await.expect("Re-running migrations should be idempotent");
    }
}
