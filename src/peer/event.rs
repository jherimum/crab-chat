use tokio::sync::broadcast;

#[derive(Clone, Debug)]
pub enum PeerEvent {
    Started,
}

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
