use crate::recipes::Recipes;
use command::Command;
use futures::StreamExt;
use libp2p::mdns::{Config as MdsnConfig, tokio::Behaviour as MdsnBehaviour};
use libp2p::{Multiaddr, PeerId, SwarmBuilder, identity, noise, tcp, yamux};
use libp2p::{
    floodsub::{self, Topic},
    identity::Keypair,
};
use libp2p_swarm_derive::NetworkBehaviour;
use log::*;
use message::{ListMode, ListResponse, RecipeMessage};
use std::collections::HashSet;
use std::error::Error;
use std::path::PathBuf;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;

pub mod command;
pub mod message;

pub struct Peer {
    pub recipes: Recipes,
    pub swarm: libp2p::Swarm<RecipeBehaviour>,
    pub topic: Topic,
    pub peer_id: PeerId,
}

impl Peer {
    pub async fn new(
        topic: Topic,
        recipes_path: &str,
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'static>> {
        let recipes = Recipes::new(PathBuf::from(recipes_path)).await;
        let key_pair = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(key_pair.public());
        let swarm = SwarmBuilder::with_existing_identity(key_pair.clone())
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|k| RecipeBehaviour::create(&k, &topic))?
            .with_swarm_config(|cfg| {
                cfg.with_idle_connection_timeout(Duration::from_secs(u64::MAX))
            })
            .build();

        Ok(Self {
            recipes,
            swarm,
            topic,
            peer_id,
        })
    }

    fn respond_with_public_recipes(
        &self,
        sender: mpsc::UnboundedSender<ListResponse>,
        receiver: String,
    ) {
        let recipes = self.recipes.clone();
        tokio::spawn(async move {
            match recipes.load_recipes() {
                Ok(recipes) => {
                    let resp = ListResponse {
                        mode: ListMode::ALL,
                        receiver,
                        data: recipes.into_iter().filter(|r| r.public).collect(),
                    };
                    if let Err(e) = sender.send(resp) {
                        error!("error sending response via channel, {}", e);
                    }
                }
                Err(e) => log::error!("error fetching local recipes to answer ALL request, {}", e),
            }
        });
    }

    async fn event_loop(&mut self) {
        let mut stdin = BufReader::new(tokio::io::stdin()).lines();
        let (response_sender, mut response_rcv) = mpsc::unbounded_channel::<ListResponse>();
        loop {
            tokio::select! {
                line = stdin.next_line() => {
                    match line {
                        Ok(Some(line)) => {
                            let cmd = Command::from(line.as_str());
                            if let Err(e) =  cmd.execute(self.topic.clone(), &self.recipes, self.swarm.behaviour_mut() ).await{
                                error!("Error executing command: {:?}", e);
                            }
                        }
                        Ok(None) => continue,
                        Err(e) => error!("Error reading from stdin: {:?}", e),
                    }

                }
                response = response_rcv.recv() => {
                    if let Some(response) = response {
                        let message = RecipeMessage::Response(response);
                        match message.serialize_to_bytes(){
                            Ok(data) => self.swarm.behaviour_mut().publish(self.topic.clone(), data),
                            Err(e) => error!("Error serializing message: {:?}", e),
                        }
                    }
                }
                swarm_event = self.swarm.next() => {
                    match swarm_event {
                        Some(libp2p::swarm::SwarmEvent::Behaviour(e)) => match e {
                            RecipeBehaviourEvent::Floodsub(e) => match e {
                                floodsub::FloodsubEvent::Message(msg) => {
                                    match RecipeMessage::deserialize_from_bytes(&msg.data){
                                        Ok(RecipeMessage::Request(req)) => {
                                            match req.mode{
                                                ListMode::ALL => {
                                                    info!("Received ALL req: {:?} from {:?}", req, msg.source);
                                                    self.respond_with_public_recipes(
                                                        response_sender.clone(),
                                                        msg.source.to_string(),

                                                );

                                                },
                                                ListMode::One(ref peer_id) => {
                                                    if peer_id == &self.peer_id.to_string() {
                                                        info!("Received req: {:?} from {:?}", req, msg.source);
                                                        self.respond_with_public_recipes(
                                                            response_sender.clone(),
                                                            msg.source.to_string(),
                                                        );
                                                    }
                                                },
                                            }
                                        },
                                        Ok(RecipeMessage::Response(response)) => {
                                            if response.receiver == self.peer_id.to_string() {
                                                info!("Received from: {:?}", msg.source);
                                                response.data.iter().for_each(|r| info!("{:?}", r));
                                            }
                                        },
                                        Err(e) => info!("Error deserializing message: {:?}", e),
                                    }
                                }
                                _ => info!("Event: {:?}", e),
                            },
                            RecipeBehaviourEvent::Mdns(e) => match e {
                                libp2p::mdns::Event::Discovered(items) => {
                                    for (peer, _) in items {
                                        self.swarm.behaviour_mut().add_to_floodsub(peer);
                                    }
                                }
                                libp2p::mdns::Event::Expired(items) => {
                                    for (peer, _) in items {
                                        self.swarm.behaviour_mut().remove_from_floodsub(peer);
                                    }
                                }
                            },
                        },
                        _ => {},
                    }
                }
            }
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        let addr: Multiaddr = "/ip4/0.0.0.0/tcp/0".parse().unwrap();
        self.swarm.listen_on(addr)?;
        self.event_loop().await;
        Ok(())
    }
}

#[derive(NetworkBehaviour)]
pub struct RecipeBehaviour {
    pub floodsub: floodsub::Floodsub,
    pub mdns: MdsnBehaviour,
}

impl RecipeBehaviour {
    fn create(keypair: &Keypair, topic: &Topic) -> Self {
        let mut floodsub = floodsub::Floodsub::new(keypair.public().into());
        floodsub.subscribe(topic.clone());

        let mdns = MdsnBehaviour::new(MdsnConfig::default(), keypair.public().into()).unwrap();
        Self { floodsub, mdns }
    }

    pub fn nodes(&self) -> impl Iterator<Item = &PeerId> {
        self.mdns
            .discovered_nodes()
            .into_iter()
            .collect::<HashSet<_>>()
            .into_iter()
    }

    fn add_to_floodsub(&mut self, peer: PeerId) {
        info!("Adding peer to floodsub: {}", peer);
        self.floodsub.add_node_to_partial_view(peer);
    }

    fn remove_from_floodsub(&mut self, peer: PeerId) {
        info!("Removing peer from floodsub: {}", peer);
        self.floodsub.remove_node_from_partial_view(&peer);
    }

    pub fn publish(&mut self, topic: Topic, data: Vec<u8>) {
        self.floodsub.publish(topic, data);
    }
}
