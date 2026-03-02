use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::api::middleware::AuthProject;
use crate::api::AppState;
use crate::error::AppError;
use crate::model::conversation::ConversationResponse;
use crate::model::end_user::EndUserResponse;
use crate::service;

#[derive(Debug, Deserialize)]
pub struct SdkInitRequest {
    pub device_id: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SdkInitResponse {
    pub end_user: EndUserResponse,
    pub conversation: ConversationResponse,
}

/// POST /api/v1/sdk/init
///
/// Creates or resumes a session for an end user identified by `device_id`.
/// Requires `X-API-Key` header with a valid project API key.
pub async fn init(
    auth: AuthProject,
    State(state): State<AppState>,
    Json(body): Json<SdkInitRequest>,
) -> Result<(StatusCode, Json<SdkInitResponse>), AppError> {
    let project_id = auth.project.id;

    let end_user = service::sdk::find_or_create_end_user(
        &state.db,
        project_id,
        &body.device_id,
        body.name.as_deref(),
    )
    .await?;

    let conversation =
        service::sdk::find_or_create_conversation(&state.db, project_id, end_user.id).await?;

    Ok((StatusCode::OK, Json(SdkInitResponse { end_user, conversation })))
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        Router,
    };
    use http_body_util::BodyExt;
    use sqlx::postgres::PgPoolOptions;
    use tower::ServiceExt;

    use crate::config::JwtConfig;

    fn test_jwt() -> JwtConfig {
        JwtConfig {
            secret: "test-secret-at-least-32-chars!!".to_string(),
            access_token_expiry_secs: 3600,
            refresh_token_expiry_secs: 604800,
        }
    }

    async fn sdk_test_app() -> (Router, sqlx::PgPool) {
        let _guard = crate::test_support::ENV_MUTEX.lock().unwrap();
        let _ = dotenvy::dotenv_override();
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPoolOptions::new().max_connections(2).connect(&url).await.expect("connect");
        crate::db::run_migrations(&pool).await.expect("migrations");
        let app = crate::api::create_router(pool.clone(), test_jwt());
        (app, pool)
    }

    async fn create_dev_and_project(pool: &sqlx::PgPool) -> (uuid::Uuid, String) {
        let dev_id = uuid::Uuid::now_v7();
        sqlx::query("INSERT INTO developers (id, email, password_hash, name) VALUES ($1,$2,$3,$4)")
            .bind(dev_id)
            .bind(format!("sdk_api_{}@test.com", dev_id))
            .bind("$argon2id$hash")
            .bind("SDK API Dev")
            .execute(pool)
            .await
            .expect("insert developer");

        let dto = crate::model::project::CreateProject {
            name: "SDK Test Project".to_string(),
            description: None,
        };
        let proj = crate::service::project::create(pool, dev_id, dto).await.expect("create project");
        (dev_id, proj.api_key)
    }

    async fn cleanup(pool: &sqlx::PgPool, dev_id: uuid::Uuid) {
        sqlx::query("DELETE FROM conversations WHERE project_id IN (SELECT id FROM projects WHERE developer_id = $1)")
            .bind(dev_id).execute(pool).await.ok();
        sqlx::query("DELETE FROM end_users WHERE project_id IN (SELECT id FROM projects WHERE developer_id = $1)")
            .bind(dev_id).execute(pool).await.ok();
        sqlx::query("DELETE FROM projects WHERE developer_id = $1").bind(dev_id).execute(pool).await.ok();
        sqlx::query("DELETE FROM developers WHERE id = $1").bind(dev_id).execute(pool).await.ok();
    }

    #[tokio::test]
    async fn sdk_init_without_api_key_returns_401() {
        let (app, _) = sdk_test_app().await;
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/sdk/init")
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"device_id":"dev-1"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn sdk_init_with_invalid_api_key_returns_401() {
        let (app, _) = sdk_test_app().await;
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/sdk/init")
                    .header("Content-Type", "application/json")
                    .header("X-API-Key", "proj_invalid_key")
                    .body(Body::from(r#"{"device_id":"dev-1"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn sdk_init_creates_end_user_and_conversation() {
        let (app, pool) = sdk_test_app().await;
        let (dev_id, api_key) = create_dev_and_project(&pool).await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/sdk/init")
                    .header("Content-Type", "application/json")
                    .header("X-API-Key", &api_key)
                    .body(Body::from(r#"{"device_id":"test-device-x","name":"Alice"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(json["end_user"]["device_id"], "test-device-x");
        assert_eq!(json["end_user"]["name"], "Alice");
        assert_eq!(json["conversation"]["status"], "open");
        assert!(json["end_user"]["id"].is_string());
        assert!(json["conversation"]["id"].is_string());

        cleanup(&pool, dev_id).await;
    }

    #[tokio::test]
    async fn sdk_init_idempotent_returns_same_ids() {
        let (app, pool) = sdk_test_app().await;
        let (dev_id, api_key) = create_dev_and_project(&pool).await;

        let body = r#"{"device_id":"test-device-y"}"#;

        let make_request = || {
            Request::builder()
                .method("POST")
                .uri("/api/v1/sdk/init")
                .header("Content-Type", "application/json")
                .header("X-API-Key", api_key.clone())
                .body(Body::from(body))
                .unwrap()
        };

        let r1 = crate::api::create_router(pool.clone(), test_jwt())
            .oneshot(make_request())
            .await
            .unwrap();
        let r2 = crate::api::create_router(pool.clone(), test_jwt())
            .oneshot(make_request())
            .await
            .unwrap();

        let b1: serde_json::Value =
            serde_json::from_slice(&r1.into_body().collect().await.unwrap().to_bytes()).unwrap();
        let b2: serde_json::Value =
            serde_json::from_slice(&r2.into_body().collect().await.unwrap().to_bytes()).unwrap();

        assert_eq!(b1["end_user"]["id"], b2["end_user"]["id"], "end_user id should be same");
        assert_eq!(
            b1["conversation"]["id"], b2["conversation"]["id"],
            "conversation id should be same"
        );

        cleanup(&pool, dev_id).await;
    }

    #[tokio::test]
    async fn sdk_init_empty_device_id_returns_400() {
        let (_app, pool) = sdk_test_app().await;
        let (dev_id, api_key) = create_dev_and_project(&pool).await;

        let response = crate::api::create_router(pool.clone(), test_jwt())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/sdk/init")
                    .header("Content-Type", "application/json")
                    .header("X-API-Key", &api_key)
                    .body(Body::from(r#"{"device_id":"  "}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        cleanup(&pool, dev_id).await;
    }
}
