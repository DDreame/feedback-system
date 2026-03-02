use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Conversation status values matching the DB CHECK constraint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConversationStatus {
    Open,
    Closed,
}

impl ConversationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConversationStatus::Open => "open",
            ConversationStatus::Closed => "closed",
        }
    }
}

impl std::fmt::Display for ConversationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Full conversation record as stored in the `conversations` table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Conversation {
    pub id: Uuid,
    pub project_id: Uuid,
    pub end_user_id: Uuid,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Conversation data safe to return to API consumers.
#[derive(Debug, Clone, Serialize)]
pub struct ConversationResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub end_user_id: Uuid,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

impl From<Conversation> for ConversationResponse {
    fn from(c: Conversation) -> Self {
        ConversationResponse {
            id: c.id,
            project_id: c.project_id,
            end_user_id: c.end_user_id,
            status: c.status,
            created_at: c.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_conversation() -> Conversation {
        Conversation {
            id: Uuid::now_v7(),
            project_id: Uuid::now_v7(),
            end_user_id: Uuid::now_v7(),
            status: "open".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn conversation_response_from_conversation_copies_fields() {
        let c = make_conversation();
        let r = ConversationResponse::from(c.clone());
        assert_eq!(r.id, c.id);
        assert_eq!(r.project_id, c.project_id);
        assert_eq!(r.end_user_id, c.end_user_id);
        assert_eq!(r.status, c.status);
        assert_eq!(r.created_at, c.created_at);
    }

    #[test]
    fn conversation_response_serializes_to_json() {
        let c = make_conversation();
        let r = ConversationResponse::from(c.clone());
        let json = serde_json::to_value(&r).expect("should serialize");
        assert_eq!(json["status"], "open");
        assert!(json["id"].is_string());
    }

    #[test]
    fn conversation_status_enum_as_str() {
        assert_eq!(ConversationStatus::Open.as_str(), "open");
        assert_eq!(ConversationStatus::Closed.as_str(), "closed");
    }

    #[test]
    fn conversation_status_serializes_lowercase() {
        let json = serde_json::to_value(ConversationStatus::Open).expect("serialize");
        assert_eq!(json, "open");
        let json = serde_json::to_value(ConversationStatus::Closed).expect("serialize");
        assert_eq!(json, "closed");
    }
}
