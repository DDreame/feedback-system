use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Full developer record as stored in the `developers` table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Developer {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input payload for developer registration.
#[derive(Debug, Deserialize)]
pub struct CreateDeveloper {
    pub email: String,
    pub password: String,
    pub name: String,
}

/// Developer data safe to return to API consumers (excludes `password_hash`).
#[derive(Debug, Serialize)]
pub struct DeveloperResponse {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

impl From<Developer> for DeveloperResponse {
    fn from(d: Developer) -> Self {
        DeveloperResponse {
            id: d.id,
            email: d.email,
            name: d.name,
            created_at: d.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn make_developer() -> Developer {
        Developer {
            id: Uuid::now_v7(),
            email: "dev@example.com".to_string(),
            password_hash: "$argon2id$v=19$m=19456,t=2,p=1$...".to_string(),
            name: "Alice".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn developer_response_excludes_password_hash() {
        let dev = make_developer();
        let response = DeveloperResponse::from(dev.clone());

        let json = serde_json::to_value(&response).expect("should serialize");
        assert!(!json.as_object().unwrap().contains_key("password_hash"),
            "password_hash must not appear in DeveloperResponse");
        assert_eq!(json["email"], dev.email);
        assert_eq!(json["name"], dev.name);
    }

    #[test]
    fn developer_response_from_developer_copies_fields() {
        let dev = make_developer();
        let response = DeveloperResponse::from(dev.clone());

        assert_eq!(response.id, dev.id);
        assert_eq!(response.email, dev.email);
        assert_eq!(response.name, dev.name);
        assert_eq!(response.created_at, dev.created_at);
    }

    #[test]
    fn create_developer_deserializes_from_json() {
        let json = r#"{"email":"new@example.com","password":"s3cr3t!","name":"Bob"}"#;
        let dto: CreateDeveloper = serde_json::from_str(json).expect("should deserialize");

        assert_eq!(dto.email, "new@example.com");
        assert_eq!(dto.password, "s3cr3t!");
        assert_eq!(dto.name, "Bob");
    }

    /// Verify that the Developer struct can actually be loaded from a real DB row via sqlx::FromRow.
    /// Requires a running PostgreSQL instance with migrations applied.
    #[tokio::test]
    async fn developer_from_row_roundtrip() {
        let _guard = crate::test_support::ENV_MUTEX.lock().unwrap();
        let _ = dotenvy::dotenv_override();
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(2)
            .connect(&url)
            .await
            .expect("should connect");

        crate::db::run_migrations(&pool).await.expect("migrations should run");

        // Insert a test row and fetch it back via FromRow
        let id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO developers (id, email, password_hash, name) VALUES ($1, $2, $3, $4)",
        )
        .bind(id)
        .bind("fromrow@example.com")
        .bind("$argon2id$hash")
        .bind("FromRow Test")
        .execute(&pool)
        .await
        .expect("insert should succeed");

        let dev: Developer =
            sqlx::query_as("SELECT id, email, password_hash, name, created_at, updated_at FROM developers WHERE id = $1")
                .bind(id)
                .fetch_one(&pool)
                .await
                .expect("should fetch developer row");

        assert_eq!(dev.id, id);
        assert_eq!(dev.email, "fromrow@example.com");
        assert_eq!(dev.name, "FromRow Test");

        // Cleanup
        sqlx::query("DELETE FROM developers WHERE id = $1")
            .bind(id)
            .execute(&pool)
            .await
            .expect("cleanup should succeed");
    }
}
