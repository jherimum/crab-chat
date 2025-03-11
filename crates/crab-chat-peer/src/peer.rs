use super::command::{SendMessageCommand, UnsubscribeCommand};
use super::event::{PeerEventBus, PeerEventListener};
use super::message::Message;
use super::{
    BootstrapAddress, PeerCommandBus, PeerError, PeerResult, SubscribeCommand,
};
use crate::PeerEvent;
use crate::command::PeerCommand;
use crate::event::{MessageReceivedEvent, PeerJoinedEvent, PeerLeftEvent};
use chrono::Utc;
use futures::StreamExt;
use libp2p::gossipsub::{IdentTopic, MessageId, PublishError, SubscriptionError};
use libp2p::identity::Keypair;
use libp2p::mdns::{Config as MdsnConfig, tokio::Behaviour as MdsnBehaviour};
use libp2p::swarm::SwarmEvent;
use libp2p::{
    Multiaddr, PeerId, Swarm, SwarmBuilder, gossipsub, mdns, noise, tcp, yamux,
};
use libp2p_swarm_derive::NetworkBehaviour;
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct Peer {
    event_bus: PeerEventBus,
    command_bus: PeerCommandBus,
    peer_id: PeerId,
}

impl Peer {
    pub fn peer_id(&self) -> &PeerId {
        &self.peer_id
    }

    pub fn new(config: PeerConfig) -> PeerResult<Self> {
        let peer_id = config.keypair.public().to_peer_id();
        log::info!("Starting peer: {}", peer_id);
        let (command_bus_tx, command_bus_rx) =
            tokio::sync::mpsc::unbounded_channel();

        let mut listeners = Vec::new();
        let mut swarm: Swarm<PeerBehaviour> =
            SwarmBuilder::with_existing_identity(config.keypair.clone())
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
                    cfg.with_idle_connection_timeout(Duration::from_secs(
                        u64::MAX,
                    ))
                })
                .build();
        listeners.push(swarm.listen_on(config.addr.clone())?);

        let event_bus = PeerEventBus::new();
        tokio::spawn(swarm_loop(
            peer_id,
            swarm,
            command_bus_rx,
            event_bus.clone(),
        ));
        Ok(Self {
            event_bus,
            command_bus: PeerCommandBus::new(command_bus_tx),
            peer_id,
        })
    }

    pub async fn subscribe_topic(&self, topic: String) -> PeerResult<bool> {
        self.command_bus
            .send(SubscribeCommand::builder().topic(topic).build())
            .await
    }

    pub async fn unsubscribe_topic(&self, topic: String) -> PeerResult<bool> {
        self.command_bus
            .send(UnsubscribeCommand::builder().topic(topic).build())
            .await
    }

    pub async fn send_message(
        &self,
        message: String,
        topic: String,
    ) -> PeerResult<MessageId> {
        self.command_bus
            .send(
                SendMessageCommand::builder()
                    .message(message)
                    .topic(topic)
                    .timestamp(Utc::now().timestamp() as u64)
                    .build(),
            )
            .await
    }

    pub fn subscribe(&self) -> PeerEventListener {
        self.event_bus.subscribe()
    }
}

async fn swarm_loop(
    local_peer_id: PeerId,
    mut swarm: Swarm<PeerBehaviour>,
    mut command_bus_rx: mpsc::UnboundedReceiver<PeerCommand>,
    event_bus: PeerEventBus,
) -> PeerResult<()> {
    loop {
        tokio::select! {
            event =  swarm.select_next_some() => {
                match event {
                    SwarmEvent::Behaviour(PeerBehaviourEvent::Mdns(mdns::Event::Discovered(items))) => {
                        for (peer, addr ) in items {
                            swarm.behaviour_mut().kad.add_address(&peer, addr);
                        }
                    },
                    SwarmEvent::Behaviour(PeerBehaviourEvent::Gossip(gossipsub::Event::Message { propagation_source, message_id, message })) => {
                        let mesage = serde_json::from_slice::<Message>(&message.data).unwrap();
                        let event = MessageReceivedEvent::builder()
                            .message_id(message_id.to_string())
                            .message(mesage.data().to_string())
                            .timestamp(*mesage.timestamp())
                            .topic(mesage.topic().to_string())
                            .peer_id(propagation_source.to_string())
                            .build();
                        event_bus.emit(PeerEvent::MessageReceived(event));
                    },

                    SwarmEvent::Behaviour(PeerBehaviourEvent::Gossip(gossipsub::Event::Subscribed { peer_id, topic })) => {
                            event_bus.emit(PeerEvent::PeerJoined(PeerJoinedEvent::builder()
                                .peer_id(peer_id.to_string())
                                .topic(topic.to_string())
                                .timestamp(Utc::now().timestamp() as u64)
                                .build()));

                    },

                    SwarmEvent::Behaviour(PeerBehaviourEvent::Gossip(gossipsub::Event::Unsubscribed { peer_id, topic })) => {
                            event_bus.emit(PeerEvent::PeerLeft(PeerLeftEvent::builder()
                                .peer_id(peer_id.to_string())
                                .topic(topic.to_string())
                                .timestamp(Utc::now().timestamp() as u64)
                                .build()));

                    },
                    _ => {},
                }
            }
            cmd = command_bus_rx.recv() => {
                if let Some(cmd) = cmd {
                    match cmd {
                        PeerCommand::SendMessage(command)=>{
                            let r = swarm.behaviour_mut().publish_message(command.as_ref());
                            command.send(r.map_err(PeerError::from));

                        },
                        PeerCommand::Subscribe(cmd) => {
                            let response = swarm.behaviour_mut().subscribe(cmd.as_ref());
                            cmd.send(response.map_err(PeerError::from));

                        },
                        PeerCommand::Unsubscribe(cmd) => {
                            let response = swarm.behaviour_mut().unsubscribe(cmd.as_ref());
                            cmd.send(Ok(response));
                        },
                    }
                }
            }
        }
    }
}

pub struct PeerConfig {
    pub addr: Multiaddr,
    pub bootstrap: Vec<BootstrapAddress>,
    pub keypair: Keypair,
}

impl PeerConfig {
    pub fn new(
        addr: Multiaddr,
        bootstrap: Vec<BootstrapAddress>,
        keypair: Keypair,
    ) -> Self {
        Self {
            addr,
            bootstrap,
            keypair,
        }
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
        let mdns =
            MdsnBehaviour::new(MdsnConfig::default(), local_peer_id).unwrap();

        let mut kad = libp2p::kad::Behaviour::new(
            local_peer_id,
            libp2p::kad::store::MemoryStore::new(local_peer_id),
        );

        bootstrap.iter().for_each(|b| {
            kad.add_address(&b.peer_id, b.addr.clone());
        });

        Self { gossip, mdns, kad }
    }

    pub fn subscribe(
        &mut self,
        commad: &SubscribeCommand,
    ) -> Result<bool, SubscriptionError> {
        let topic = IdentTopic::new(commad.topic());
        self.gossip.subscribe(&topic)
    }

    pub fn unsubscribe(&mut self, commad: &UnsubscribeCommand) -> bool {
        let topic = IdentTopic::new(commad.topic());
        self.gossip.unsubscribe(&topic)
    }

    pub fn publish_message(
        &mut self,
        command: &SendMessageCommand,
    ) -> Result<MessageId, PublishError> {
        let message = Message::builder()
            .data(command.message().clone())
            .timestamp(*command.timestamp())
            .topic(command.topic().clone())
            .build();
        let data = serde_json::to_vec(&message).unwrap();

        self.gossip.publish(IdentTopic::new(command.topic()), data)
    }
}
