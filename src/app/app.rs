use crate::{PeerCommandBus, PeerEvent, PeerEventListener, StartCommand};

use super::Chat;
use std::{collections::HashMap, sync::Arc};

pub struct App {
    chats: Arc<HashMap<String, Chat>>,
    command_bus: PeerCommandBus,
}

impl App {
    pub fn new(command_bus: PeerCommandBus, mut event_listener: PeerEventListener) -> Self {
        tokio::spawn(async move {
            while let Ok(event) = event_listener.recv().await {
                match event {
                    PeerEvent::Started => {
                        println!("Peer started");
                    }
                }
            }
        });

        Self {
            chats: Arc::new(HashMap::new()),
            command_bus,
        }
    }

    pub async fn start(&self) {
        let r = self
            .command_bus
            .send(StartCommand)
            .await
            .unwrap()
            .result()
            .await;
    }
}
