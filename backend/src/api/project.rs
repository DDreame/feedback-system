use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::api::{middleware::AuthDeveloper, AppState};
use crate::error::AppError;
use crate::model::project::CreateProject;
use crate::service;

pub async fn create(
    State(state): State<AppState>,
    auth: AuthDeveloper,
    Json(dto): Json<CreateProject>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let project = service::project::create(&state.db, auth.developer_id, dto).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "project": project }))))
}

pub async fn list(
    State(state): State<AppState>,
    auth: AuthDeveloper,
) -> Result<Json<serde_json::Value>, AppError> {
    let projects = service::project::list(&state.db, auth.developer_id).await?;
    Ok(Json(serde_json::json!({ "projects": projects })))
}

pub async fn get(
    State(state): State<AppState>,
    auth: AuthDeveloper,
    Path(project_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let project = service::project::get(&state.db, project_id, auth.developer_id).await?;
    Ok(Json(serde_json::json!({ "project": project })))
}

#[derive(Deserialize)]
pub struct UpdateProject {
    pub name: Option<String>,
    pub description: Option<String>,
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthDeveloper,
    Path(project_id): Path<Uuid>,
    Json(body): Json<UpdateProject>,
) -> Result<Json<serde_json::Value>, AppError> {
    let project =
        service::project::update(&state.db, project_id, auth.developer_id, body.name, body.description).await?;
    Ok(Json(serde_json::json!({ "project": project })))
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthDeveloper,
    Path(project_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    service::project::delete(&state.db, project_id, auth.developer_id).await?;
    Ok(StatusCode::NO_CONTENT)
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
    use crate::utils::jwt::{generate_token, TokenKind};

    fn test_jwt() -> crate::config::JwtConfig {
        crate::config::JwtConfig {
            secret: "test-secret-at-least-32-chars!!".to_string(),
            access_token_expiry_secs: 3600,
            refresh_token_expiry_secs: 604800,
        }
    }

    async fn real_router() -> axum::Router {
        let _guard = crate::test_support::ENV_MUTEX.lock().unwrap();
        let _ = dotenvy::dotenv_override();
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPoolOptions::new().max_connections(2).connect(&url).await.expect("connect");
        crate::db::run_migrations(&pool).await.expect("migrations");
        create_router(pool, test_jwt())
    }

    /// Register a developer and return (dev_id, access_token)
    async fn register_dev(app: &axum::Router, email: &str) -> (uuid::Uuid, String) {
        let body = serde_json::json!({ "email": email, "password": "password123", "name": "Dev" });
        let resp = app.clone()
            .oneshot(Request::builder().method("POST").uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string())).unwrap())
            .await.unwrap();

        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let dev_id: uuid::Uuid = json["developer"]["id"].as_str().unwrap().parse().unwrap();
        let token = generate_token(dev_id, &test_jwt().secret, TokenKind::Access, 3600).unwrap();
        (dev_id, token)
    }

    fn bearer(token: &str) -> String {
        format!("Bearer {token}")
    }

    #[tokio::test]
    async fn create_project_no_auth_returns_401() {
        let app = real_router().await;
        let body = serde_json::json!({ "name": "App" });
        let resp = app.oneshot(
            Request::builder().method("POST").uri("/api/v1/projects")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string())).unwrap()
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn create_and_list_projects() {
        let app = real_router().await;
        let email = format!("api_proj_{}@test.com", uuid::Uuid::now_v7());
        let (_, token) = register_dev(&app, &email).await;

        let body = serde_json::json!({ "name": "My App", "description": "Cool" });
        let resp = app.clone().oneshot(
            Request::builder().method("POST").uri("/api/v1/projects")
                .header("content-type", "application/json")
                .header("Authorization", bearer(&token))
                .body(Body::from(body.to_string())).unwrap()
        ).await.unwrap();

        assert_eq!(resp.status(), StatusCode::CREATED);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["project"]["name"], "My App");
        assert!(json["project"]["api_key"].as_str().unwrap().starts_with("proj_"));

        // List
        let resp = app.oneshot(
            Request::builder().uri("/api/v1/projects")
                .header("Authorization", bearer(&token))
                .body(Body::empty()).unwrap()
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(json["projects"].as_array().unwrap().len() >= 1);
    }

    #[tokio::test]
    async fn get_update_delete_project() {
        let app = real_router().await;
        let email = format!("api_proj2_{}@test.com", uuid::Uuid::now_v7());
        let (_, token) = register_dev(&app, &email).await;

        // Create
        let body = serde_json::json!({ "name": "ToCRUD" });
        let resp = app.clone().oneshot(
            Request::builder().method("POST").uri("/api/v1/projects")
                .header("content-type", "application/json")
                .header("Authorization", bearer(&token))
                .body(Body::from(body.to_string())).unwrap()
        ).await.unwrap();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let proj_id = serde_json::from_slice::<serde_json::Value>(&bytes).unwrap()["project"]["id"]
            .as_str().unwrap().to_string();

        // Get
        let resp = app.clone().oneshot(
            Request::builder().uri(format!("/api/v1/projects/{proj_id}"))
                .header("Authorization", bearer(&token))
                .body(Body::empty()).unwrap()
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Update
        let upd = serde_json::json!({ "name": "Updated" });
        let resp = app.clone().oneshot(
            Request::builder().method("PATCH").uri(format!("/api/v1/projects/{proj_id}"))
                .header("content-type", "application/json")
                .header("Authorization", bearer(&token))
                .body(Body::from(upd.to_string())).unwrap()
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["project"]["name"], "Updated");

        // Delete
        let resp = app.oneshot(
            Request::builder().method("DELETE").uri(format!("/api/v1/projects/{proj_id}"))
                .header("Authorization", bearer(&token))
                .body(Body::empty()).unwrap()
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn get_other_developers_project_returns_404() {
        let app = real_router().await;
        let email1 = format!("api_own1_{}@test.com", uuid::Uuid::now_v7());
        let email2 = format!("api_own2_{}@test.com", uuid::Uuid::now_v7());
        let (_, token1) = register_dev(&app, &email1).await;
        let (_, token2) = register_dev(&app, &email2).await;

        let body = serde_json::json!({ "name": "Private" });
        let resp = app.clone().oneshot(
            Request::builder().method("POST").uri("/api/v1/projects")
                .header("content-type", "application/json")
                .header("Authorization", bearer(&token1))
                .body(Body::from(body.to_string())).unwrap()
        ).await.unwrap();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let proj_id = serde_json::from_slice::<serde_json::Value>(&bytes).unwrap()["project"]["id"]
            .as_str().unwrap().to_string();

        let resp = app.oneshot(
            Request::builder().uri(format!("/api/v1/projects/{proj_id}"))
                .header("Authorization", bearer(&token2))
                .body(Body::empty()).unwrap()
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
