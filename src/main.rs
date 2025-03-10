use std::error::Error;
use std::io;

use clap::Parser;
use crab_chat::BootstrapAddress;
use crab_chat::Cli;
use crab_chat::Peer;
use crab_chat::PeerConfig;
use crab_chat::event::EventHandler;
use crab_chat::handler::handle_key_events;
use crab_chat::tui::Tui;
use crab_chat::ui;
use crab_chat::ui::app::App;
use libp2p::identity::Keypair;
use ratatui::Terminal;
use ratatui::prelude::CrosstermBackend;
use tap::TapFallible;
use tracing_subscriber::EnvFilter;

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
    //let mut app = App::new(peer);

    let mut app = App::new(peer);
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next().await? {
            ui::event::Event::Tick => app.tick(),
            ui::event::Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            ui::event::Event::Mouse(_) => {}
            ui::event::Event::Resize(_, _) => {}
        }
    }

    tui.exit()?;

    // tokio::select! {
    //     _ = app.start() => {
    //         log::info!("App finished");
    //     }
    // }

    Ok(())
}
