use crab_chat_peer::Peer;
use tui_textarea::TextArea;

use std::error;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
pub struct App<'a> {
    /// Is the application running?
    pub running: bool,
    /// counter
    pub peer: Peer,

    pub input: TextArea<'a>,
}

impl<'a> App<'a> {
    /// Constructs a new instance of [`App`].
    pub fn new(peer: Peer) -> Self {
        Self {
            running: true,
            peer,
            input: TextArea::default(),
        }
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}
