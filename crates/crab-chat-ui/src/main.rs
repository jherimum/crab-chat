use clap::Parser;
use cli::Cli;
use color_eyre::Result;
use crab_chat_peer::create_peer;
use crate::app::App;

mod action;
mod app;
mod cli;
mod components;
mod config;
mod errors;
mod logging;
mod tui;

#[tokio::main]
async fn main() -> Result<()> {
    crate::errors::init()?;
    crate::logging::init()?;

    let args = Cli::parse();
    let mut app = App::new(args.tick_rate, args.frame_rate, create_peer())?;
    app.run().await?;
    Ok(())
}
