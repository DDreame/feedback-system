use sqlx::PgPool;

use crate::config::JwtConfig;
use crate::error::AppError;
use crate::model::developer::{CreateDeveloper, Developer, DeveloperResponse};
use crate::utils::jwt::{generate_token, TokenKind};
use crate::utils::password::{hash_password, verify_password};

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

/// Tokens returned by a successful login.
#[derive(Debug)]
pub struct LoginTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub developer: DeveloperResponse,
}

/// Authenticate a developer and return a signed access + refresh token pair.
///
/// Returns `AppError::Unauthorized` for any authentication failure
/// (unknown email or wrong password) — deliberately no distinction to
/// prevent user enumeration.
pub async fn login(
    pool: &PgPool,
    email: &str,
    password: &str,
    jwt: &JwtConfig,
) -> Result<LoginTokens, AppError> {
    const AUTH_FAILED: &str = "Invalid email or password";

    let developer: Developer = sqlx::query_as(
        "SELECT id, email, password_hash, name, created_at, updated_at
         FROM developers WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error during login: {e}")))?
    .ok_or_else(|| AppError::Unauthorized(AUTH_FAILED.to_string()))?;

    if !verify_password(password, &developer.password_hash)? {
        return Err(AppError::Unauthorized(AUTH_FAILED.to_string()));
    }

    let dev_uuid = developer.id;
    let access_token = generate_token(dev_uuid, &jwt.secret, TokenKind::Access, jwt.access_token_expiry_secs)?;
    let refresh_token = generate_token(dev_uuid, &jwt.secret, TokenKind::Refresh, jwt.refresh_token_expiry_secs)?;

    Ok(LoginTokens {
        access_token,
        refresh_token,
        developer: DeveloperResponse::from(developer),
    })
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

    // ── Login integration tests ───────────────────────────────────────────────

    fn test_jwt_config() -> crate::config::JwtConfig {
        crate::config::JwtConfig {
            secret: "test-secret-at-least-32-chars-long!!".to_string(),
            access_token_expiry_secs: 3600,
            refresh_token_expiry_secs: 604800,
        }
    }

    #[tokio::test]
    #[ignore = "requires a running PostgreSQL instance (set DATABASE_URL in backend/.env)"]
    async fn login_returns_tokens_for_valid_credentials() {
        let pool = get_pool().await;
        let email = format!("login_ok_{}@example.com", uuid::Uuid::now_v7());
        let password = "password123";

        // Register first
        let dto = make_valid_dto(&email, password, "Login Test");
        register(&pool, dto).await.expect("registration");

        let tokens = login(&pool, &email, password, &test_jwt_config())
            .await
            .expect("login should succeed");

        assert!(!tokens.access_token.is_empty());
        assert!(!tokens.refresh_token.is_empty());
        assert_eq!(tokens.developer.email, email);

        // Verify the tokens are valid JWTs with the correct kind
        let access_claims = crate::utils::jwt::validate_token(
            &tokens.access_token,
            &test_jwt_config().secret,
            Some(crate::utils::jwt::TokenKind::Access),
        )
        .expect("access token must be valid");
        assert_eq!(access_claims.sub, tokens.developer.id.to_string());

        crate::utils::jwt::validate_token(
            &tokens.refresh_token,
            &test_jwt_config().secret,
            Some(crate::utils::jwt::TokenKind::Refresh),
        )
        .expect("refresh token must be valid");

        // Cleanup
        sqlx::query("DELETE FROM developers WHERE email = $1")
            .bind(&email)
            .execute(&pool)
            .await
            .expect("cleanup");
    }

    #[tokio::test]
    #[ignore = "requires a running PostgreSQL instance (set DATABASE_URL in backend/.env)"]
    async fn login_fails_with_wrong_password() {
        let pool = get_pool().await;
        let email = format!("login_bad_{}@example.com", uuid::Uuid::now_v7());

        register(&pool, make_valid_dto(&email, "correct_pass", "Dev"))
            .await
            .expect("registration");

        let err = login(&pool, &email, "wrong_pass", &test_jwt_config())
            .await
            .expect_err("wrong password should fail");
        assert!(matches!(err, AppError::Unauthorized(_)));

        sqlx::query("DELETE FROM developers WHERE email = $1")
            .bind(&email)
            .execute(&pool)
            .await
            .expect("cleanup");
    }

    #[tokio::test]
    #[ignore = "requires a running PostgreSQL instance (set DATABASE_URL in backend/.env)"]
    async fn login_fails_with_unknown_email() {
        let pool = get_pool().await;
        let err = login(&pool, "nobody@example.com", "any_pass", &test_jwt_config())
            .await
            .expect_err("unknown email should fail");
        assert!(matches!(err, AppError::Unauthorized(_)));
    }
}
