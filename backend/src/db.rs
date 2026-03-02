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

    #[tokio::test]
    #[ignore = "requires a running PostgreSQL instance (set DATABASE_URL env var)"]
    async fn create_pool_with_valid_url_succeeds() {
        let url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/feedback_test".to_string());
        let config = DatabaseConfig {
            url,
            max_connections: 5,
        };

        let pool = create_pool(&config).await.expect("Pool creation should succeed");
        let row: (i64,) = sqlx::query_as("SELECT 1").fetch_one(&pool).await.expect("Query should succeed");
        assert_eq!(row.0, 1);
    }

    #[tokio::test]
    #[ignore = "requires a running PostgreSQL instance (set DATABASE_URL env var)"]
    async fn run_migrations_succeeds_on_fresh_db() {
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let config = DatabaseConfig { url, max_connections: 5 };
        let pool = create_pool(&config).await.expect("Pool should be created");

        run_migrations(&pool).await.expect("Migrations should run without error");

        // Running migrations again is idempotent
        run_migrations(&pool).await.expect("Re-running migrations should be idempotent");
    }
}
