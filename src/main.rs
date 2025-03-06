use command::Command;
use futures::StreamExt;
use libp2p::{
    PeerId, SwarmBuilder,
    floodsub::{self, Topic},
    identity::{self, Keypair},
    mdns::{Config as MdsnConfig, tokio::Behaviour as MdsnBehaviour},
    noise, tcp, yamux,
};
use libp2p_swarm_derive::NetworkBehaviour;
use log::info;
use recipes::{Recipe, recipes};
use serde::{Deserialize, Serialize};
use std::{error::Error, str::FromStr, sync::LazyLock, time::Duration};
use tokio::{io::AsyncBufReadExt, sync::mpsc};
use tracing_subscriber::EnvFilter;

pub mod command;
pub mod recipes;

static KEYS: LazyLock<Keypair> = LazyLock::new(|| identity::Keypair::generate_ed25519());
static PEER_ID: LazyLock<PeerId> = LazyLock::new(|| PeerId::from(KEYS.public()));
static TOPIC: LazyLock<Topic> = LazyLock::new(|| Topic::new("recipes"));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    Command(crate::command::Command),
    Response(ListResponse),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum ListMode {
    ALL,
    One(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResponse {
    mode: ListMode,
    data: Vec<Recipe>,
    receiver: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ListRequest {
    mode: ListMode,
}

#[derive(NetworkBehaviour)]
pub struct RecipeBehaviour {
    floodsub: floodsub::Floodsub,
    mdns: MdsnBehaviour,
}

impl RecipeBehaviour {
    fn create(keypair: &Keypair) -> Self {
        let mut floodsub = floodsub::Floodsub::new(keypair.public().into());
        floodsub.subscribe(TOPIC.clone());

        let mdns = MdsnBehaviour::new(MdsnConfig::default(), keypair.public().into()).unwrap();
        Self { floodsub, mdns }
    }

    fn add_to_floodsub(&mut self, peer: PeerId) {
        info!("Adding peer to floodsub: {}", peer);
        self.floodsub.add_node_to_partial_view(peer);
    }

    fn remove_from_floodsub(&mut self, peer: PeerId) {
        info!("Removing peer from floodsub: {}", peer);
        self.floodsub.remove_node_from_partial_view(&peer);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Sync + Send + 'static>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    info!("Peer Id: {}", PEER_ID.clone());
    let (response_sender, mut response_rcv) = mpsc::unbounded_channel::<ListResponse>();

    let mut swarm = SwarmBuilder::with_existing_identity(KEYS.clone())
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|k| RecipeBehaviour::create(&k))?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(u64::MAX)))
        .build();

    swarm
        .listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap())
        .unwrap();

    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();

    loop {
        tokio::select! {
            line = stdin.next_line() => {
                if let Ok(Some(line)) = line {
                    let cmd = Command::from_str(&line);
                    if let Ok(cmd) = cmd {
                        cmd.execute(&mut swarm).await?;
                    }
                }
            }
            response = response_rcv.recv() => {
                if let Some(response) = response {
                    let json = serde_json::to_string(&response).expect("can jsonify response");
                    swarm.behaviour_mut().floodsub.publish(TOPIC.clone(), json);
                }
            }
            swarm_event = swarm.next() => {
                match swarm_event {
                    Some(libp2p::swarm::SwarmEvent::Behaviour(e)) => match e {
                        RecipeBehaviourEvent::Floodsub(e) => match e {
                            floodsub::FloodsubEvent::Message(msg) => {
                                if let Ok(response) = serde_json::from_slice::<ListResponse>(&msg.data){
                                    if response.receiver == PEER_ID.to_string() {
                                        info!("Received from: {:?}", msg.source);
                                        response.data.iter().for_each(|r| info!("{:?}", r));
                                    }
                                } else if let Ok(req) = serde_json::from_slice::<ListRequest>(&msg.data){
                                    match req.mode{
                                        ListMode::ALL => {
                                            info!("Received ALL req: {:?} from {:?}", req, msg.source);
                                            respond_with_public_recipes(
                                                response_sender.clone(),
                                                msg.source.to_string(),
                                        );

                                        },
                                        ListMode::One(ref peer_id) => {
                                            if peer_id == &PEER_ID.to_string() {
                                                info!("Received req: {:?} from {:?}", req, msg.source);
                                                respond_with_public_recipes(
                                                    response_sender.clone(),
                                                    msg.source.to_string(),
                                                );
                                            }
                                        },
                                    }
                                }

                            }
                            _ => (),
                        },
                        RecipeBehaviourEvent::Mdns(e) => match e {
                            libp2p::mdns::Event::Discovered(items) => {
                                for (peer, _) in items {
                                    swarm.behaviour_mut().add_to_floodsub(peer);
                                }
                            }
                            libp2p::mdns::Event::Expired(items) => {
                                for (peer, _) in items {
                                    swarm.behaviour_mut().remove_from_floodsub(peer);
                                }
                            }
                        },
                    },
                    _ => info!("Event: {:?}", swarm_event),
                }
            }
        }
    }
}

fn respond_with_public_recipes(sender: mpsc::UnboundedSender<ListResponse>, receiver: String) {
    tokio::spawn(async move {
        match recipes().await.load_recipes() {
            Ok(recipes) => {
                let resp = ListResponse {
                    mode: ListMode::ALL,
                    receiver,
                    data: recipes.into_iter().filter(|r| r.public).collect(),
                };
                if let Err(e) = sender.send(resp) {
                    panic!("error sending response via channel, {}", e);
                    //log::error!("error sending response via channel, {}", e);
                }
            }
            Err(e) => log::error!("error fetching local recipes to answer ALL request, {}", e),
        }
    });
}
