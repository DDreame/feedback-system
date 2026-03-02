use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Full project record as stored in the `projects` table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Project {
    pub id: Uuid,
    pub developer_id: Uuid,
    pub name: String,
    pub description: String,
    pub api_key: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input payload for creating a project.
#[derive(Debug, Deserialize)]
pub struct CreateProject {
    pub name: String,
    pub description: Option<String>,
}

/// Project data safe to return to API consumers.
#[derive(Debug, Clone, Serialize)]
pub struct ProjectResponse {
    pub id: Uuid,
    pub developer_id: Uuid,
    pub name: String,
    pub description: String,
    pub api_key: String,
    pub created_at: DateTime<Utc>,
}

impl From<Project> for ProjectResponse {
    fn from(p: Project) -> Self {
        ProjectResponse {
            id: p.id,
            developer_id: p.developer_id,
            name: p.name,
            description: p.description,
            api_key: p.api_key,
            created_at: p.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn make_project() -> Project {
        Project {
            id: Uuid::now_v7(),
            developer_id: Uuid::now_v7(),
            name: "My App".to_string(),
            description: "A cool app".to_string(),
            api_key: "proj_abc123".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn project_response_from_project_copies_fields() {
        let p = make_project();
        let r = ProjectResponse::from(p.clone());
        assert_eq!(r.id, p.id);
        assert_eq!(r.developer_id, p.developer_id);
        assert_eq!(r.name, p.name);
        assert_eq!(r.description, p.description);
        assert_eq!(r.api_key, p.api_key);
        assert_eq!(r.created_at, p.created_at);
    }

    #[test]
    fn project_response_serializes_to_json() {
        let p = make_project();
        let r = ProjectResponse::from(p.clone());
        let json = serde_json::to_value(&r).expect("should serialize");
        assert_eq!(json["name"], p.name);
        assert_eq!(json["api_key"], p.api_key);
        assert!(json["id"].is_string());
    }

    #[test]
    fn create_project_deserializes_from_json() {
        let json = r#"{"name":"Test","description":"Desc"}"#;
        let dto: CreateProject = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(dto.name, "Test");
        assert_eq!(dto.description.as_deref(), Some("Desc"));
    }

    #[test]
    fn create_project_description_is_optional() {
        let json = r#"{"name":"Test"}"#;
        let dto: CreateProject = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(dto.name, "Test");
        assert!(dto.description.is_none());
    }

    #[tokio::test]
    async fn project_from_row_roundtrip() {
        let _guard = crate::test_support::ENV_MUTEX.lock().unwrap();
        let _ = dotenvy::dotenv_override();
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(2)
            .connect(&url)
            .await
            .expect("should connect");
        crate::db::run_migrations(&pool).await.expect("migrations");

        // Need a developer first (FK constraint)
        let dev_id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO developers (id, email, password_hash, name) VALUES ($1, $2, $3, $4)",
        )
        .bind(dev_id)
        .bind(format!("proj_model_{}@test.com", dev_id))
        .bind("$argon2id$hash")
        .bind("Test Dev")
        .execute(&pool)
        .await
        .expect("insert developer");

        let proj_id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO projects (id, developer_id, name, description, api_key) VALUES ($1,$2,$3,$4,$5)",
        )
        .bind(proj_id)
        .bind(dev_id)
        .bind("My App")
        .bind("A desc")
        .bind(format!("proj_test_{}", proj_id))
        .execute(&pool)
        .await
        .expect("insert project");

        let project: Project = sqlx::query_as(
            "SELECT id, developer_id, name, description, api_key, created_at, updated_at
             FROM projects WHERE id = $1",
        )
        .bind(proj_id)
        .fetch_one(&pool)
        .await
        .expect("fetch project");

        assert_eq!(project.id, proj_id);
        assert_eq!(project.developer_id, dev_id);
        assert_eq!(project.name, "My App");

        // Cleanup
        sqlx::query("DELETE FROM projects WHERE id = $1").bind(proj_id).execute(&pool).await.ok();
        sqlx::query("DELETE FROM developers WHERE id = $1").bind(dev_id).execute(&pool).await.ok();
    }
}
