use std::collections::BTreeMap;

pub struct Message {
    pub room: String,
    pub author: String,
    pub content: String,
    pub timestamp: u64,
}

pub struct Chat {
    messages: BTreeMap<u64, Message>,
}

impl Chat {
    pub fn new() -> Self {
        Self {
            messages: BTreeMap::new(),
        }
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.insert(message.timestamp, message);
    }

    pub fn get_messages(&self) -> Vec<&Message> {
        self.messages.values().collect()
    }
}
