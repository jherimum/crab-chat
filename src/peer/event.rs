use bon::Builder;
use derive_getters::Getters;
use tokio::sync::broadcast;

#[derive(Clone, Debug)]
pub enum PeerEvent {
    MessageReceived(MessageReceivedEvent),
    PeerJoined(PeerJoinedEvent),
    PeerLeft(PeerLeftEvent),
}

#[derive(Clone, Debug, Getters, Builder)]
pub struct PeerLeftEvent {
    peer_id: String,
    topic: String,
    timestamp: u64,
}

#[derive(Clone, Debug, Getters, Builder)]
pub struct PeerJoinedEvent {
    peer_id: String,
    topic: String,
    timestamp: u64,
}

#[derive(Clone, Debug, Getters, Builder)]
pub struct MessageReceivedEvent {
    message_id: String,
    message: String,
    timestamp: u64,
    topic: String,
    peer_id: String,
}

#[derive(Clone, Debug)]
pub struct PeerEventBus {
    sender: broadcast::Sender<PeerEvent>,
}

impl PeerEventBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(100);
        Self { sender }
    }

    pub fn subscribe(&self) -> PeerEventListener {
        PeerEventListener {
            receiver: self.sender.subscribe(),
        }
    }

    pub fn emit(&self, event: PeerEvent) {
        self.sender.send(event).unwrap();
    }
}

pub struct PeerEventListener {
    receiver: broadcast::Receiver<PeerEvent>,
}

impl PeerEventListener {
    pub fn new(receiver: broadcast::Receiver<PeerEvent>) -> Self {
        Self { receiver }
    }

    pub async fn recv(&mut self) -> Result<PeerEvent, broadcast::error::RecvError> {
        self.receiver.recv().await
    }
}
