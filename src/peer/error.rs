#[derive(Debug, thiserror::Error)]
pub enum PeerError {
    #[error("Failed to create swarm: {0}")]
    SwarmError(Box<dyn std::error::Error + Send + Sync>),

    #[error("Failed to parse bootstrap address: {0}")]
    InvalidBootstrapError(Box<dyn std::error::Error + Send + Sync>),

    #[error("Failed to send command: {0}")]
    CommandResponseError(#[from] tokio::sync::oneshot::error::RecvError),
}
