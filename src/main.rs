use std::error::Error;

use libp2p::floodsub::Topic;
use peer::Peer;
use tracing_subscriber::EnvFilter;

pub mod peer;
pub mod recipes;
pub mod ui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Sync + Send + 'static>> {
    dotenvy::dotenv().ok();
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    load_perr().await;

    Ok(())
}

async fn load_perr() {
    let mut peer = Peer::new(Topic::new("recipes"), "./recipes.json")
        .await
        .unwrap();

    tokio::spawn(async move {
        peer.run().await.unwrap();
    })
    .await
    .unwrap();
}
