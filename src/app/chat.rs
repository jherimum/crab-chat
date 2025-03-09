use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Chats(Arc<Mutex<HashMap<String, Chat>>>);

impl Chats {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(HashMap::new())))
    }

    pub async fn add_message(&self, topic: String, message: ChatMessage) {
        let mut chats = self.0.lock().await;
        let chat = chats.entry(topic).or_insert_with(|| Chat::new());
        chat.add_message(message);
    }

    pub async fn get_messages(&self, topic: String) -> Vec<ChatMessage> {
        let chats_guard = self.0.lock().await;
        match chats_guard.get(&topic) {
            Some(chat) => chat.get_messages().into_iter().cloned().collect(),
            None => Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ChatMessage {
    PeerMessage(PeerMessage),
}

impl ChatMessage {
    pub fn timestamp(&self) -> u64 {
        match self {
            ChatMessage::PeerMessage(message) => message.timestamp,
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

#[derive(Default)]
pub struct Chat {
    messages: BTreeMap<u64, ChatMessage>,
}

impl Chat {
    pub fn new() -> Self {
        Self {
            messages: BTreeMap::new(),
        }
    }

    pub fn add_message(&mut self, message: ChatMessage) {
        self.messages.insert(message.timestamp(), message);
    }

    pub fn get_messages(&self) -> Vec<&ChatMessage> {
        self.messages.values().collect()
    }
}
