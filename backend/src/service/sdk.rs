use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::model::conversation::{Conversation, ConversationResponse};
use crate::model::end_user::{EndUser, EndUserResponse};

/// Find or create an end user identified by (project_id, device_id).
/// If a user with that device_id already exists in the project, return it;
/// otherwise insert a new record.
pub async fn find_or_create_end_user(
    pool: &PgPool,
    project_id: Uuid,
    device_id: &str,
    name: Option<&str>,
) -> Result<EndUserResponse, AppError> {
    if device_id.trim().is_empty() {
        return Err(AppError::BadRequest("device_id must not be empty".to_string()));
    }

    // Try to find existing user
    let existing: Option<EndUser> = sqlx::query_as(
        "SELECT id, project_id, device_id, name, created_at, updated_at
         FROM end_users
         WHERE project_id = $1 AND device_id = $2",
    )
    .bind(project_id)
    .bind(device_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    if let Some(user) = existing {
        return Ok(EndUserResponse::from(user));
    }

    // Insert new end user
    let id = Uuid::now_v7();
    let user: EndUser = sqlx::query_as(
        "INSERT INTO end_users (id, project_id, device_id, name)
         VALUES ($1, $2, $3, $4)
         RETURNING id, project_id, device_id, name, created_at, updated_at",
    )
    .bind(id)
    .bind(project_id)
    .bind(device_id)
    .bind(name)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(EndUserResponse::from(user))
}

/// Find the most recent open conversation for an end user, or create a new one.
pub async fn find_or_create_conversation(
    pool: &PgPool,
    project_id: Uuid,
    end_user_id: Uuid,
) -> Result<ConversationResponse, AppError> {
    // Look for an existing open conversation
    let existing: Option<Conversation> = sqlx::query_as(
        "SELECT id, project_id, end_user_id, status, created_at, updated_at
         FROM conversations
         WHERE project_id = $1 AND end_user_id = $2 AND status = 'open'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(project_id)
    .bind(end_user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    if let Some(conv) = existing {
        return Ok(ConversationResponse::from(conv));
    }

    // Create new conversation
    let id = Uuid::now_v7();
    let conv: Conversation = sqlx::query_as(
        "INSERT INTO conversations (id, project_id, end_user_id, status)
         VALUES ($1, $2, $3, 'open')
         RETURNING id, project_id, end_user_id, status, created_at, updated_at",
    )
    .bind(id)
    .bind(project_id)
    .bind(end_user_id)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(ConversationResponse::from(conv))
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

    async fn create_test_developer(pool: &PgPool) -> Uuid {
        let id = Uuid::now_v7();
        sqlx::query("INSERT INTO developers (id, email, password_hash, name) VALUES ($1,$2,$3,$4)")
            .bind(id)
            .bind(format!("sdk_svc_{}@test.com", id))
            .bind("$argon2id$hash")
            .bind("SDK Test Dev")
            .execute(pool)
            .await
            .expect("insert developer");
        id
    }

    async fn create_test_project(pool: &PgPool, dev_id: Uuid) -> Uuid {
        let id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO projects (id, developer_id, name, description, api_key) VALUES ($1,$2,$3,$4,$5)",
        )
        .bind(id)
        .bind(dev_id)
        .bind("Test Project")
        .bind("")
        .bind(format!("proj_test_{}", id))
        .execute(pool)
        .await
        .expect("insert project");
        id
    }

    #[tokio::test]
    async fn find_or_create_end_user_creates_new_user() {
        let pool = test_pool().await;
        let dev_id = create_test_developer(&pool).await;
        let proj_id = create_test_project(&pool, dev_id).await;

        let result = find_or_create_end_user(&pool, proj_id, "device-001", Some("Alice")).await;
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.device_id, "device-001");
        assert_eq!(user.name.as_deref(), Some("Alice"));
        assert_eq!(user.project_id, proj_id);

        // Cleanup
        sqlx::query("DELETE FROM end_users WHERE project_id = $1").bind(proj_id).execute(&pool).await.ok();
        sqlx::query("DELETE FROM projects WHERE id = $1").bind(proj_id).execute(&pool).await.ok();
        sqlx::query("DELETE FROM developers WHERE id = $1").bind(dev_id).execute(&pool).await.ok();
    }

    #[tokio::test]
    async fn find_or_create_end_user_returns_existing_user() {
        let pool = test_pool().await;
        let dev_id = create_test_developer(&pool).await;
        let proj_id = create_test_project(&pool, dev_id).await;

        // Create user first
        let first = find_or_create_end_user(&pool, proj_id, "device-002", None).await.unwrap();
        // Call again with same device_id
        let second = find_or_create_end_user(&pool, proj_id, "device-002", Some("Bob")).await.unwrap();

        // Should return the same user (idempotent), not create a new one
        assert_eq!(first.id, second.id);
        assert_eq!(second.device_id, "device-002");

        // Cleanup
        sqlx::query("DELETE FROM end_users WHERE project_id = $1").bind(proj_id).execute(&pool).await.ok();
        sqlx::query("DELETE FROM projects WHERE id = $1").bind(proj_id).execute(&pool).await.ok();
        sqlx::query("DELETE FROM developers WHERE id = $1").bind(dev_id).execute(&pool).await.ok();
    }

    #[tokio::test]
    async fn find_or_create_end_user_empty_device_id_returns_bad_request() {
        let pool = test_pool().await;
        let proj_id = Uuid::now_v7();
        let result = find_or_create_end_user(&pool, proj_id, "  ", None).await;
        assert!(matches!(result, Err(AppError::BadRequest(_))));
    }

    #[tokio::test]
    async fn find_or_create_conversation_creates_new() {
        let pool = test_pool().await;
        let dev_id = create_test_developer(&pool).await;
        let proj_id = create_test_project(&pool, dev_id).await;
        let user = find_or_create_end_user(&pool, proj_id, "device-003", None).await.unwrap();

        let result = find_or_create_conversation(&pool, proj_id, user.id).await;
        assert!(result.is_ok());
        let conv = result.unwrap();
        assert_eq!(conv.project_id, proj_id);
        assert_eq!(conv.end_user_id, user.id);
        assert_eq!(conv.status, "open");

        // Cleanup
        sqlx::query("DELETE FROM conversations WHERE project_id = $1").bind(proj_id).execute(&pool).await.ok();
        sqlx::query("DELETE FROM end_users WHERE project_id = $1").bind(proj_id).execute(&pool).await.ok();
        sqlx::query("DELETE FROM projects WHERE id = $1").bind(proj_id).execute(&pool).await.ok();
        sqlx::query("DELETE FROM developers WHERE id = $1").bind(dev_id).execute(&pool).await.ok();
    }

    #[tokio::test]
    async fn find_or_create_conversation_returns_existing_open() {
        let pool = test_pool().await;
        let dev_id = create_test_developer(&pool).await;
        let proj_id = create_test_project(&pool, dev_id).await;
        let user = find_or_create_end_user(&pool, proj_id, "device-004", None).await.unwrap();

        let first = find_or_create_conversation(&pool, proj_id, user.id).await.unwrap();
        let second = find_or_create_conversation(&pool, proj_id, user.id).await.unwrap();

        assert_eq!(first.id, second.id, "Should return same open conversation");

        // Cleanup
        sqlx::query("DELETE FROM conversations WHERE project_id = $1").bind(proj_id).execute(&pool).await.ok();
        sqlx::query("DELETE FROM end_users WHERE project_id = $1").bind(proj_id).execute(&pool).await.ok();
        sqlx::query("DELETE FROM projects WHERE id = $1").bind(proj_id).execute(&pool).await.ok();
        sqlx::query("DELETE FROM developers WHERE id = $1").bind(dev_id).execute(&pool).await.ok();
    }
}
