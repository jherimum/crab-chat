use std::{env, error::Error};

use futures::StreamExt;
use libp2p::{
    Multiaddr, PeerId, SwarmBuilder, gossipsub, identity,
    kad::{self, store::MemoryStore},
    mdns, noise, ping,
    swarm::SwarmEvent,
    tcp,
};
use libp2p_swarm_derive::NetworkBehaviour;
use tracing_subscriber::EnvFilter;

// We create a custom network behaviour that combines Gossipsub and Mdns.
#[derive(NetworkBehaviour)]
struct MyBehaviour {
    kad: kad::Behaviour<MemoryStore>,
    ping: ping::Behaviour,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    let keypair = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(keypair.public());
    println!("🌐 Local peer id: {peer_id}");

    let mut swarm = SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            libp2p::yamux::Config::default,
        )
        .unwrap()
        .with_behaviour(|k| MyBehaviour {
            kad: kad::Behaviour::new(
                PeerId::from(k.public()),
                MemoryStore::new(PeerId::from(k.public())),
            ),
            ping: ping::Behaviour::new(ping::Config::default()),
        })
        .unwrap()
        .build();

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    if let Some(add) = env::args().nth(1) {
        let addr: Multiaddr = add.parse()?;
        swarm.dial(addr.clone())?;
        println!("👥 Attempting to dial {addr}");
    }

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("📡 Listening on {address}");
            }
            SwarmEvent::Behaviour(MyBehaviourEvent::Ping(e)) => {
                println!("🔄 Ping event occurred: {e:?}");
            }
            _ => {}
        }
    }
}
