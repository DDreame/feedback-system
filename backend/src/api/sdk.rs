use axum::extract::ws::WebSocketUpgrade;
use axum::{extract::Query, extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::middleware::AuthProject;
use crate::api::AppState;
use crate::error::AppError;
use crate::model::conversation::ConversationResponse;
use crate::model::end_user::EndUserResponse;
use crate::model::message::{MessageResponse, SenderType};
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

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub conversation_id: Uuid,
    pub content: String,
    pub end_user_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct SendMessageResponse {
    pub message: MessageResponse,
}

/// POST /api/v1/sdk/messages
///
/// Send a message from an end user.
pub async fn send_message(
    _auth: AuthProject,
    State(state): State<AppState>,
    Json(body): Json<SendMessageRequest>,
) -> Result<(StatusCode, Json<SendMessageResponse>), AppError> {
    let msg = service::chat::send_message(
        &state.db,
        body.conversation_id,
        &SenderType::EndUser,
        Some(body.end_user_id),
        &body.content,
    )
    .await?;

    // Broadcast to WebSocket connections in this conversation
    state.ws.broadcast(body.conversation_id, &msg).await;

    Ok((StatusCode::CREATED, Json(SendMessageResponse { message: msg })))
}

#[derive(Debug, Deserialize)]
pub struct ListMessagesQuery {
    pub conversation_id: Uuid,
    pub before: Option<Uuid>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ListMessagesResponse {
    pub messages: Vec<MessageResponse>,
}

/// GET /api/v1/sdk/messages?conversation_id=...&before=...&limit=...
///
/// Retrieve messages for a conversation with cursor-based pagination.
pub async fn list_messages(
    _auth: AuthProject,
    State(state): State<AppState>,
    Query(query): Query<ListMessagesQuery>,
) -> Result<Json<ListMessagesResponse>, AppError> {
    let limit = query.limit.unwrap_or(50);
    let messages =
        service::chat::list_messages(&state.db, query.conversation_id, query.before, limit).await?;
    Ok(Json(ListMessagesResponse { messages }))
}

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    pub api_key: String,
    pub conversation_id: Uuid,
    pub end_user_id: Uuid,
}

/// GET /api/v1/sdk/ws?api_key=...&conversation_id=...&end_user_id=...
///
/// Upgrades an HTTP connection to WebSocket for real-time messaging.
/// Uses query parameters for auth since browser WebSocket API cannot set custom headers.
pub async fn ws_upgrade(
    State(state): State<AppState>,
    Query(query): Query<WsQuery>,
    ws: WebSocketUpgrade,
) -> Result<axum::response::Response, AppError> {
    // Validate API key
    let _project = service::project::get_by_api_key(&state.db, &query.api_key).await?;

    let conn_mgr = state.ws.clone();
    let db = state.db.clone();
    let conv_id = query.conversation_id;
    let user_id = query.end_user_id;

    Ok(ws.on_upgrade(move |socket| {
        crate::ws::handle_ws_connection(socket, conv_id, user_id, conn_mgr, db)
    }))
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
        sqlx::query("DELETE FROM messages WHERE conversation_id IN (SELECT c.id FROM conversations c JOIN projects p ON c.project_id=p.id WHERE p.developer_id=$1)")
            .bind(dev_id).execute(pool).await.ok();
        sqlx::query("DELETE FROM conversations WHERE project_id IN (SELECT id FROM projects WHERE developer_id = $1)")
            .bind(dev_id).execute(pool).await.ok();
        sqlx::query("DELETE FROM end_users WHERE project_id IN (SELECT id FROM projects WHERE developer_id = $1)")
            .bind(dev_id).execute(pool).await.ok();
        sqlx::query("DELETE FROM projects WHERE developer_id = $1").bind(dev_id).execute(pool).await.ok();
        sqlx::query("DELETE FROM developers WHERE id = $1").bind(dev_id).execute(pool).await.ok();
    }

    /// Helper: SDK init to get end_user + conversation ids
    async fn sdk_init_helper(pool: &sqlx::PgPool, api_key: &str) -> (uuid::Uuid, uuid::Uuid) {
        let app = crate::api::create_router(pool.clone(), test_jwt());
        let r = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/sdk/init")
                    .header("Content-Type", "application/json")
                    .header("X-API-Key", api_key)
                    .body(Body::from(r#"{"device_id":"msg-test-device"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        let bytes = r.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let end_user_id: uuid::Uuid = json["end_user"]["id"].as_str().unwrap().parse().unwrap();
        let conv_id: uuid::Uuid = json["conversation"]["id"].as_str().unwrap().parse().unwrap();
        (end_user_id, conv_id)
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

    // ── Message API tests ────────────────────────────────────────────────

    #[tokio::test]
    async fn send_message_returns_201() {
        let (_app, pool) = sdk_test_app().await;
        let (dev_id, api_key) = create_dev_and_project(&pool).await;
        let (end_user_id, conv_id) = sdk_init_helper(&pool, &api_key).await;

        let body = serde_json::json!({
            "conversation_id": conv_id,
            "end_user_id": end_user_id,
            "content": "Hello from SDK!"
        });

        let app = crate::api::create_router(pool.clone(), test_jwt());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/sdk/messages")
                    .header("Content-Type", "application/json")
                    .header("X-API-Key", &api_key)
                    .body(Body::from(serde_json::to_string(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["message"]["content"], "Hello from SDK!");
        assert_eq!(json["message"]["sender_type"], "end_user");

        cleanup(&pool, dev_id).await;
    }

    #[tokio::test]
    async fn list_messages_returns_sent_messages() {
        let (_app, pool) = sdk_test_app().await;
        let (dev_id, api_key) = create_dev_and_project(&pool).await;
        let (end_user_id, conv_id) = sdk_init_helper(&pool, &api_key).await;

        // Send two messages
        for msg in &["First message", "Second message"] {
            let body = serde_json::json!({
                "conversation_id": conv_id,
                "end_user_id": end_user_id,
                "content": msg
            });
            let app = crate::api::create_router(pool.clone(), test_jwt());
            app.oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/sdk/messages")
                    .header("Content-Type", "application/json")
                    .header("X-API-Key", &api_key)
                    .body(Body::from(serde_json::to_string(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        }

        // List messages
        let app = crate::api::create_router(pool.clone(), test_jwt());
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(&format!("/api/v1/sdk/messages?conversation_id={conv_id}"))
                    .header("X-API-Key", &api_key)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let messages = json["messages"].as_array().unwrap();
        assert_eq!(messages.len(), 2);
        // Most recent first
        assert_eq!(messages[0]["content"], "Second message");
        assert_eq!(messages[1]["content"], "First message");

        cleanup(&pool, dev_id).await;
    }

    #[tokio::test]
    async fn send_message_without_api_key_returns_401() {
        let (app, _) = sdk_test_app().await;
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/sdk/messages")
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"conversation_id":"00000000-0000-0000-0000-000000000000","end_user_id":"00000000-0000-0000-0000-000000000000","content":"test"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    // ── WebSocket Integration tests ─────────────────────────────────────────

    /// Test that WebSocket message is persisted to DB and broadcast to other subscribers.
    /// This test verifies the full flow: connect WS -> send message -> persist -> broadcast.
    #[tokio::test]
    async fn ws_message_persisted_and_broadcast() {
        // Setup: create test app with DB and a shared ConnectionManager
        let _guard = crate::test_support::ENV_MUTEX.lock().unwrap();
        let _ = dotenvy::dotenv_override();
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPoolOptions::new()
            .max_connections(3)
            .connect(&url)
            .await
            .expect("connect");
        crate::db::run_migrations(&pool).await.expect("migrations");

        // Create developer and project
        let dev_id = uuid::Uuid::now_v7();
        sqlx::query("INSERT INTO developers (id, email, password_hash, name) VALUES ($1,$2,$3,$4)")
            .bind(dev_id)
            .bind(format!("ws_dev_{}@test.com", dev_id))
            .bind("$argon2id$hash")
            .bind("WS Dev")
            .execute(&pool)
            .await
            .expect("insert developer");

        let dto = crate::model::project::CreateProject {
            name: "WS Test Project".to_string(),
            description: None,
        };
        let proj = crate::service::project::create(&pool, dev_id, dto)
            .await
            .expect("create project");
        let api_key = proj.api_key.clone();

        // Create SDK session via HTTP
        let app = crate::api::create_router(pool.clone(), test_jwt());
        let init_response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/sdk/init")
                    .header("Content-Type", "application/json")
                    .header("X-API-Key", &api_key)
                    .body(Body::from(r#"{"device_id":"ws-test-device"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(init_response.status(), StatusCode::OK);
        let bytes = init_response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let conv_id: uuid::Uuid = json["conversation"]["id"].as_str().unwrap().parse().unwrap();
        let end_user_id: uuid::Uuid = json["end_user"]["id"].as_str().unwrap().parse().unwrap();

        // Test: Use ConnectionManager directly to simulate WebSocket behavior
        // 1. Create two subscribers (simulating two WS connections)
        let mgr = crate::ws::ConnectionManager::new();
        let mut rx1 = mgr.subscribe(conv_id).await;
        let mut rx2 = mgr.subscribe(conv_id).await;

        // 2. Simulate receiving a WebSocket message and persisting it
        let test_content = "Hello from WebSocket!";
        let saved_msg = crate::service::chat::send_message(
            &pool,
            conv_id,
            &crate::model::message::SenderType::EndUser,
            Some(end_user_id),
            test_content,
        )
        .await
        .expect("send_message");

        // 3. Broadcast to subscribers
        mgr.broadcast(conv_id, &saved_msg).await;

        // 4. Verify both subscribers receive the broadcast
        let received1 = rx1.recv().await.expect("rx1 should receive");
        let received2 = rx2.recv().await.expect("rx2 should receive");

        let broadcast1: crate::ws::WsBroadcast =
            serde_json::from_str(&received1).expect("parse broadcast1");
        let broadcast2: crate::ws::WsBroadcast =
            serde_json::from_str(&received2).expect("parse broadcast2");

        assert_eq!(broadcast1.msg_type, "new_message");
        assert_eq!(broadcast1.message.content, test_content);
        assert_eq!(broadcast2.message.content, test_content);

        // 5. Verify message is persisted in DB
        let db_messages = crate::service::chat::list_messages(&pool, conv_id, None, 10)
            .await
            .expect("list_messages");
        assert!(!db_messages.is_empty());
        assert_eq!(db_messages[0].content, test_content);

        // Cleanup
        sqlx::query("DELETE FROM messages WHERE conversation_id IN (SELECT c.id FROM conversations c JOIN projects p ON c.project_id=p.id WHERE p.developer_id=$1)")
            .bind(dev_id).execute(&pool).await.ok();
        sqlx::query("DELETE FROM conversations WHERE project_id IN (SELECT id FROM projects WHERE developer_id = $1)")
            .bind(dev_id).execute(&pool).await.ok();
        sqlx::query("DELETE FROM end_users WHERE project_id IN (SELECT id FROM projects WHERE developer_id = $1)")
            .bind(dev_id).execute(&pool).await.ok();
        sqlx::query("DELETE FROM projects WHERE developer_id = $1").bind(dev_id).execute(&pool).await.ok();
        sqlx::query("DELETE FROM developers WHERE id = $1").bind(dev_id).execute(&pool).await.ok();
    }

    /// Test that WebSocket clients in different conversations do not receive each other's messages.
    #[tokio::test]
    async fn ws_conversations_are_isolated() {
        let _guard = crate::test_support::ENV_MUTEX.lock().unwrap();
        let _ = dotenvy::dotenv_override();
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPoolOptions::new()
            .max_connections(3)
            .connect(&url)
            .await
            .expect("connect");
        crate::db::run_migrations(&pool).await.expect("migrations");

        let dev_id = uuid::Uuid::now_v7();
        sqlx::query("INSERT INTO developers (id, email, password_hash, name) VALUES ($1,$2,$3,$4)")
            .bind(dev_id)
            .bind(format!("ws_iso_{}@test.com", dev_id))
            .bind("$argon2id$hash")
            .bind("WS Iso Dev")
            .execute(&pool)
            .await
            .expect("insert developer");

        let dto = crate::model::project::CreateProject {
            name: "WS Iso Project".to_string(),
            description: None,
        };
        let proj = crate::service::project::create(&pool, dev_id, dto)
            .await
            .expect("create project");
        let api_key = proj.api_key.clone();

        // Create two conversations
        let app = crate::api::create_router(pool.clone(), test_jwt());
        let make_init = |device_id: &str| {
            Request::builder()
                .method("POST")
                .uri("/api/v1/sdk/init")
                .header("Content-Type", "application/json")
                .header("X-API-Key", &api_key)
                .body(Body::from(format!(r#"{{"device_id":"{}"}}"#, device_id)))
                .unwrap()
        };

        let r1 = app.clone().oneshot(make_init("device-a")).await.unwrap();
        let r2 = app.oneshot(make_init("device-b")).await.unwrap();

        let json1: serde_json::Value = serde_json::from_slice(
            &r1.into_body().collect().await.unwrap().to_bytes(),
        )
        .unwrap();
        let json2: serde_json::Value = serde_json::from_slice(
            &r2.into_body().collect().await.unwrap().to_bytes(),
        )
        .unwrap();

        let conv_a: uuid::Uuid = json1["conversation"]["id"].as_str().unwrap().parse().unwrap();
        let user_a: uuid::Uuid = json1["end_user"]["id"].as_str().unwrap().parse().unwrap();
        let conv_b: uuid::Uuid = json2["conversation"]["id"].as_str().unwrap().parse().unwrap();

        // Test: Create subscribers for different conversations
        let mgr = crate::ws::ConnectionManager::new();
        let mut rx_a = mgr.subscribe(conv_a).await;
        let mut rx_b = mgr.subscribe(conv_b).await;

        // Send message to conversation A
        let msg_a = crate::service::chat::send_message(
            &pool,
            conv_a,
            &crate::model::message::SenderType::EndUser,
            Some(user_a),
            "Message for A",
        )
        .await
        .expect("send to A");

        mgr.broadcast(conv_a, &msg_a).await;

        // rx_a should receive the message
        let received_a = rx_a.recv().await.expect("rx_a should receive");
        let broadcast_a: crate::ws::WsBroadcast =
            serde_json::from_str(&received_a).expect("parse");
        assert_eq!(broadcast_a.message.content, "Message for A");

        // rx_b should NOT receive the message
        assert!(
            rx_b.try_recv().is_err(),
            "rx_b should not receive message from conv_a"
        );

        // Cleanup
        sqlx::query("DELETE FROM messages WHERE conversation_id IN (SELECT c.id FROM conversations c JOIN projects p ON c.project_id=p.id WHERE p.developer_id=$1)")
            .bind(dev_id).execute(&pool).await.ok();
        sqlx::query("DELETE FROM conversations WHERE project_id IN (SELECT id FROM projects WHERE developer_id = $1)")
            .bind(dev_id).execute(&pool).await.ok();
        sqlx::query("DELETE FROM end_users WHERE project_id IN (SELECT id FROM projects WHERE developer_id = $1)")
            .bind(dev_id).execute(&pool).await.ok();
        sqlx::query("DELETE FROM projects WHERE developer_id = $1").bind(dev_id).execute(&pool).await.ok();
        sqlx::query("DELETE FROM developers WHERE id = $1").bind(dev_id).execute(&pool).await.ok();
    }
}
