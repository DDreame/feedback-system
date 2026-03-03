use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::model::message::{Message, MessageResponse, SenderType};

/// Store a new message in the database.
pub async fn send_message(
    pool: &PgPool,
    conversation_id: Uuid,
    sender_type: &SenderType,
    sender_id: Option<Uuid>,
    content: &str,
) -> Result<MessageResponse, AppError> {
    if content.trim().is_empty() {
        return Err(AppError::BadRequest("Message content must not be empty".to_string()));
    }

    // Verify conversation exists
    let exists: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM conversations WHERE id = $1",
    )
    .bind(conversation_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    if exists.is_none() {
        return Err(AppError::NotFound("Conversation not found".to_string()));
    }

    let id = Uuid::now_v7();
    let msg: Message = sqlx::query_as(
        "INSERT INTO messages (id, conversation_id, sender_type, sender_id, message_type, content)
         VALUES ($1, $2, $3, $4, 'text', $5)
         RETURNING id, conversation_id, sender_type, sender_id, message_type, content, created_at",
    )
    .bind(id)
    .bind(conversation_id)
    .bind(sender_type.as_str())
    .bind(sender_id)
    .bind(content)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    // Update conversation's updated_at timestamp
    sqlx::query("UPDATE conversations SET updated_at = NOW() WHERE id = $1")
        .bind(conversation_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(MessageResponse::from(msg))
}

/// List messages for a conversation with cursor-based pagination.
/// Returns messages older than `before` cursor (or latest if None), limited to `limit`.
pub async fn list_messages(
    pool: &PgPool,
    conversation_id: Uuid,
    before: Option<Uuid>,
    limit: i64,
) -> Result<Vec<MessageResponse>, AppError> {
    let limit = limit.min(100).max(1);

    let messages: Vec<Message> = if let Some(cursor) = before {
        sqlx::query_as(
            "SELECT id, conversation_id, sender_type, sender_id, message_type, content, created_at
             FROM messages
             WHERE conversation_id = $1 AND created_at < (SELECT created_at FROM messages WHERE id = $2)
             ORDER BY created_at DESC
             LIMIT $3",
        )
        .bind(conversation_id)
        .bind(cursor)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    } else {
        sqlx::query_as(
            "SELECT id, conversation_id, sender_type, sender_id, message_type, content, created_at
             FROM messages
             WHERE conversation_id = $1
             ORDER BY created_at DESC
             LIMIT $2",
        )
        .bind(conversation_id)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    };

    Ok(messages.into_iter().map(MessageResponse::from).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;

    async fn test_pool() -> PgPool {
        let _guard = crate::test_support::ENV_MUTEX.lock().unwrap();
        let _ = dotenvy::dotenv_override();
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPoolOptions::new().max_connections(2).connect(&url).await.expect("connect");
        crate::db::run_migrations(&pool).await.expect("migrations");
        pool
    }

    /// Creates a developer → project → end_user → conversation chain for testing.
    /// Returns (developer_id, project_id, end_user_id, conversation_id).
    async fn setup_conversation(pool: &PgPool) -> (Uuid, Uuid, Uuid, Uuid) {
        let dev_id = Uuid::now_v7();
        sqlx::query("INSERT INTO developers (id, email, password_hash, name) VALUES ($1,$2,$3,$4)")
            .bind(dev_id)
            .bind(format!("chat_svc_{}@test.com", dev_id))
            .bind("$argon2id$hash")
            .bind("Chat Dev")
            .execute(pool)
            .await
            .expect("insert developer");

        let proj_id = Uuid::now_v7();
        sqlx::query("INSERT INTO projects (id, developer_id, name, description, api_key) VALUES ($1,$2,$3,$4,$5)")
            .bind(proj_id)
            .bind(dev_id)
            .bind("Chat Project")
            .bind("")
            .bind(format!("proj_chat_{}", proj_id))
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

    async fn cleanup(pool: &PgPool, dev_id: Uuid) {
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
    async fn send_message_stores_and_returns() {
        let pool = test_pool().await;
        let (dev_id, _, user_id, conv_id) = setup_conversation(&pool).await;

        let result = send_message(&pool, conv_id, &SenderType::EndUser, Some(user_id), "Hello!").await;
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert_eq!(msg.conversation_id, conv_id);
        assert_eq!(msg.sender_type, "end_user");
        assert_eq!(msg.sender_id, Some(user_id));
        assert_eq!(msg.content, "Hello!");
        assert_eq!(msg.message_type, "text");

        cleanup(&pool, dev_id).await;
    }

    #[tokio::test]
    async fn send_message_empty_content_returns_bad_request() {
        let pool = test_pool().await;
        let (dev_id, _, user_id, conv_id) = setup_conversation(&pool).await;

        let result = send_message(&pool, conv_id, &SenderType::EndUser, Some(user_id), "  ").await;
        assert!(matches!(result, Err(AppError::BadRequest(_))));

        cleanup(&pool, dev_id).await;
    }

    #[tokio::test]
    async fn send_message_nonexistent_conversation_returns_not_found() {
        let pool = test_pool().await;
        let fake_conv = Uuid::now_v7();
        let result = send_message(&pool, fake_conv, &SenderType::EndUser, None, "Hello").await;
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }

    #[tokio::test]
    async fn list_messages_returns_recent_first() {
        let pool = test_pool().await;
        let (dev_id, _, user_id, conv_id) = setup_conversation(&pool).await;

        send_message(&pool, conv_id, &SenderType::EndUser, Some(user_id), "First").await.unwrap();
        send_message(&pool, conv_id, &SenderType::EndUser, Some(user_id), "Second").await.unwrap();
        send_message(&pool, conv_id, &SenderType::EndUser, Some(user_id), "Third").await.unwrap();

        let messages = list_messages(&pool, conv_id, None, 10).await.unwrap();
        assert_eq!(messages.len(), 3);
        // Most recent first
        assert_eq!(messages[0].content, "Third");
        assert_eq!(messages[1].content, "Second");
        assert_eq!(messages[2].content, "First");

        cleanup(&pool, dev_id).await;
    }

    #[tokio::test]
    async fn list_messages_with_cursor_pagination() {
        let pool = test_pool().await;
        let (dev_id, _, user_id, conv_id) = setup_conversation(&pool).await;

        send_message(&pool, conv_id, &SenderType::EndUser, Some(user_id), "Msg1").await.unwrap();
        send_message(&pool, conv_id, &SenderType::EndUser, Some(user_id), "Msg2").await.unwrap();
        let third = send_message(&pool, conv_id, &SenderType::EndUser, Some(user_id), "Msg3").await.unwrap();

        // Get messages before the third one
        let older = list_messages(&pool, conv_id, Some(third.id), 10).await.unwrap();
        assert_eq!(older.len(), 2);
        assert_eq!(older[0].content, "Msg2");
        assert_eq!(older[1].content, "Msg1");

        cleanup(&pool, dev_id).await;
    }

    #[tokio::test]
    async fn list_messages_respects_limit() {
        let pool = test_pool().await;
        let (dev_id, _, user_id, conv_id) = setup_conversation(&pool).await;

        for i in 0..5 {
            send_message(&pool, conv_id, &SenderType::EndUser, Some(user_id), &format!("Msg{i}")).await.unwrap();
        }

        let messages = list_messages(&pool, conv_id, None, 2).await.unwrap();
        assert_eq!(messages.len(), 2);

        cleanup(&pool, dev_id).await;
    }
}
