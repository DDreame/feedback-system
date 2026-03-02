use sqlx::PgPool;

use crate::error::AppError;
use crate::model::developer::{CreateDeveloper, Developer, DeveloperResponse};
use crate::utils::password::hash_password;

/// Register a new developer account.
///
/// Validates input, hashes the password, and inserts a new row into `developers`.
/// Returns `AppError::Conflict` if the email is already taken.
/// Returns `AppError::BadRequest` for invalid input (email format, password too short).
pub async fn register(pool: &PgPool, dto: CreateDeveloper) -> Result<DeveloperResponse, AppError> {
    validate_registration(&dto)?;

    let password_hash = hash_password(&dto.password)?;

    let developer = sqlx::query_as::<_, Developer>(
        "INSERT INTO developers (id, email, password_hash, name)
         VALUES (gen_random_uuid(), $1, $2, $3)
         RETURNING id, email, password_hash, name, created_at, updated_at",
    )
    .bind(&dto.email)
    .bind(&password_hash)
    .bind(&dto.name)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        // Unique-constraint violation → email already registered
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.constraint() == Some("developers_email_key") {
                return AppError::Conflict("Email already registered".to_string());
            }
        }
        AppError::Internal(format!("Failed to insert developer: {e}"))
    })?;

    Ok(DeveloperResponse::from(developer))
}

fn validate_registration(dto: &CreateDeveloper) -> Result<(), AppError> {
    if !dto.email.contains('@') || dto.email.len() < 3 {
        return Err(AppError::BadRequest("Invalid email address".to_string()));
    }
    if dto.password.len() < 8 {
        return Err(AppError::BadRequest(
            "Password must be at least 8 characters".to_string(),
        ));
    }
    if dto.name.trim().is_empty() {
        return Err(AppError::BadRequest("Name must not be empty".to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Unit tests for input validation (no DB required) ─────────────────────

    fn make_valid_dto(email: &str, password: &str, name: &str) -> CreateDeveloper {
        CreateDeveloper {
            email: email.to_string(),
            password: password.to_string(),
            name: name.to_string(),
        }
    }

    #[test]
    fn validate_rejects_invalid_email() {
        let dto = make_valid_dto("notanemail", "password123", "Alice");
        let err = validate_registration(&dto).unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }

    #[test]
    fn validate_rejects_short_password() {
        let dto = make_valid_dto("alice@example.com", "short", "Alice");
        let err = validate_registration(&dto).unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }

    #[test]
    fn validate_rejects_empty_name() {
        let dto = make_valid_dto("alice@example.com", "password123", "   ");
        let err = validate_registration(&dto).unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }

    #[test]
    fn validate_accepts_valid_input() {
        let dto = make_valid_dto("alice@example.com", "password123", "Alice");
        assert!(validate_registration(&dto).is_ok());
    }

    // ── Integration tests (require PostgreSQL) ────────────────────────────────

    async fn get_pool() -> PgPool {
        let _guard = crate::test_support::ENV_MUTEX.lock().unwrap();
        let _ = dotenvy::dotenv_override();
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(2)
            .connect(&url)
            .await
            .expect("should connect");
        crate::db::run_migrations(&pool).await.expect("migrations should run");
        pool
    }

    #[tokio::test]
    #[ignore = "requires a running PostgreSQL instance (set DATABASE_URL in backend/.env)"]
    async fn register_succeeds_with_valid_input() {
        let pool = get_pool().await;
        let email = format!("reg_ok_{}@example.com", uuid::Uuid::now_v7());

        let dto = make_valid_dto(&email, "password123", "Test Dev");
        let response = register(&pool, dto).await.expect("registration should succeed");

        assert_eq!(response.email, email);
        assert_eq!(response.name, "Test Dev");

        // Cleanup
        sqlx::query("DELETE FROM developers WHERE email = $1")
            .bind(&email)
            .execute(&pool)
            .await
            .expect("cleanup");
    }

    #[tokio::test]
    #[ignore = "requires a running PostgreSQL instance (set DATABASE_URL in backend/.env)"]
    async fn register_fails_with_duplicate_email() {
        let pool = get_pool().await;
        let email = format!("dup_{}@example.com", uuid::Uuid::now_v7());

        let dto1 = make_valid_dto(&email, "password123", "First");
        register(&pool, dto1).await.expect("first registration should succeed");

        let dto2 = make_valid_dto(&email, "password456", "Second");
        let err = register(&pool, dto2).await.expect_err("duplicate should fail");
        assert!(matches!(err, AppError::Conflict(_)));

        // Cleanup
        sqlx::query("DELETE FROM developers WHERE email = $1")
            .bind(&email)
            .execute(&pool)
            .await
            .expect("cleanup");
    }
}
