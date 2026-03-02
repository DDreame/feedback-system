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

    #[tokio::test]
    #[ignore = "requires a running PostgreSQL instance (set DATABASE_URL in backend/.env)"]
    async fn developers_table_exists_with_correct_columns() {
        let config = DatabaseConfig { url: db_url_from_env(), max_connections: 5 };
        let pool = create_pool(&config).await.expect("Pool should be created");
        run_migrations(&pool).await.expect("Migrations should run");

        // Check table exists and has the expected columns
        let columns: Vec<(String, String)> = sqlx::query_as(
            "SELECT column_name, data_type
             FROM information_schema.columns
             WHERE table_name = 'developers'
             ORDER BY column_name",
        )
        .fetch_all(&pool)
        .await
        .expect("Should query information_schema");

        let column_map: std::collections::HashMap<_, _> = columns.into_iter().collect();

        assert!(column_map.contains_key("id"), "id column missing");
        assert!(column_map.contains_key("email"), "email column missing");
        assert!(column_map.contains_key("password_hash"), "password_hash column missing");
        assert!(column_map.contains_key("name"), "name column missing");
        assert!(column_map.contains_key("created_at"), "created_at column missing");
        assert!(column_map.contains_key("updated_at"), "updated_at column missing");

        assert_eq!(column_map["id"], "uuid");
        assert_eq!(column_map["email"], "character varying");
        assert_eq!(column_map["password_hash"], "character varying");
        assert_eq!(column_map["name"], "character varying");
    }

    #[tokio::test]
    async fn projects_table_exists_with_correct_columns() {
        let config = DatabaseConfig { url: db_url_from_env(), max_connections: 5 };
        let pool = create_pool(&config).await.expect("Pool should be created");
        run_migrations(&pool).await.expect("Migrations should run");

        let columns: Vec<(String, String)> = sqlx::query_as(
            "SELECT column_name, data_type
             FROM information_schema.columns
             WHERE table_name = 'projects'
             ORDER BY column_name",
        )
        .fetch_all(&pool)
        .await
        .expect("Should query information_schema");

        let column_map: std::collections::HashMap<_, _> = columns.into_iter().collect();

        assert!(column_map.contains_key("id"), "id column missing");
        assert!(column_map.contains_key("developer_id"), "developer_id column missing");
        assert!(column_map.contains_key("name"), "name column missing");
        assert!(column_map.contains_key("description"), "description column missing");
        assert!(column_map.contains_key("api_key"), "api_key column missing");
        assert!(column_map.contains_key("created_at"), "created_at column missing");
        assert!(column_map.contains_key("updated_at"), "updated_at column missing");

        assert_eq!(column_map["id"], "uuid");
        assert_eq!(column_map["developer_id"], "uuid");
        assert_eq!(column_map["api_key"], "character varying");

        // Verify api_key unique constraint exists
        let unique_constraints: Vec<(String,)> = sqlx::query_as(
            "SELECT constraint_name FROM information_schema.table_constraints
             WHERE table_name = 'projects' AND constraint_type = 'UNIQUE'",
        )
        .fetch_all(&pool)
        .await
        .expect("Should query constraints");

        assert!(
            unique_constraints.iter().any(|(name,)| name.contains("api_key")),
            "api_key unique constraint missing"
        );
    }

    #[tokio::test]
    async fn end_users_and_conversations_tables_exist() {
        let config = DatabaseConfig { url: db_url_from_env(), max_connections: 5 };
        let pool = create_pool(&config).await.expect("Pool should be created");
        run_migrations(&pool).await.expect("Migrations should run");

        // Check end_users table columns
        let eu_columns: Vec<(String, String)> = sqlx::query_as(
            "SELECT column_name, data_type
             FROM information_schema.columns
             WHERE table_name = 'end_users'
             ORDER BY column_name",
        )
        .fetch_all(&pool)
        .await
        .expect("Should query end_users columns");

        let eu_map: std::collections::HashMap<_, _> = eu_columns.into_iter().collect();
        assert!(eu_map.contains_key("id"), "end_users.id missing");
        assert!(eu_map.contains_key("project_id"), "end_users.project_id missing");
        assert!(eu_map.contains_key("device_id"), "end_users.device_id missing");
        assert!(eu_map.contains_key("name"), "end_users.name missing");
        assert!(eu_map.contains_key("created_at"), "end_users.created_at missing");
        assert!(eu_map.contains_key("updated_at"), "end_users.updated_at missing");
        assert_eq!(eu_map["id"], "uuid");
        assert_eq!(eu_map["project_id"], "uuid");

        // Verify (project_id, device_id) unique constraint
        let eu_unique: Vec<(String,)> = sqlx::query_as(
            "SELECT constraint_name FROM information_schema.table_constraints
             WHERE table_name = 'end_users' AND constraint_type = 'UNIQUE'",
        )
        .fetch_all(&pool)
        .await
        .expect("Should query end_users constraints");
        assert!(!eu_unique.is_empty(), "end_users unique constraint missing");

        // Check conversations table columns
        let cv_columns: Vec<(String, String)> = sqlx::query_as(
            "SELECT column_name, data_type
             FROM information_schema.columns
             WHERE table_name = 'conversations'
             ORDER BY column_name",
        )
        .fetch_all(&pool)
        .await
        .expect("Should query conversations columns");

        let cv_map: std::collections::HashMap<_, _> = cv_columns.into_iter().collect();
        assert!(cv_map.contains_key("id"), "conversations.id missing");
        assert!(cv_map.contains_key("project_id"), "conversations.project_id missing");
        assert!(cv_map.contains_key("end_user_id"), "conversations.end_user_id missing");
        assert!(cv_map.contains_key("status"), "conversations.status missing");
        assert!(cv_map.contains_key("created_at"), "conversations.created_at missing");
        assert!(cv_map.contains_key("updated_at"), "conversations.updated_at missing");
        assert_eq!(cv_map["id"], "uuid");
        assert_eq!(cv_map["end_user_id"], "uuid");
    }
}
