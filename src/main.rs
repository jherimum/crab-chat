use std::time::Duration;

use clap::Parser;
use crab_chat::App;
use crab_chat::BootstrapAddress;
use crab_chat::Cli;
use crab_chat::Peer;
use crab_chat::PeerConfig;
use tap::TapFallible;
use tokio::time::sleep;
use tracing_subscriber::EnvFilter;

const DEFAULT_ADDR: &str = "/ip4/0.0.0.0/tcp/0";

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::dotenv().ok();
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

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

    let peer = Peer::new(PeerConfig::new(addr, bootstrap))?;
    let listener = peer.subscribe();
    let command_bus = peer.run().await?;

    let app = App::new(command_bus, listener);
    app.start().await;

    loop {
        sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}
