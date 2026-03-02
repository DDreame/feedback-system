use axum::{
    extract::FromRequestParts,
    http::{request::Parts, HeaderMap},
};
use uuid::Uuid;

use crate::api::AppState;
use crate::error::AppError;
use crate::model::project::ProjectResponse;
use crate::service;
use crate::utils::jwt::{validate_token, TokenKind};

/// Axum extractor for authenticated developer requests.
///
/// Expects `Authorization: Bearer <access_token>` in the request headers.
/// On success, holds the developer's UUID.
/// On failure, returns `AppError::Unauthorized`.
pub struct AuthDeveloper {
    pub developer_id: Uuid,
}

impl FromRequestParts<AppState> for AuthDeveloper {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let token = extract_bearer_token(&parts.headers)?;
        let claims = validate_token(token, &state.jwt.secret, Some(TokenKind::Access))?;
        let developer_id = claims
            .sub
            .parse::<Uuid>()
            .map_err(|_| AppError::Unauthorized("Invalid token subject".to_string()))?;
        Ok(AuthDeveloper { developer_id })
    }
}

/// Axum extractor for SDK endpoints authenticated with an API key.
///
/// Expects `X-API-Key: proj_<...>` header.
/// On success, holds the full `ProjectResponse` for the matched project.
/// On failure, returns `AppError::Unauthorized`.
pub struct AuthProject {
    pub project: ProjectResponse,
}

impl FromRequestParts<AppState> for AuthProject {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let api_key = extract_api_key(&parts.headers)?;
        let project = service::project::get_by_api_key(&state.db, api_key).await?;
        Ok(AuthProject { project })
    }
}

fn extract_bearer_token(headers: &HeaderMap) -> Result<&str, AppError> {
    let header_value = headers
        .get("Authorization")
        .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".to_string()))?;

    let header_str = header_value
        .to_str()
        .map_err(|_| AppError::Unauthorized("Invalid Authorization header".to_string()))?;

    header_str
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("Authorization header must use Bearer scheme".to_string()))
}

fn extract_api_key(headers: &HeaderMap) -> Result<&str, AppError> {
    headers
        .get("X-API-Key")
        .ok_or_else(|| AppError::Unauthorized("Missing X-API-Key header".to_string()))?
        .to_str()
        .map_err(|_| AppError::Unauthorized("Invalid X-API-Key header".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::JwtConfig;
    use crate::utils::jwt::{generate_token, TokenKind};
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        Json, Router,
    };
    use http_body_util::BodyExt;
    use sqlx::postgres::PgPoolOptions;
    use tower::ServiceExt;

    fn test_jwt() -> JwtConfig {
        JwtConfig {
            secret: "test-secret-at-least-32-chars!!".to_string(),
            access_token_expiry_secs: 3600,
            refresh_token_expiry_secs: 604800,
        }
    }

    /// A protected endpoint used for testing the extractor.
    async fn protected(auth: AuthDeveloper) -> Json<serde_json::Value> {
        Json(serde_json::json!({ "developer_id": auth.developer_id.to_string() }))
    }

    fn test_router() -> Router {
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgres://unused:unused@localhost/unused")
            .expect("lazy pool");
        let state = crate::api::AppState { db: pool, jwt: test_jwt() };
        Router::new()
            .route("/protected", get(protected))
            .with_state(state)
    }

    #[tokio::test]
    async fn valid_token_grants_access() {
        let dev_id = Uuid::now_v7();
        let token = generate_token(dev_id, &test_jwt().secret, TokenKind::Access, 3600).unwrap();

        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header("Authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["developer_id"], dev_id.to_string());
    }

    #[tokio::test]
    async fn missing_auth_header_returns_401() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn invalid_token_returns_401() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header("Authorization", "Bearer not.a.valid.jwt")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn refresh_token_rejected_on_access_endpoint() {
        let dev_id = Uuid::now_v7();
        // Issue a refresh token, not an access token
        let token = generate_token(dev_id, &test_jwt().secret, TokenKind::Refresh, 604800).unwrap();

        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header("Authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn wrong_bearer_scheme_returns_401() {
        let dev_id = Uuid::now_v7();
        let token = generate_token(dev_id, &test_jwt().secret, TokenKind::Access, 3600).unwrap();

        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header("Authorization", format!("Token {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    // ── AuthProject (SDK API key) tests ──────────────────────────────────────

    async fn real_sdk_router() -> Router {
        let _guard = crate::test_support::ENV_MUTEX.lock().unwrap();
        let _ = dotenvy::dotenv_override();
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPoolOptions::new().max_connections(2).connect(&url).await.expect("connect");
        crate::db::run_migrations(&pool).await.expect("migrations");
        let state = crate::api::AppState { db: pool, jwt: test_jwt() };

        async fn sdk_endpoint(auth: AuthProject) -> Json<serde_json::Value> {
            Json(serde_json::json!({ "project_id": auth.project.id.to_string() }))
        }

        Router::new()
            .route("/sdk/test", get(sdk_endpoint))
            .with_state(state)
    }

    #[tokio::test]
    async fn valid_api_key_grants_sdk_access() {
        let app = real_sdk_router().await;

        // Create a developer + project to get a real API key
        let _guard = crate::test_support::ENV_MUTEX.lock().unwrap();
        let _ = dotenvy::dotenv_override();
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPoolOptions::new().max_connections(2).connect(&url).await.expect("connect");

        let dev_id = uuid::Uuid::now_v7();
        sqlx::query("INSERT INTO developers (id, email, password_hash, name) VALUES ($1,$2,$3,$4)")
            .bind(dev_id).bind(format!("sdk_mw_{}@test.com", dev_id))
            .bind("$argon2id$hash").bind("SDK MW Dev")
            .execute(&pool).await.expect("insert dev");

        let dto = crate::model::project::CreateProject { name: "SDK App".to_string(), description: None };
        let project = crate::service::project::create(&pool, dev_id, dto).await.expect("create project");

        let response = app.oneshot(
            Request::builder().uri("/sdk/test")
                .header("X-API-Key", &project.api_key)
                .body(Body::empty()).unwrap()
        ).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["project_id"], project.id.to_string());

        // Cleanup
        sqlx::query("DELETE FROM projects WHERE developer_id = $1").bind(dev_id).execute(&pool).await.ok();
        sqlx::query("DELETE FROM developers WHERE id = $1").bind(dev_id).execute(&pool).await.ok();
    }

    #[tokio::test]
    async fn missing_api_key_header_returns_401() {
        // We need a real router here but the SDK endpoint just needs the extractor to reject
        let app = real_sdk_router().await;
        let response = app.oneshot(
            Request::builder().uri("/sdk/test").body(Body::empty()).unwrap()
        ).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn invalid_api_key_returns_401() {
        let app = real_sdk_router().await;
        let response = app.oneshot(
            Request::builder().uri("/sdk/test")
                .header("X-API-Key", "proj_invalid_key_that_does_not_exist")
                .body(Body::empty()).unwrap()
        ).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
