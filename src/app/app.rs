use super::chat::{ChatMessage, Chats, PeerMessage};
use crate::{Peer, PeerEvent};
use tokio::io::{AsyncBufReadExt, BufReader};

pub struct App {
    chats: Chats,
    peer: Peer,
}

impl App {
    pub fn new(peer: Peer) -> Self {
        let mut listener = peer.subscribe();
        let chats = Chats::new();
        let chats_cloned = chats.clone();
        tokio::spawn(async move {
            while let Ok(event) = listener.recv().await {
                match event {
                    PeerEvent::MessageReceived(event) => {
                        let message = ChatMessage::PeerMessage(PeerMessage {
                            message_id: event.message_id().to_string(),
                            peer_id: Some(event.peer_id().to_string()),
                            message: event.message().to_string(),
                            timestamp: *event.timestamp(),
                        });

                        chats_cloned
                            .add_message(event.topic().to_string(), message)
                            .await;
                    }
                }
            }
        });

        Self { chats, peer }
    }

    pub async fn start(&self) {
        let mut stdin = BufReader::new(tokio::io::stdin()).lines();

        while let Some(line) = stdin.next_line().await.unwrap() {
            match line.split_once(" ") {
                Some(("/join", room)) => match self.peer.subscribe_topic(room.to_string()).await {
                    Ok(_) => {
                        println!("Joined room: {}", room);
                    }
                    Err(e) => {
                        log::error!("Failed to join room: {}", e);
                    }
                },
                Some(("/send", message)) => match message.split_once(" ") {
                    Some((room, message)) => {
                        match self
                            .peer
                            .send_message(message.to_string(), room.to_string())
                            .await
                        {
                            Ok(message_id) => {
                                let message = ChatMessage::PeerMessage(PeerMessage {
                                    peer_id: None,
                                    message: message.to_string(),
                                    message_id: message_id.to_string(),
                                    timestamp: chrono::Utc::now().timestamp() as u64,
                                });

                                self.chats.add_message(room.to_string(), message).await;
                            }
                            Err(e) => {
                                log::error!("Failed to send message: {}", e);
                            }
                        }
                    }
                    None => {
                        log::error!("Invalid message format");
                    }
                },
                Some(("/chat", room)) => {
                    println!("Messages in room: {}", room);
                    let room = room.to_string();
                    let messages = self.chats.get_messages(room).await;
                    for message in messages {
                        println!("message: {:?}", message);
                    }
                }
                _ => {
                    log::error!("Unknown command");
                }
            }
        }
    }
}
