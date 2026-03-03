use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Who sent a message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SenderType {
    EndUser,
    Developer,
    System,
    Ai,
}

impl SenderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SenderType::EndUser => "end_user",
            SenderType::Developer => "developer",
            SenderType::System => "system",
            SenderType::Ai => "ai",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "end_user" => Some(SenderType::EndUser),
            "developer" => Some(SenderType::Developer),
            "system" => Some(SenderType::System),
            "ai" => Some(SenderType::Ai),
            _ => None,
        }
    }
}

/// The type/format of a message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Text,
    Image,
    File,
}

impl MessageType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageType::Text => "text",
            MessageType::Image => "image",
            MessageType::File => "file",
        }
    }
}

/// Full message record as stored in the `messages` table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Message {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub sender_type: String,
    pub sender_id: Option<Uuid>,
    pub message_type: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

/// Message data safe to return to API consumers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageResponse {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub sender_type: String,
    pub sender_id: Option<Uuid>,
    pub message_type: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl From<Message> for MessageResponse {
    fn from(m: Message) -> Self {
        MessageResponse {
            id: m.id,
            conversation_id: m.conversation_id,
            sender_type: m.sender_type,
            sender_id: m.sender_id,
            message_type: m.message_type,
            content: m.content,
            created_at: m.created_at,
        }
    }
}

/// Input payload for sending a message from the SDK.
#[derive(Debug, Deserialize)]
pub struct SendMessage {
    pub conversation_id: Uuid,
    pub content: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_message() -> Message {
        Message {
            id: Uuid::now_v7(),
            conversation_id: Uuid::now_v7(),
            sender_type: "end_user".to_string(),
            sender_id: Some(Uuid::now_v7()),
            message_type: "text".to_string(),
            content: "Hello!".to_string(),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn message_response_from_message_copies_fields() {
        let m = make_message();
        let r = MessageResponse::from(m.clone());
        assert_eq!(r.id, m.id);
        assert_eq!(r.conversation_id, m.conversation_id);
        assert_eq!(r.sender_type, m.sender_type);
        assert_eq!(r.sender_id, m.sender_id);
        assert_eq!(r.content, m.content);
    }

    #[test]
    fn message_response_serializes_to_json() {
        let m = make_message();
        let r = MessageResponse::from(m);
        let json = serde_json::to_value(&r).expect("should serialize");
        assert_eq!(json["sender_type"], "end_user");
        assert_eq!(json["message_type"], "text");
        assert_eq!(json["content"], "Hello!");
    }

    #[test]
    fn sender_type_as_str() {
        assert_eq!(SenderType::EndUser.as_str(), "end_user");
        assert_eq!(SenderType::Developer.as_str(), "developer");
        assert_eq!(SenderType::System.as_str(), "system");
        assert_eq!(SenderType::Ai.as_str(), "ai");
    }

    #[test]
    fn sender_type_from_str() {
        assert_eq!(SenderType::from_str("end_user"), Some(SenderType::EndUser));
        assert_eq!(SenderType::from_str("developer"), Some(SenderType::Developer));
        assert_eq!(SenderType::from_str("invalid"), None);
    }

    #[test]
    fn sender_type_serializes_snake_case() {
        let json = serde_json::to_value(SenderType::EndUser).expect("serialize");
        assert_eq!(json, "end_user");
    }

    #[test]
    fn message_type_as_str() {
        assert_eq!(MessageType::Text.as_str(), "text");
        assert_eq!(MessageType::Image.as_str(), "image");
        assert_eq!(MessageType::File.as_str(), "file");
    }

    #[test]
    fn send_message_deserializes() {
        let json = r#"{"conversation_id":"01234567-89ab-cdef-0123-456789abcdef","content":"Hi"}"#;
        let dto: SendMessage = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(dto.content, "Hi");
    }
}
