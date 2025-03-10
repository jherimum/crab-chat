use std::{
    collections::{BTreeMap, HashMap},
    fmt::Display,
    sync::Arc,
};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Chats(Arc<RwLock<HashMap<String, Chat>>>);

impl Chats {
    pub fn new() -> Self {
        Self(Default::default())
    }

    pub async fn rooms(&self) -> Vec<String> {
        self.0.read().await.keys().cloned().collect()
    }

    pub async fn add_room(&mut self, topic: &str) {
        self.0
            .write()
            .await
            .entry(topic.to_string())
            .or_insert_with(Chat::new);
    }

    pub async fn remove_room(&mut self, topic: &str) {
        self.0.write().await.remove(topic);
    }

    pub async fn add_message(&mut self, topic: &str, message: ChatMessage) {
        if let Some(chat) = self.0.write().await.get(topic) {
            chat.add_message(message).await;
        }
    }

    pub async fn get_messages(&self, topic: &str) -> Vec<ChatMessage> {
        if let Some(chat) = self.0.read().await.get(topic) {
            chat.get_messages().await
        } else {
            Vec::new()
        }
    }
}

#[derive(Clone, Debug)]
pub enum ChatMessage {
    PeerMessage(PeerMessage),
    NotificationMessage(NotificationMessage),
}

impl Display for ChatMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dispaly = match self {
            ChatMessage::PeerMessage(peer_message) => peer_message.to_string(),
            ChatMessage::NotificationMessage(notification_message) => {
                notification_message.to_string()
            }
        };
        write!(f, "{}", dispaly)
    }
}

impl ChatMessage {
    pub fn timestamp(&self) -> u64 {
        match self {
            ChatMessage::PeerMessage(message) => message.timestamp,
            ChatMessage::NotificationMessage(notification) => match notification {
                NotificationMessage::PeerJoined(notification) => notification.timestamp,
            },
        }
    }
}

#[derive(Clone, Debug)]
pub enum NotificationMessage {
    PeerJoined(PeerJoinedNotification),
}

#[derive(Clone, Debug)]
pub struct PeerJoinedNotification {
    pub peer_id: String,
    pub topic: String,
    pub timestamp: u64,
    pub leave: bool,
}

impl Display for NotificationMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotificationMessage::PeerJoined(notification) => {
                write!(
                    f,
                    "{} {} the room at {}",
                    notification.peer_id,
                    if notification.leave { "left" } else { "joined" },
                    notification.timestamp
                )
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct PeerMessage {
    pub message_id: String,
    pub peer_id: Option<String>,
    pub message: String,
    pub timestamp: u64,
}

impl Display for PeerMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} at {}: {}",
            self.peer_id.as_deref().unwrap_or("me"),
            self.timestamp,
            self.message
        )
    }
}

#[derive(Default, Clone)]
pub struct Chat {
    messages: Arc<RwLock<BTreeMap<u64, ChatMessage>>>,
}

impl Chat {
    pub fn new() -> Self {
        Self {
            messages: Default::default(),
        }
    }

    pub async fn add_message(&self, message: ChatMessage) {
        let mut messages = self.messages.write().await;
        messages.insert(message.timestamp(), message);
    }

    pub async fn get_messages(&self) -> Vec<ChatMessage> {
        let messages = self.messages.read().await;
        messages.values().cloned().collect()
    }
}
