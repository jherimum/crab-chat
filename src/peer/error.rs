use libp2p::{
    TransportError,
    gossipsub::{PublishError, SubscriptionError},
};
use tokio::io;

#[derive(Debug, thiserror::Error)]
pub enum PeerError {
    #[error("Failed to create swarm: {0}")]
    SwarmError(Box<dyn std::error::Error + Send + Sync>),

    #[error("Failed to parse bootstrap address: {0}")]
    InvalidBootstrapError(Box<dyn std::error::Error + Send + Sync>),

    #[error("Failed to send command: {0}")]
    CommandResponseError(#[from] tokio::sync::oneshot::error::RecvError),

    #[error("Failed to send event: {0}")]
    TransportError(#[from] TransportError<io::Error>),

    #[error("Failed to send event: {0}")]
    EventError(#[from] PublishError),

    #[error("Failed to send event: {0}")]
    SubscribeError(#[from] SubscriptionError),
}
