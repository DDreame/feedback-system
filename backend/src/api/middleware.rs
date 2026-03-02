use axum::{
    extract::FromRequestParts,
    http::{request::Parts, HeaderMap},
};
use uuid::Uuid;

use crate::api::AppState;
use crate::error::AppError;
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
}
