use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::model::project::{CreateProject, Project, ProjectResponse};
use crate::utils::api_key::generate_api_key;

/// Create a new project owned by `developer_id`.
pub async fn create(
    pool: &PgPool,
    developer_id: Uuid,
    dto: CreateProject,
) -> Result<ProjectResponse, AppError> {
    if dto.name.trim().is_empty() {
        return Err(AppError::BadRequest("Project name must not be empty".to_string()));
    }

    let api_key = generate_api_key();
    let description = dto.description.unwrap_or_default();

    let project = sqlx::query_as::<_, Project>(
        "INSERT INTO projects (id, developer_id, name, description, api_key)
         VALUES (gen_random_uuid(), $1, $2, $3, $4)
         RETURNING id, developer_id, name, description, api_key, created_at, updated_at",
    )
    .bind(developer_id)
    .bind(dto.name.trim())
    .bind(&description)
    .bind(&api_key)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Internal(format!("Failed to create project: {e}")))?;

    Ok(ProjectResponse::from(project))
}

/// Fetch a single project by ID, verifying it belongs to `developer_id`.
pub async fn get(
    pool: &PgPool,
    project_id: Uuid,
    developer_id: Uuid,
) -> Result<ProjectResponse, AppError> {
    let project: Option<Project> = sqlx::query_as(
        "SELECT id, developer_id, name, description, api_key, created_at, updated_at
         FROM projects WHERE id = $1",
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Internal(format!("DB error: {e}")))?;

    match project {
        None => Err(AppError::NotFound("Project not found".to_string())),
        Some(p) if p.developer_id != developer_id => Err(AppError::NotFound("Project not found".to_string())),
        Some(p) => Ok(ProjectResponse::from(p)),
    }
}

/// List all projects owned by `developer_id` (ordered by created_at desc).
pub async fn list(pool: &PgPool, developer_id: Uuid) -> Result<Vec<ProjectResponse>, AppError> {
    let projects: Vec<Project> = sqlx::query_as(
        "SELECT id, developer_id, name, description, api_key, created_at, updated_at
         FROM projects WHERE developer_id = $1
         ORDER BY created_at DESC",
    )
    .bind(developer_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Internal(format!("DB error: {e}")))?;

    Ok(projects.into_iter().map(ProjectResponse::from).collect())
}

/// Update a project's name and/or description, verifying ownership.
pub async fn update(
    pool: &PgPool,
    project_id: Uuid,
    developer_id: Uuid,
    name: Option<String>,
    description: Option<String>,
) -> Result<ProjectResponse, AppError> {
    // Fetch first to verify ownership
    let existing = get(pool, project_id, developer_id).await?;

    let new_name = name.map(|n| n.trim().to_string()).unwrap_or(existing.name);
    let new_description = description.unwrap_or(existing.description);

    if new_name.is_empty() {
        return Err(AppError::BadRequest("Project name must not be empty".to_string()));
    }

    let project = sqlx::query_as::<_, Project>(
        "UPDATE projects SET name = $1, description = $2, updated_at = NOW()
         WHERE id = $3
         RETURNING id, developer_id, name, description, api_key, created_at, updated_at",
    )
    .bind(&new_name)
    .bind(&new_description)
    .bind(project_id)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Internal(format!("DB error: {e}")))?;

    Ok(ProjectResponse::from(project))
}

/// Delete a project, verifying ownership.
pub async fn delete(pool: &PgPool, project_id: Uuid, developer_id: Uuid) -> Result<(), AppError> {
    // Verify ownership first (get returns NotFound if wrong owner)
    get(pool, project_id, developer_id).await?;

    sqlx::query("DELETE FROM projects WHERE id = $1")
        .bind(project_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(format!("DB error: {e}")))?;

    Ok(())
}

/// Regenerate the API key for a project, verifying ownership.
/// Returns the updated ProjectResponse with the new api_key.
pub async fn regenerate_api_key(
    pool: &PgPool,
    project_id: Uuid,
    developer_id: Uuid,
) -> Result<ProjectResponse, AppError> {
    // Verify ownership
    get(pool, project_id, developer_id).await?;

    let new_key = generate_api_key();

    let project = sqlx::query_as::<_, Project>(
        "UPDATE projects SET api_key = $1, updated_at = NOW()
         WHERE id = $2
         RETURNING id, developer_id, name, description, api_key, created_at, updated_at",
    )
    .bind(&new_key)
    .bind(project_id)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Internal(format!("DB error: {e}")))?;

    Ok(ProjectResponse::from(project))
}

/// Look up a project by its API key (used by SDK authentication).
pub async fn get_by_api_key(pool: &PgPool, api_key: &str) -> Result<ProjectResponse, AppError> {
    let project: Option<Project> = sqlx::query_as(
        "SELECT id, developer_id, name, description, api_key, created_at, updated_at
         FROM projects WHERE api_key = $1",
    )
    .bind(api_key)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Internal(format!("DB error: {e}")))?;

    project
        .map(ProjectResponse::from)
        .ok_or_else(|| AppError::Unauthorized("Invalid API key".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup() -> (PgPool, Uuid) {
        let _guard = crate::test_support::ENV_MUTEX.lock().unwrap();
        let _ = dotenvy::dotenv_override();
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(2)
            .connect(&url)
            .await
            .expect("connect");
        crate::db::run_migrations(&pool).await.expect("migrations");

        // Insert a test developer
        let dev_id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO developers (id, email, password_hash, name) VALUES ($1,$2,$3,$4)",
        )
        .bind(dev_id)
        .bind(format!("proj_svc_{}@test.com", dev_id))
        .bind("$argon2id$hash")
        .bind("Proj Dev")
        .execute(&pool)
        .await
        .expect("insert dev");

        (pool, dev_id)
    }

    async fn cleanup(pool: &PgPool, dev_id: Uuid) {
        sqlx::query("DELETE FROM projects WHERE developer_id = $1").bind(dev_id).execute(pool).await.ok();
        sqlx::query("DELETE FROM developers WHERE id = $1").bind(dev_id).execute(pool).await.ok();
    }

    #[tokio::test]
    async fn create_project_returns_response() {
        let (pool, dev_id) = setup().await;

        let dto = CreateProject { name: "Test App".to_string(), description: Some("Desc".to_string()) };
        let r = create(&pool, dev_id, dto).await.expect("create");

        assert_eq!(r.name, "Test App");
        assert_eq!(r.description, "Desc");
        assert!(r.api_key.starts_with("proj_"));
        assert_eq!(r.developer_id, dev_id);

        cleanup(&pool, dev_id).await;
    }

    #[tokio::test]
    async fn create_project_empty_name_returns_bad_request() {
        let (pool, dev_id) = setup().await;

        let dto = CreateProject { name: "   ".to_string(), description: None };
        let err = create(&pool, dev_id, dto).await.unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));

        cleanup(&pool, dev_id).await;
    }

    #[tokio::test]
    async fn get_project_returns_not_found_for_wrong_owner() {
        let (pool, dev_id) = setup().await;

        let dto = CreateProject { name: "App".to_string(), description: None };
        let r = create(&pool, dev_id, dto).await.expect("create");

        let other_dev = Uuid::now_v7();
        let err = get(&pool, r.id, other_dev).await.unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));

        cleanup(&pool, dev_id).await;
    }

    #[tokio::test]
    async fn list_projects_returns_only_owned() {
        let (pool, dev_id) = setup().await;

        create(&pool, dev_id, CreateProject { name: "A".to_string(), description: None }).await.ok();
        create(&pool, dev_id, CreateProject { name: "B".to_string(), description: None }).await.ok();

        let other_dev = Uuid::now_v7();
        sqlx::query("INSERT INTO developers (id, email, password_hash, name) VALUES ($1,$2,$3,$4)")
            .bind(other_dev).bind(format!("other_{}@test.com", other_dev)).bind("$hash").bind("Other")
            .execute(&pool).await.ok();
        create(&pool, other_dev, CreateProject { name: "Other".to_string(), description: None }).await.ok();

        let projects = list(&pool, dev_id).await.expect("list");
        assert!(projects.iter().all(|p| p.developer_id == dev_id));
        assert!(projects.len() >= 2);

        // cleanup other dev
        sqlx::query("DELETE FROM projects WHERE developer_id = $1").bind(other_dev).execute(&pool).await.ok();
        sqlx::query("DELETE FROM developers WHERE id = $1").bind(other_dev).execute(&pool).await.ok();
        cleanup(&pool, dev_id).await;
    }

    #[tokio::test]
    async fn update_project_changes_name() {
        let (pool, dev_id) = setup().await;

        let r = create(&pool, dev_id, CreateProject { name: "Old".to_string(), description: None }).await.expect("create");
        let updated = update(&pool, r.id, dev_id, Some("New Name".to_string()), None).await.expect("update");

        assert_eq!(updated.name, "New Name");
        assert_eq!(updated.api_key, r.api_key); // api_key unchanged

        cleanup(&pool, dev_id).await;
    }

    #[tokio::test]
    async fn delete_project_removes_it() {
        let (pool, dev_id) = setup().await;

        let r = create(&pool, dev_id, CreateProject { name: "ToDelete".to_string(), description: None }).await.expect("create");
        delete(&pool, r.id, dev_id).await.expect("delete");

        let err = get(&pool, r.id, dev_id).await.unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));

        cleanup(&pool, dev_id).await;
    }

    #[tokio::test]
    async fn delete_wrong_owner_returns_not_found() {
        let (pool, dev_id) = setup().await;

        let r = create(&pool, dev_id, CreateProject { name: "App".to_string(), description: None }).await.expect("create");
        let other = Uuid::now_v7();
        let err = delete(&pool, r.id, other).await.unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));

        cleanup(&pool, dev_id).await;
    }
}
