use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::model::conversation::{Conversation, ConversationResponse, ConversationStatus};
use crate::model::end_user::EndUserResponse;
use crate::model::message::{Message, MessageResponse, SenderType};

/// Conversation with end user info for admin UI.
#[derive(Debug, Clone, Serialize)]
pub struct ConversationWithUser {
    #[serde(flatten)]
    pub conversation: ConversationResponse,
    pub end_user: EndUserResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_message: Option<MessageResponse>,
}

/// Input for listing conversations.
#[derive(Debug, Deserialize, Default)]
pub struct ListConversationsQuery {
    pub status: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

/// List conversations for a project, with optional status filter and pagination.
pub async fn list_conversations(
    pool: &PgPool,
    project_id: Uuid,
    query: ListConversationsQuery,
) -> Result<Vec<ConversationWithUser>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).min(100).max(1);
    let offset = (page - 1) * limit;

    // Build query based on status filter
    let conversations: Vec<Conversation> = if let Some(status) = &query.status {
        let status_val = match status.to_lowercase().as_str() {
            "open" => "open",
            "closed" => "closed",
            _ => return Err(AppError::BadRequest("Invalid status filter".to_string())),
        };
        sqlx::query_as(
            "SELECT id, project_id, end_user_id, status, created_at, updated_at
             FROM conversations
             WHERE project_id = $1 AND status = $2
             ORDER BY updated_at DESC
             LIMIT $3 OFFSET $4",
        )
        .bind(project_id)
        .bind(status_val)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    } else {
        sqlx::query_as(
            "SELECT id, project_id, end_user_id, status, created_at, updated_at
             FROM conversations
             WHERE project_id = $1
             ORDER BY updated_at DESC
             LIMIT $2 OFFSET $3",
        )
        .bind(project_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    };

    // Fetch end users and last messages for each conversation
    let mut results = Vec::with_capacity(conversations.len());
    for conv in conversations {
        // Get end user
        let end_user: Option<crate::model::end_user::EndUser> = sqlx::query_as(
            "SELECT id, project_id, device_id, name, created_at, updated_at
             FROM end_users WHERE id = $1",
        )
        .bind(conv.end_user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        let end_user = end_user.ok_or_else(|| {
            AppError::Internal("Conversation has invalid end_user_id".to_string())
        })?;

        // Get last message
        let last_message: Option<Message> = sqlx::query_as(
            "SELECT id, conversation_id, sender_type, sender_id, message_type, content, created_at
             FROM messages
             WHERE conversation_id = $1
             ORDER BY created_at DESC
             LIMIT 1",
        )
        .bind(conv.id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        let last_message = last_message.map(MessageResponse::from);

        results.push(ConversationWithUser {
            conversation: ConversationResponse::from(conv),
            end_user: EndUserResponse::from(end_user),
            last_message,
        });
    }

    Ok(results)
}

/// Get a single conversation by ID, verifying it belongs to the project.
pub async fn get_conversation(
    pool: &PgPool,
    project_id: Uuid,
    conversation_id: Uuid,
) -> Result<ConversationWithUser, AppError> {
    let conv: Option<Conversation> = sqlx::query_as(
        "SELECT id, project_id, end_user_id, status, created_at, updated_at
         FROM conversations
         WHERE id = $1 AND project_id = $2",
    )
    .bind(conversation_id)
    .bind(project_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let conv = conv.ok_or(AppError::NotFound("Conversation not found".to_string()))?;

    // Get end user
    let end_user: crate::model::end_user::EndUser = sqlx::query_as(
        "SELECT id, project_id, device_id, name, created_at, updated_at
         FROM end_users WHERE id = $1",
    )
    .bind(conv.end_user_id)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(ConversationWithUser {
        conversation: ConversationResponse::from(conv),
        end_user: EndUserResponse::from(end_user),
        last_message: None,
    })
}

/// Update conversation status.
pub async fn update_conversation_status(
    pool: &PgPool,
    project_id: Uuid,
    conversation_id: Uuid,
    status: &ConversationStatus,
) -> Result<ConversationResponse, AppError> {
    // Verify conversation belongs to project
    let exists: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM conversations WHERE id = $1 AND project_id = $2",
    )
    .bind(conversation_id)
    .bind(project_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    if exists.is_none() {
        return Err(AppError::NotFound("Conversation not found".to_string()));
    }

    let conv: Conversation = sqlx::query_as(
        "UPDATE conversations SET status = $1, updated_at = NOW()
         WHERE id = $2
         RETURNING id, project_id, end_user_id, status, created_at, updated_at",
    )
    .bind(status.as_str())
    .bind(conversation_id)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(ConversationResponse::from(conv))
}

/// Send a message as developer in a conversation.
pub async fn send_developer_message(
    pool: &PgPool,
    project_id: Uuid,
    conversation_id: Uuid,
    developer_id: Uuid,
    content: &str,
) -> Result<MessageResponse, AppError> {
    // Verify conversation belongs to project
    let exists: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM conversations WHERE id = $1 AND project_id = $2",
    )
    .bind(conversation_id)
    .bind(project_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    if exists.is_none() {
        return Err(AppError::NotFound("Conversation not found".to_string()));
    }

    // Use the chat service to send the message
    crate::service::chat::send_message(
        pool,
        conversation_id,
        &SenderType::Developer,
        Some(developer_id),
        content,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn test_pool() -> sqlx::PgPool {
        let _guard = crate::test_support::ENV_MUTEX.lock().unwrap();
        let _ = dotenvy::dotenv_override();
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(2)
            .connect(&url)
            .await
            .expect("connect");
        crate::db::run_migrations(&pool).await.expect("migrations");
        pool
    }

    async fn setup_data(pool: &sqlx::PgPool) -> (Uuid, Uuid, Uuid, Uuid) {
        let dev_id = Uuid::now_v7();
        sqlx::query("INSERT INTO developers (id, email, password_hash, name) VALUES ($1,$2,$3,$4)")
            .bind(dev_id)
            .bind(format!("admin_svc_{}@test.com", dev_id))
            .bind("$argon2id$hash")
            .bind("Admin Dev")
            .execute(pool)
            .await
            .expect("insert developer");

        let proj_id = Uuid::now_v7();
        sqlx::query("INSERT INTO projects (id, developer_id, name, description, api_key) VALUES ($1,$2,$3,$4,$5)")
            .bind(proj_id)
            .bind(dev_id)
            .bind("Admin Project")
            .bind("")
            .bind(format!("proj_admin_{}", proj_id))
            .execute(pool)
            .await
            .expect("insert project");

        let user_id = Uuid::now_v7();
        sqlx::query("INSERT INTO end_users (id, project_id, device_id) VALUES ($1,$2,$3)")
            .bind(user_id)
            .bind(proj_id)
            .bind(format!("device_{}", user_id))
            .execute(pool)
            .await
            .expect("insert end_user");

        let conv_id = Uuid::now_v7();
        sqlx::query("INSERT INTO conversations (id, project_id, end_user_id) VALUES ($1,$2,$3)")
            .bind(conv_id)
            .bind(proj_id)
            .bind(user_id)
            .execute(pool)
            .await
            .expect("insert conversation");

        (dev_id, proj_id, user_id, conv_id)
    }

    async fn cleanup(pool: &sqlx::PgPool, dev_id: Uuid) {
        sqlx::query("DELETE FROM messages WHERE conversation_id IN (SELECT c.id FROM conversations c JOIN projects p ON c.project_id=p.id WHERE p.developer_id=$1)")
            .bind(dev_id).execute(pool).await.ok();
        sqlx::query("DELETE FROM conversations WHERE project_id IN (SELECT id FROM projects WHERE developer_id=$1)")
            .bind(dev_id).execute(pool).await.ok();
        sqlx::query("DELETE FROM end_users WHERE project_id IN (SELECT id FROM projects WHERE developer_id=$1)")
            .bind(dev_id).execute(pool).await.ok();
        sqlx::query("DELETE FROM projects WHERE developer_id=$1")
            .bind(dev_id).execute(pool).await.ok();
        sqlx::query("DELETE FROM developers WHERE id=$1")
            .bind(dev_id).execute(pool).await.ok();
    }

    #[tokio::test]
    async fn list_conversations_returns_results() {
        let pool = test_pool().await;
        let (dev_id, proj_id, _, _) = setup_data(&pool).await;

        let result = list_conversations(&pool, proj_id, ListConversationsQuery::default()).await;
        assert!(result.is_ok());
        let convs = result.unwrap();
        assert!(!convs.is_empty());
        assert_eq!(convs[0].conversation.project_id, proj_id);

        cleanup(&pool, dev_id).await;
    }

    #[tokio::test]
    async fn list_conversations_filters_by_status() {
        let pool = test_pool().await;
        let (dev_id, proj_id, _, conv_id) = setup_data(&pool).await;

        // Close the conversation
        sqlx::query("UPDATE conversations SET status = 'closed' WHERE id = $1")
            .bind(conv_id)
            .execute(&pool)
            .await
            .unwrap();

        // Filter by open - should be empty
        let open_convs = list_conversations(
            &pool,
            proj_id,
            ListConversationsQuery {
                status: Some("open".to_string()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert!(open_convs.is_empty());

        // Filter by closed - should have one
        let closed_convs = list_conversations(
            &pool,
            proj_id,
            ListConversationsQuery {
                status: Some("closed".to_string()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(closed_convs.len(), 1);

        cleanup(&pool, dev_id).await;
    }

    #[tokio::test]
    async fn get_conversation_returns_details() {
        let pool = test_pool().await;
        let (dev_id, proj_id, user_id, conv_id) = setup_data(&pool).await;

        let result = get_conversation(&pool, proj_id, conv_id).await;
        assert!(result.is_ok());
        let conv = result.unwrap();
        assert_eq!(conv.conversation.id, conv_id);
        assert_eq!(conv.end_user.id, user_id);

        cleanup(&pool, dev_id).await;
    }

    #[tokio::test]
    async fn get_conversation_wrong_project_returns_not_found() {
        let pool = test_pool().await;
        let (dev_id, _, _, conv_id) = setup_data(&pool).await;

        let fake_project = Uuid::now_v7();
        let result = get_conversation(&pool, fake_project, conv_id).await;
        assert!(matches!(result, Err(AppError::NotFound(_))));

        cleanup(&pool, dev_id).await;
    }

    #[tokio::test]
    async fn update_conversation_status_changes_status() {
        let pool = test_pool().await;
        let (dev_id, proj_id, _, conv_id) = setup_data(&pool).await;

        // Update to closed
        let result = update_conversation_status(
            &pool,
            proj_id,
            conv_id,
            &ConversationStatus::Closed,
        )
        .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, "closed");

        cleanup(&pool, dev_id).await;
    }

    #[tokio::test]
    async fn send_developer_message_persists_message() {
        let pool = test_pool().await;
        let (dev_id, proj_id, _, conv_id) = setup_data(&pool).await;

        let result = send_developer_message(&pool, proj_id, conv_id, dev_id, "Hello from developer!").await;
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert_eq!(msg.content, "Hello from developer!");
        assert_eq!(msg.sender_type, "developer");

        cleanup(&pool, dev_id).await;
    }

    #[tokio::test]
    async fn send_developer_message_invalid_conversation_returns_not_found() {
        let pool = test_pool().await;
        let (dev_id, proj_id, _, _) = setup_data(&pool).await;

        let fake_conv = Uuid::now_v7();
        let result = send_developer_message(&pool, proj_id, fake_conv, dev_id, "Hello").await;
        assert!(matches!(result, Err(AppError::NotFound(_))));

        cleanup(&pool, dev_id).await;
    }
}
