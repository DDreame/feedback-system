use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use uuid::Uuid;

/// Manages user online status using Redis.
#[derive(Clone)]
pub struct PresenceService {
    conn: ConnectionManager,
}

impl PresenceService {
    pub fn new(conn: ConnectionManager) -> Self {
        PresenceService { conn }
    }

    /// Mark a user as online for a conversation.
    pub async fn set_online(&mut self, conversation_id: Uuid, user_id: Uuid) -> Result<(), redis::RedisError> {
        let key = format!("presence:{}:{}", conversation_id, user_id);
        // Set expiry to 30 seconds, caller should refresh periodically
        self.conn.set_ex(key, "online", 30).await
    }

    /// Mark a user as offline.
    pub async fn set_offline(&mut self, conversation_id: Uuid, user_id: Uuid) -> Result<(), redis::RedisError> {
        let key = format!("presence:{}:{}", conversation_id, user_id);
        self.conn.del(key).await
    }

    /// Check if a specific user is online.
    pub async fn is_online(&mut self, conversation_id: Uuid, user_id: Uuid) -> Result<bool, redis::RedisError> {
        let key = format!("presence:{}:{}", conversation_id, user_id);
        let exists: Option<String> = self.conn.get(key).await?;
        Ok(exists.is_some())
    }
}

/// Message publisher for cross-instance communication.
#[derive(Clone)]
pub struct PubSubService {
    conn: ConnectionManager,
}

impl PubSubService {
    pub fn new(conn: ConnectionManager) -> Self {
        PubSubService { conn }
    }

    /// Publish a message to a conversation channel.
    pub async fn publish(&mut self, conversation_id: Uuid, message: &str) -> Result<usize, redis::RedisError> {
        let channel = format!("chat:{}", conversation_id);
        let result: usize = self.conn.publish(channel, message).await?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a running Redis instance.

    #[tokio::test]
    #[ignore = "requires a running Redis instance"]
    async fn presence_set_online_and_check() {
        let url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
        let conn_mgr = redis::Client::open(url.as_str())
            .unwrap()
            .get_connection_manager()
            .await
            .unwrap();

        let mut presence = PresenceService::new(conn_mgr);

        let conv_id = Uuid::now_v7();
        let user_id = Uuid::now_v7();

        // Set online
        presence.set_online(conv_id, user_id).await.unwrap();

        // Check is online
        let is_online = presence.is_online(conv_id, user_id).await.unwrap();
        assert!(is_online);

        // Set offline
        presence.set_offline(conv_id, user_id).await.unwrap();

        // Check is offline
        let is_online = presence.is_online(conv_id, user_id).await.unwrap();
        assert!(!is_online);
    }

    #[tokio::test]
    #[ignore = "requires a running Redis instance"]
    async fn pubsub_publish_message() {
        let url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
        let conn_mgr = redis::Client::open(url.as_str())
            .unwrap()
            .get_connection_manager()
            .await
            .unwrap();

        let mut pubsub = PubSubService::new(conn_mgr);

        let conv_id = Uuid::now_v7();
        let test_message = r#"{"type":"new_message","message":{"id":"test"}}"#;

        // Publish message - returns number of subscribers
        let count = pubsub.publish(conv_id, test_message).await.unwrap();
        // count >= 0 means publish succeeded (may be 0 if no subscribers)
        assert!(count >= 0);
    }
}
