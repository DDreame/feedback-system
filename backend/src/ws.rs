use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::model::message::MessageResponse;

/// A message broadcast over WebSocket to all participants of a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsBroadcast {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub message: MessageResponse,
}

/// Manages active WebSocket connections grouped by conversation.
///
/// Each conversation gets a broadcast channel. When a message is sent to a
/// conversation, all connected clients receive it.
#[derive(Clone)]
pub struct ConnectionManager {
    channels: Arc<RwLock<HashMap<Uuid, broadcast::Sender<String>>>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        ConnectionManager {
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get or create a broadcast sender for a conversation.
    pub async fn get_sender(&self, conversation_id: Uuid) -> broadcast::Sender<String> {
        {
            let channels = self.channels.read().await;
            if let Some(tx) = channels.get(&conversation_id) {
                return tx.clone();
            }
        }
        let mut channels = self.channels.write().await;
        // Double-check after acquiring write lock
        channels
            .entry(conversation_id)
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(128);
                tx
            })
            .clone()
    }

    /// Subscribe to messages for a conversation.
    pub async fn subscribe(
        &self,
        conversation_id: Uuid,
    ) -> broadcast::Receiver<String> {
        let tx = self.get_sender(conversation_id).await;
        tx.subscribe()
    }

    /// Broadcast a message to all connected clients in a conversation.
    pub async fn broadcast(&self, conversation_id: Uuid, msg: &MessageResponse) {
        let broadcast_msg = WsBroadcast {
            msg_type: "new_message".to_string(),
            message: msg.clone(),
        };
        if let Ok(json) = serde_json::to_string(&broadcast_msg) {
            let tx = self.get_sender(conversation_id).await;
            // Ignore send errors (no receivers)
            let _ = tx.send(json);
        }
    }

    /// Remove a channel if there are no more active senders/receivers.
    pub async fn cleanup(&self, conversation_id: Uuid) {
        let mut channels = self.channels.write().await;
        if let Some(tx) = channels.get(&conversation_id) {
            if tx.receiver_count() == 0 {
                channels.remove(&conversation_id);
            }
        }
    }
}

/// Incoming WebSocket message from a client.
#[derive(Debug, Deserialize)]
pub struct WsClientMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub content: Option<String>,
}

/// Handle a single WebSocket connection for a given conversation and end user.
pub async fn handle_ws_connection(
    mut socket: WebSocket,
    conversation_id: Uuid,
    end_user_id: Uuid,
    conn_mgr: ConnectionManager,
    db: sqlx::PgPool,
) {
    let mut rx = conn_mgr.subscribe(conversation_id).await;

    let (mut ws_tx, mut ws_rx) = socket.split();

    // Task: forward broadcast messages to this WebSocket client
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if ws_tx.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // Task: receive messages from this WebSocket client, persist and broadcast
    let conn_mgr_clone = conn_mgr.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_rx.next().await {
            match msg {
                Message::Text(text) => {
                    let text_ref: &str = &text;
                    if let Ok(client_msg) = serde_json::from_str::<WsClientMessage>(text_ref) {
                        if client_msg.msg_type == "send_message" {
                            if let Some(content) = client_msg.content {
                                if let Ok(saved) = crate::service::chat::send_message(
                                    &db,
                                    conversation_id,
                                    &crate::model::message::SenderType::EndUser,
                                    Some(end_user_id),
                                    &content,
                                )
                                .await
                                {
                                    conn_mgr_clone.broadcast(conversation_id, &saved).await;
                                }
                            }
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    // Wait for either task to complete, then abort the other
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    conn_mgr.cleanup(conversation_id).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::message::MessageResponse;
    use chrono::Utc;

    #[tokio::test]
    async fn connection_manager_subscribe_and_broadcast() {
        let mgr = ConnectionManager::new();
        let conv_id = Uuid::now_v7();

        let mut rx = mgr.subscribe(conv_id).await;

        let msg = MessageResponse {
            id: Uuid::now_v7(),
            conversation_id: conv_id,
            sender_type: "end_user".to_string(),
            sender_id: Some(Uuid::now_v7()),
            message_type: "text".to_string(),
            content: "Hello WS!".to_string(),
            created_at: Utc::now(),
        };

        mgr.broadcast(conv_id, &msg).await;

        let received = rx.recv().await.unwrap();
        let parsed: WsBroadcast = serde_json::from_str(&received).unwrap();
        assert_eq!(parsed.msg_type, "new_message");
        assert_eq!(parsed.message.content, "Hello WS!");
    }

    #[tokio::test]
    async fn connection_manager_multiple_subscribers() {
        let mgr = ConnectionManager::new();
        let conv_id = Uuid::now_v7();

        let mut rx1 = mgr.subscribe(conv_id).await;
        let mut rx2 = mgr.subscribe(conv_id).await;

        let msg = MessageResponse {
            id: Uuid::now_v7(),
            conversation_id: conv_id,
            sender_type: "end_user".to_string(),
            sender_id: None,
            message_type: "text".to_string(),
            content: "Broadcast test".to_string(),
            created_at: Utc::now(),
        };

        mgr.broadcast(conv_id, &msg).await;

        let r1: WsBroadcast = serde_json::from_str(&rx1.recv().await.unwrap()).unwrap();
        let r2: WsBroadcast = serde_json::from_str(&rx2.recv().await.unwrap()).unwrap();
        assert_eq!(r1.message.content, "Broadcast test");
        assert_eq!(r2.message.content, "Broadcast test");
    }

    #[tokio::test]
    async fn connection_manager_different_conversations_isolated() {
        let mgr = ConnectionManager::new();
        let conv_a = Uuid::now_v7();
        let conv_b = Uuid::now_v7();

        let mut rx_a = mgr.subscribe(conv_a).await;
        let mut rx_b = mgr.subscribe(conv_b).await;

        let msg = MessageResponse {
            id: Uuid::now_v7(),
            conversation_id: conv_a,
            sender_type: "end_user".to_string(),
            sender_id: None,
            message_type: "text".to_string(),
            content: "Only for A".to_string(),
            created_at: Utc::now(),
        };

        mgr.broadcast(conv_a, &msg).await;

        let r_a: WsBroadcast = serde_json::from_str(&rx_a.recv().await.unwrap()).unwrap();
        assert_eq!(r_a.message.content, "Only for A");

        // rx_b should not receive anything — try_recv should fail
        assert!(rx_b.try_recv().is_err());
    }

    #[tokio::test]
    async fn connection_manager_cleanup_removes_empty_channel() {
        let mgr = ConnectionManager::new();
        let conv_id = Uuid::now_v7();

        // Subscribe and immediately drop the receiver
        let _rx = mgr.subscribe(conv_id).await;
        drop(_rx);

        mgr.cleanup(conv_id).await;

        let channels = mgr.channels.read().await;
        assert!(!channels.contains_key(&conv_id));
    }

    #[test]
    fn ws_client_message_deserializes() {
        let json = r#"{"type":"send_message","content":"Hello!"}"#;
        let msg: WsClientMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.msg_type, "send_message");
        assert_eq!(msg.content.as_deref(), Some("Hello!"));
    }

    #[test]
    fn ws_broadcast_serializes() {
        let b = WsBroadcast {
            msg_type: "new_message".to_string(),
            message: MessageResponse {
                id: Uuid::now_v7(),
                conversation_id: Uuid::now_v7(),
                sender_type: "end_user".to_string(),
                sender_id: None,
                message_type: "text".to_string(),
                content: "Test".to_string(),
                created_at: Utc::now(),
            },
        };
        let json = serde_json::to_value(&b).unwrap();
        assert_eq!(json["type"], "new_message");
        assert_eq!(json["message"]["content"], "Test");
    }
}
