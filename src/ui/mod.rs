use app::App;
use event::{Event, EventHandler};
use handler::handle_key_events;
use ratatui::{Terminal, prelude::CrosstermBackend};
use std::{error::Error, io};
use tui::Tui;

pub mod app;
pub mod event;
pub mod handler;
pub mod tui;
pub mod ui;

pub async fn ui() -> Result<(), Box<dyn Error>> {
    // Create an application.
    let mut app = App::new();

    // Initialize the terminal user interface.
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
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;

    Ok(())
}
