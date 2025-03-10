use super::chat::{ChatMessage, Chats, NotificationMessage, PeerJoinedNotification, PeerMessage};
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
        let mut chats_cloned = chats.clone();
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
                        chats_cloned.add_message(event.topic(), message).await;
                    }
                    PeerEvent::PeerJoined(event) => {
                        let message = ChatMessage::NotificationMessage(
                            NotificationMessage::PeerJoined(PeerJoinedNotification {
                                peer_id: event.peer_id().to_string(),
                                topic: event.topic().to_string(),
                                timestamp: *event.timestamp(),
                                leave: false,
                            }),
                        );
                        chats_cloned.add_message(event.topic(), message).await;
                    }
                    PeerEvent::PeerLeft(event) => {
                        let message = ChatMessage::NotificationMessage(
                            NotificationMessage::PeerJoined(PeerJoinedNotification {
                                peer_id: event.peer_id().to_string(),
                                topic: event.topic().to_string(),
                                timestamp: *event.timestamp(),
                                leave: true,
                            }),
                        );
                        chats_cloned.add_message(event.topic(), message).await;
                    }
                }
            }
        });

        Self { chats, peer }
    }

    pub async fn start(&mut self) {
        let mut stdin = BufReader::new(tokio::io::stdin()).lines();

        while let Some(line) = stdin.next_line().await.unwrap() {
            match line.split_once(" ") {
                Some(("/join", room)) => match self.peer.subscribe_topic(room.to_string()).await {
                    Ok(_) => {
                        self.chats.add_room(room).await;
                        println!("Joined room: {}", room);
                    }
                    Err(e) => {
                        log::error!("Failed to join room: {}", e);
                    }
                },
                Some(("/leave", room)) => match self.peer.unsubscribe_topic(room.to_string()).await
                {
                    Ok(_) => {
                        self.chats.remove_room(room).await;
                        println!("Left room: {}", room);
                    }
                    Err(e) => {
                        log::error!("Failed to leave room: {}", e);
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

                                self.chats.add_message(room, message).await;
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

                    let messages = self.chats.get_messages(room).await;
                    for message in messages {
                        println!("{}", message);
                    }
                }
                _ => {
                    log::error!("Unknown command");
                }
            }
        }
    }
}
