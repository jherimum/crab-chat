mod bootstrap_address;
mod command;
mod error;
mod event;
mod message;
mod peer;

pub type PeerResult<T> = Result<T, PeerError>;

pub use bootstrap_address::BootstrapAddress;
pub use command::PeerCommandBus;
pub use command::SubscribeCommand;
pub use command::UnsubscribeCommand;
pub use command::SendMessageCommand;
pub use error::PeerError;
pub use event::PeerEvent;
pub use event::PeerEventListener;
use libp2p::identity::Keypair;
pub use peer::Peer;
pub use peer::PeerConfig;

pub fn create_peer() -> Peer {
    let keypair = Keypair::generate_ed25519();
    let cfg =
        PeerConfig::new("/ip4/0.0.0.0/tcp/0".parse().unwrap(), vec![], keypair);
    Peer::new(cfg).unwrap()
}
