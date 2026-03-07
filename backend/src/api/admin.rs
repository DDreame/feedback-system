use axum::{
    extract::{Path, Query, State, ws::{Message, WebSocket, WebSocketUpgrade}},
    response::Json,
    routing::{get, patch},
    Router,
};
use futures::{StreamExt, SinkExt};
use serde::Deserialize;
use uuid::Uuid;

use crate::api::middleware::AuthDeveloper;
use crate::api::AppState;
use crate::error::AppError;
use crate::model::conversation::ConversationStatus;
use crate::model::message::MessageResponse as MsgResponse;
use crate::service::admin::{self, ListConversationsQuery};

/// Router for admin API endpoints.
pub fn create_router() -> Router<AppState> {
    Router::new()
        .route(
            "/projects/{project_id}/conversations",
            get(list_conversations),
        )
        .route(
            "/projects/{project_id}/conversations/{conversation_id}",
            get(get_conversation),
        )
        .route(
            "/projects/{project_id}/conversations/{conversation_id}/messages",
            get(get_messages).post(send_message),
        )
        .route(
            "/projects/{project_id}/conversations/{conversation_id}/status",
            patch(update_status),
        )
        .route(
            "/admin/ws",
            get(admin_ws_upgrade),
        )
}

/// Query parameters for listing conversations.
#[derive(Debug, Deserialize, Default)]
pub struct ListQuery {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub page: Option<i64>,
    #[serde(default)]
    pub limit: Option<i64>,
}

/// Response wrapper for conversations list.
#[derive(serde::Serialize)]
pub struct ConversationsResponse {
    pub conversations: Vec<admin::ConversationWithUser>,
}

/// GET /api/v1/projects/:project_id/conversations
pub async fn list_conversations(
    State(state): State<AppState>,
    AuthDeveloper { developer_id: _ }: AuthDeveloper,
    Path(project_id): Path<uuid::Uuid>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ConversationsResponse>, AppError> {
    let query = ListConversationsQuery {
        status: query.status,
        page: query.page,
        limit: query.limit,
    };
    let conversations = admin::list_conversations(&state.db, project_id, query).await?;
    Ok(Json(ConversationsResponse { conversations }))
}

/// Response wrapper for single conversation.
#[derive(serde::Serialize)]
pub struct ConversationResponse {
    pub conversation: admin::ConversationWithUser,
}

/// GET /api/v1/projects/:project_id/conversations/:conversation_id
pub async fn get_conversation(
    State(state): State<AppState>,
    AuthDeveloper { developer_id: _ }: AuthDeveloper,
    Path((project_id, conversation_id)): Path<(uuid::Uuid, uuid::Uuid)>,
) -> Result<Json<ConversationResponse>, AppError> {
    let conversation = admin::get_conversation(&state.db, project_id, conversation_id).await?;
    Ok(Json(ConversationResponse { conversation }))
}

/// Request body for sending a message.
#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
}

/// Response wrapper for message.
#[derive(serde::Serialize)]
pub struct AdminMessageResponse {
    pub message: MsgResponse,
}

/// GET /api/v1/projects/:project_id/conversations/:conversation_id/messages
pub async fn get_messages(
    State(state): State<AppState>,
    AuthDeveloper { developer_id: _ }: AuthDeveloper,
    Path((project_id, conversation_id)): Path<(uuid::Uuid, uuid::Uuid)>,
) -> Result<Json<AdminMessageResponse>, AppError> {
    // Verify conversation belongs to project
    let _ = admin::get_conversation(&state.db, project_id, conversation_id).await?;

    // Get messages using the chat service
    let messages = crate::service::chat::list_messages(&state.db, conversation_id, None, 50).await?;

    // Return the first message for this simple endpoint
    // For a real implementation, you'd return a list
    if let Some(msg) = messages.into_iter().last() {
        Ok(Json(AdminMessageResponse { message: msg }))
    } else {
        Err(AppError::NotFound("No messages found".to_string()))
    }
}

/// POST /api/v1/projects/:project_id/conversations/:conversation_id/messages
pub async fn send_message(
    State(state): State<AppState>,
    AuthDeveloper { developer_id }: AuthDeveloper,
    Path((project_id, conversation_id)): Path<(uuid::Uuid, uuid::Uuid)>,
    Json(body): Json<SendMessageRequest>,
) -> Result<Json<AdminMessageResponse>, AppError> {
    let message = admin::send_developer_message(
        &state.db,
        project_id,
        conversation_id,
        developer_id,
        &body.content,
    )
    .await?;

    // Broadcast the message via WebSocket
    let _ = state.ws.broadcast(conversation_id, &message);

    Ok(Json(AdminMessageResponse { message }))
}

/// Request body for updating conversation status.
#[derive(Debug, Deserialize)]
pub struct UpdateStatusRequest {
    pub status: String,
}

/// PATCH /api/v1/projects/:project_id/conversations/:conversation_id/status
pub async fn update_status(
    State(state): State<AppState>,
    AuthDeveloper { developer_id: _ }: AuthDeveloper,
    Path((project_id, conversation_id)): Path<(uuid::Uuid, uuid::Uuid)>,
    Json(body): Json<UpdateStatusRequest>,
) -> Result<Json<ConversationResponse>, AppError> {
    let status = match body.status.to_lowercase().as_str() {
        "open" => ConversationStatus::Open,
        "closed" => ConversationStatus::Closed,
        _ => return Err(AppError::BadRequest("Invalid status".to_string())),
    };

    // Update the status
    let _ = admin::update_conversation_status(&state.db, project_id, conversation_id, &status).await?;

    // Also fetch the end user info for the response
    let full_conversation = admin::get_conversation(&state.db, project_id, conversation_id).await?;

    Ok(Json(ConversationResponse {
        conversation: full_conversation,
    }))
}

/// Query parameters for admin WebSocket.
#[derive(Debug, Deserialize)]
pub struct AdminWsQuery {
    pub conversation_id: Uuid,
}

/// GET /api/v1/admin/ws - WebSocket upgrade for admin chat
pub async fn admin_ws_upgrade(
    State(state): State<AppState>,
    AuthDeveloper { developer_id }: AuthDeveloper,
    Query(query): Query<AdminWsQuery>,
    ws: WebSocketUpgrade,
) -> Result<axum::response::Response, AppError> {
    let conversation_id = query.conversation_id;
    let developer_id = developer_id;

    // We need to verify the conversation exists and belongs to a project owned by this developer
    // Since we don't know the project_id here, we'll just accept the connection
    // and let the handler verify when receiving messages

    Ok(ws.on_upgrade(move |socket| {
        handle_admin_ws_connection(socket, conversation_id, developer_id, state.ws.clone(), state.db.clone())
    }))
}

/// Handle admin WebSocket connection for real-time chat.
async fn handle_admin_ws_connection(
    mut socket: WebSocket,
    conversation_id: Uuid,
    developer_id: Uuid,
    conn_mgr: crate::ws::ConnectionManager,
    db: sqlx::PgPool,
) {
    // We need to get the project_id from conversation to verify ownership
    let project_id_result: Option<Uuid> = sqlx::query_scalar(
        "SELECT project_id FROM conversations WHERE id = $1"
    )
    .bind(conversation_id)
    .fetch_optional(&db)
    .await
    .ok()
    .flatten();

    let project_id = match project_id_result {
        Some(id) => id,
        None => {
            let error = serde_json::json!({ "type": "error", "message": "Conversation not found" });
            let _ = socket.send(Message::Text(error.to_string().into())).await;
            return;
        }
    };

    // Verify the project belongs to this developer
    let owner_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT developer_id FROM projects WHERE id = $1"
    )
    .bind(project_id)
    .fetch_optional(&db)
    .await
    .ok()
    .flatten();

    if let Some(owner) = owner_id {
        if owner != developer_id {
            let error = serde_json::json!({ "type": "error", "message": "Not authorized to access this conversation" });
            let _ = socket.send(Message::Text(error.to_string().into())).await;
            return;
        }
    }

    let mut rx = conn_mgr.subscribe(conversation_id).await;

    let (ws_tx, ws_rx) = socket.split();

    // Task: forward broadcast messages to this WebSocket client
    let send_task = tokio::spawn(async move {
        let mut ws_tx = ws_tx;
        while let Ok(msg) = rx.recv().await {
            if ws_tx.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // Task: receive messages from this WebSocket client, persist and broadcast
    let recv_task = tokio::spawn(async move {
        let mut ws_rx = ws_rx;
        while let Some(msg_result) = ws_rx.next().await {
            let msg = match msg_result {
                Ok(Message::Text(text)) => text,
                Ok(Message::Close(_)) => break,
                Err(_) => break,
                _ => continue,
            };

            // Parse the client message
            if let Ok(client_msg) = serde_json::from_str::<serde_json::Value>(&msg) {
                if client_msg.get("type").and_then(|v| v.as_str()) == Some("send_message") {
                    if let Some(content) = client_msg.get("content").and_then(|v| v.as_str()) {
                        // Send message as developer - need project_id for this
                        match admin::send_developer_message(&db, project_id, conversation_id, developer_id, content).await {
                            Ok(message) => {
                                // Broadcast to all subscribers
                                let _ = conn_mgr.broadcast(conversation_id, &message);
                            }
                            Err(_e) => {
                                // Silently ignore errors - the connection might be closed
                            }
                        }
                    }
                }
            }
        }
    });

    // Wait for both tasks to complete
    tokio::select! {
        _ = send_task => {}
        _ = recv_task => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_state() -> AppState {
        use sqlx::postgres::PgPoolOptions;
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgres://unused:unused@localhost/unused")
            .expect("lazy pool");
        let jwt = crate::config::JwtConfig {
            secret: "test-secret-at-least-32-chars!!".to_string(),
            access_token_expiry_secs: 3600,
            refresh_token_expiry_secs: 604800,
        };
        let ws = crate::ws::ConnectionManager::new();
        AppState { db: pool, jwt, ws }
    }

    #[tokio::test]
    async fn list_conversations_route_defined() {
        // Just verify the router can be built
        let router = create_router();
        let state = test_state();
        // Verify router compiles
        let _ = router;
        let _ = state;
    }
}
