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
}
