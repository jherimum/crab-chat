use std::str::FromStr;

use libp2p::{Multiaddr, PeerId};

use super::PeerError;

pub struct BootstrapAddress {
    pub addr: Multiaddr,
    pub peer_id: libp2p::PeerId,
}

impl BootstrapAddress {
    pub fn new(addr: Multiaddr, peer_id: libp2p::PeerId) -> Self {
        Self { addr, peer_id }
    }
}

impl FromStr for BootstrapAddress {
    type Err = PeerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(2, ':');

        let peer_id = match parts.next() {
            Some(peer_id) => Ok(PeerId::from_str(peer_id)
                .map_err(|e| PeerError::InvalidBootstrapError(e.into()))?),
            None => Err(PeerError::InvalidBootstrapError("Missing peer id".into())),
        }?;

        let addr = match parts.next() {
            Some(addr) => Ok(Multiaddr::from_str(addr)
                .map_err(|e| PeerError::InvalidBootstrapError(e.into()))?),
            None => Err(PeerError::InvalidBootstrapError("Missing address".into())),
        }?;

        Ok(Self::new(addr, peer_id))
    }
}
