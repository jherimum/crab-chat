use std::error::Error;
use clap::Parser;
use cli::Cli;
use crab_chat_peer::{BootstrapAddress, Peer, PeerConfig};
use crab_chat_ui::ui;
use libp2p::identity::Keypair;
use tap::TapFallible;
use tracing_subscriber::EnvFilter;

pub mod cli;

const DEFAULT_ADDR: &str = "/ip4/0.0.0.0/tcp/0";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    // let _ = tracing_subscriber::fmt()
    //     .with_env_filter(EnvFilter::from_default_env())
    //     .try_init();

    let cli = Cli::parse();
    let addr = cli
        .addr
        .unwrap_or(DEFAULT_ADDR.to_owned())
        .parse()
        .tap_err(|e| log::error!("Failed to parse address: {e}"))?;
    let bootstrap = cli
        .bootstrap
        .into_iter()
        .map(|addr| addr.parse::<BootstrapAddress>())
        .collect::<Result<_, _>>()?;

    let keypair = Keypair::generate_ed25519();
    let peer = Peer::new(PeerConfig::new(addr, bootstrap, keypair))?;

    ui(peer).await?;

    Ok(())
}
