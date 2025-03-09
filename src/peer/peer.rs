use super::event::{PeerEventBus, PeerEventListener};
use super::{BootstrapAddress, PeerCommandBus, PeerError, PeerResult};
use crate::peer::command::PeerCommand;
use crate::peer::event::PeerEvent;
use futures::StreamExt;
use libp2p::identity::Keypair;
use libp2p::mdns::{Config as MdsnConfig, tokio::Behaviour as MdsnBehaviour};
use libp2p::swarm::SwarmEvent;
use libp2p::{Multiaddr, Swarm, SwarmBuilder, gossipsub, mdns, noise, tcp, yamux};
use libp2p_swarm_derive::NetworkBehaviour;
use std::time::Duration;

pub struct PeerConfig {
    pub addr: Multiaddr,
    pub bootstrap: Vec<BootstrapAddress>,
}

impl PeerConfig {
    pub fn new(addr: Multiaddr, bootstrap: Vec<BootstrapAddress>) -> Self {
        Self { addr, bootstrap }
    }
}

#[derive(NetworkBehaviour)]
pub struct PeerBehaviour {
    pub gossip: gossipsub::Behaviour,
    pub mdns: MdsnBehaviour,
    pub kad: libp2p::kad::Behaviour<libp2p::kad::store::MemoryStore>,
}

impl PeerBehaviour {
    pub fn new(keypair: &Keypair, bootstrap: Vec<BootstrapAddress>) -> Self {
        let local_peer_id = keypair.public().to_peer_id();
        let gossip_config = gossipsub::Config::default();
        let gossip = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(keypair.clone()),
            gossip_config,
        )
        .unwrap();
        let mdns = MdsnBehaviour::new(MdsnConfig::default(), local_peer_id).unwrap();

        let mut kad = libp2p::kad::Behaviour::new(
            local_peer_id,
            libp2p::kad::store::MemoryStore::new(local_peer_id),
        );

        bootstrap.iter().for_each(|b| {
            kad.add_address(&b.peer_id, b.addr.clone());
        });

        Self { gossip, mdns, kad }
    }
}

pub struct Peer {
    swarm: Swarm<PeerBehaviour>,
    addr: Multiaddr,
    event_bus: PeerEventBus,
}

impl Peer {
    pub fn new(config: PeerConfig) -> PeerResult<Self> {
        let swarm: Swarm<PeerBehaviour> = SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )
            .map_err(|e| PeerError::SwarmError(e.into()))?
            .with_behaviour(|k| PeerBehaviour::new(k, config.bootstrap))
            .map_err(|e| PeerError::SwarmError(e.into()))?
            .with_swarm_config(|cfg| {
                cfg.with_idle_connection_timeout(Duration::from_secs(u64::MAX))
            })
            .build();
        Ok(Self {
            swarm,
            addr: config.addr,
            event_bus: PeerEventBus::new(),
        })
    }

    pub fn subscribe(&self) -> PeerEventListener {
        self.event_bus.subscribe()
    }

    /***
     * Run the peer on a separated task and return a PeerCommandBus
     * that can be used to send commands to the peer
     */
    pub async fn run(mut self) -> PeerResult<PeerCommandBus> {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    event =  self.swarm.select_next_some() => {
                        match event {
                            SwarmEvent::Behaviour(PeerBehaviourEvent::Gossip(e)) => {
                                println!("Gossip event: {:?}", e);
                            },
                            SwarmEvent::Behaviour(PeerBehaviourEvent::Mdns(mdns::Event::Discovered(items))) => {
                                for (peer, addr ) in items {
                                    self.swarm.behaviour_mut().kad.add_address(&peer, addr);
                                }

                            },
                            SwarmEvent::Behaviour(PeerBehaviourEvent::Kad(e)) => {
                                println!("Kad event: {:?}", e);
                            },
                            _ => log::info!("Event: {:?}", event),
                        }
                    }
                    cmd = rx.recv() => {
                        if let Some(cmd) = cmd {
                            match cmd {
                                PeerCommand::SendMessage(command)=>{
                                    println!("Sending message: {:?}",command);
                                },
                                PeerCommand::Start(_) => {
                                    self.swarm.listen_on(self.addr.clone()).unwrap();
                                    self.event_bus.emit(PeerEvent::Started);
                                },
                            }
                        }
                    }
                }
            }
        });

        Ok(PeerCommandBus::new(tx))
    }
}
