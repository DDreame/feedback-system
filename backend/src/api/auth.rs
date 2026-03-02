use axum::{extract::State, http::StatusCode, Json};
use serde::Deserialize;

use crate::api::AppState;
use crate::error::AppError;
use crate::model::developer::CreateDeveloper;
use crate::service;
use crate::utils::jwt::{generate_token, validate_token, TokenKind};

pub async fn register(
    State(state): State<AppState>,
    Json(dto): Json<CreateDeveloper>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let developer = service::auth::register(&state.db, dto).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "developer": developer }))))
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let tokens = service::auth::login(&state.db, &req.email, &req.password, &state.jwt).await?;
    Ok(Json(serde_json::json!({
        "access_token": tokens.access_token,
        "refresh_token": tokens.refresh_token,
        "developer": tokens.developer,
    })))
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

/// Exchange a valid refresh token for a new access token.
pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Validate and assert kind == Refresh
    let claims = validate_token(&req.refresh_token, &state.jwt.secret, Some(TokenKind::Refresh))?;

    let developer_id = claims
        .sub
        .parse::<uuid::Uuid>()
        .map_err(|_| AppError::Unauthorized("Invalid token subject".to_string()))?;

    let new_access_token = generate_token(
        developer_id,
        &state.jwt.secret,
        TokenKind::Access,
        state.jwt.access_token_expiry_secs,
    )?;

    Ok(Json(serde_json::json!({ "access_token": new_access_token })))
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use sqlx::postgres::PgPoolOptions;
    use tower::ServiceExt;

    use crate::api::create_router;

    fn test_jwt() -> crate::config::JwtConfig {
        crate::config::JwtConfig {
            secret: "test-secret-at-least-32-chars!!".to_string(),
            access_token_expiry_secs: 3600,
            refresh_token_expiry_secs: 604800,
        }
    }

    fn lazy_router() -> axum::Router {
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgres://unused:unused@localhost/unused")
            .expect("lazy pool should be constructable");
        create_router(pool, test_jwt())
    }

    async fn real_router() -> axum::Router {
        let _guard = crate::test_support::ENV_MUTEX.lock().unwrap();
        let _ = dotenvy::dotenv_override();
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(&url)
            .await
            .expect("should connect");
        crate::db::run_migrations(&pool).await.expect("migrations should run");
        create_router(pool, test_jwt())
    }

    #[tokio::test]
    async fn register_empty_body_returns_4xx() {
        // Axum returns 400 for EOF when parsing JSON from an empty body
        let app = lazy_router();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/register")
                    .header("content-type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(
            response.status().is_client_error(),
            "empty body should return 4xx, got {}",
            response.status()
        );
    }

    #[tokio::test]
    async fn register_malformed_json_returns_400() {
        let app = lazy_router();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/register")
                    .header("content-type", "application/json")
                    .body(Body::from("not-json"))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Axum returns 400 for syntax errors and 422 for type mismatches
        assert!(
            response.status() == StatusCode::BAD_REQUEST
                || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
            "unexpected status: {}",
            response.status()
        );
    }

    #[tokio::test]
    #[ignore = "requires a running PostgreSQL instance (set DATABASE_URL in backend/.env)"]
    async fn register_returns_201_with_developer() {
        let app = real_router().await;
        let email = format!("api_reg_{}@example.com", uuid::Uuid::now_v7());

        let body = serde_json::json!({
            "email": email,
            "password": "password123",
            "name": "API Test Dev"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/register")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(json["developer"]["email"], email);
        assert_eq!(json["developer"]["name"], "API Test Dev");
        assert!(json["developer"]["id"].is_string());
        assert!(!json["developer"].as_object().unwrap().contains_key("password_hash"));
    }

    #[tokio::test]
    #[ignore = "requires a running PostgreSQL instance (set DATABASE_URL in backend/.env)"]
    async fn register_duplicate_email_returns_409() {
        let app = real_router().await;
        let email = format!("api_dup_{}@example.com", uuid::Uuid::now_v7());

        let body = serde_json::json!({
            "email": email,
            "password": "password123",
            "name": "First"
        });

        // First registration
        app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/register")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Duplicate
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/register")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    #[ignore = "requires a running PostgreSQL instance (set DATABASE_URL in backend/.env)"]
    async fn register_invalid_email_returns_400() {
        let app = real_router().await;

        let body = serde_json::json!({
            "email": "not-an-email",
            "password": "password123",
            "name": "Test"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/register")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    // ── Login endpoint tests ──────────────────────────────────────────────────

    #[tokio::test]
    async fn login_empty_body_returns_4xx() {
        let app = lazy_router();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/login")
                    .header("content-type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(response.status().is_client_error(), "got {}", response.status());
    }

    #[tokio::test]
    #[ignore = "requires a running PostgreSQL instance (set DATABASE_URL in backend/.env)"]
    async fn login_returns_200_with_tokens() {
        let app = real_router().await;
        let email = format!("api_login_{}@example.com", uuid::Uuid::now_v7());

        // Register first
        let reg_body = serde_json::json!({ "email": email, "password": "password123", "name": "L" });
        app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/register")
                    .header("content-type", "application/json")
                    .body(Body::from(reg_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let login_body = serde_json::json!({ "email": email, "password": "password123" });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/login")
                    .header("content-type", "application/json")
                    .body(Body::from(login_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        assert!(json["access_token"].is_string());
        assert!(json["refresh_token"].is_string());
        assert_eq!(json["developer"]["email"], email);
        assert!(!json["developer"].as_object().unwrap().contains_key("password_hash"));
    }

    #[tokio::test]
    #[ignore = "requires a running PostgreSQL instance (set DATABASE_URL in backend/.env)"]
    async fn login_wrong_password_returns_401() {
        let app = real_router().await;
        let email = format!("api_badpw_{}@example.com", uuid::Uuid::now_v7());

        let reg_body = serde_json::json!({ "email": email, "password": "password123", "name": "X" });
        app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/register")
                    .header("content-type", "application/json")
                    .body(Body::from(reg_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let login_body = serde_json::json!({ "email": email, "password": "wrongpassword" });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/login")
                    .header("content-type", "application/json")
                    .body(Body::from(login_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    // ── Refresh endpoint tests ────────────────────────────────────────────────

    #[tokio::test]
    async fn refresh_with_valid_refresh_token_returns_new_access_token() {
        let jwt = test_jwt();
        let dev_id = uuid::Uuid::now_v7();
        let refresh_token =
            crate::utils::jwt::generate_token(dev_id, &jwt.secret, crate::utils::jwt::TokenKind::Refresh, 604800)
                .unwrap();

        let body = serde_json::json!({ "refresh_token": refresh_token });
        let response = lazy_router()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/refresh")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(json["access_token"].is_string(), "response must contain access_token");

        // The new access token should be a valid access token for the same developer
        let claims = crate::utils::jwt::validate_token(
            json["access_token"].as_str().unwrap(),
            &jwt.secret,
            Some(crate::utils::jwt::TokenKind::Access),
        )
        .expect("returned access token must be valid");
        assert_eq!(claims.sub, dev_id.to_string());
    }

    #[tokio::test]
    async fn refresh_with_access_token_returns_401() {
        let jwt = test_jwt();
        let dev_id = uuid::Uuid::now_v7();
        let access_token =
            crate::utils::jwt::generate_token(dev_id, &jwt.secret, crate::utils::jwt::TokenKind::Access, 3600)
                .unwrap();

        let body = serde_json::json!({ "refresh_token": access_token });
        let response = lazy_router()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/refresh")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn refresh_with_invalid_token_returns_401() {
        let body = serde_json::json!({ "refresh_token": "not.a.jwt" });
        let response = lazy_router()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/refresh")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
