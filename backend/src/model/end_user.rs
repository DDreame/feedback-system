use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Full end-user record as stored in the `end_users` table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EndUser {
    pub id: Uuid,
    pub project_id: Uuid,
    pub device_id: String,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for finding or creating an end user.
#[derive(Debug, Deserialize)]
pub struct InitEndUser {
    pub device_id: String,
    pub name: Option<String>,
}

/// End-user data safe to return to API consumers.
#[derive(Debug, Clone, Serialize)]
pub struct EndUserResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub device_id: String,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<EndUser> for EndUserResponse {
    fn from(u: EndUser) -> Self {
        EndUserResponse {
            id: u.id,
            project_id: u.project_id,
            device_id: u.device_id,
            name: u.name,
            created_at: u.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_end_user() -> EndUser {
        EndUser {
            id: Uuid::now_v7(),
            project_id: Uuid::now_v7(),
            device_id: "device-abc-123".to_string(),
            name: Some("Alice".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn end_user_response_from_end_user_copies_fields() {
        let u = make_end_user();
        let r = EndUserResponse::from(u.clone());
        assert_eq!(r.id, u.id);
        assert_eq!(r.project_id, u.project_id);
        assert_eq!(r.device_id, u.device_id);
        assert_eq!(r.name, u.name);
        assert_eq!(r.created_at, u.created_at);
    }

    #[test]
    fn end_user_response_serializes_to_json() {
        let u = make_end_user();
        let r = EndUserResponse::from(u.clone());
        let json = serde_json::to_value(&r).expect("should serialize");
        assert_eq!(json["device_id"], u.device_id);
        assert_eq!(json["name"], u.name.as_deref().unwrap_or(""));
        assert!(json["id"].is_string());
    }

    #[test]
    fn end_user_response_without_name_serializes_null() {
        let mut u = make_end_user();
        u.name = None;
        let r = EndUserResponse::from(u);
        let json = serde_json::to_value(&r).expect("should serialize");
        assert!(json["name"].is_null());
    }

    #[test]
    fn init_end_user_deserializes_with_name() {
        let json = r#"{"device_id":"dev-1","name":"Bob"}"#;
        let dto: InitEndUser = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(dto.device_id, "dev-1");
        assert_eq!(dto.name.as_deref(), Some("Bob"));
    }

    #[test]
    fn init_end_user_deserializes_without_name() {
        let json = r#"{"device_id":"dev-2"}"#;
        let dto: InitEndUser = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(dto.device_id, "dev-2");
        assert!(dto.name.is_none());
    }
}
