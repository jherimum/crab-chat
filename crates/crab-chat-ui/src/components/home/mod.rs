use std::collections::HashMap;
use chat::ChatWidget;
use color_eyre::Result;
use crab_chat_peer::{Peer, PeerEventListener};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use header::HeaderWidget;
use models::Room;
use ratatui::{prelude::*, widgets::*};
use rooms::RoomsWidget;
use tokio::sync::mpsc::UnboundedSender;
use super::Component;
use crate::{action::Action, config::Config};

mod chat;
mod header;
mod models;
mod rooms;

async fn handle_peer_events(
    mut event_listener: PeerEventListener,
    command_tx: UnboundedSender<Action>,
) {
    loop {
        match event_listener.recv().await {
            Ok(_) => todo!(),
            Err(_) => todo!(),
        }
    }
}

#[derive(Default)]
pub enum Mode {
    Chat,
    #[default]
    Rooms,
}

pub struct Home {
    peer: Peer,
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    rooms: HashMap<String, Room>,
    actual_room: Option<String>,
    mode: Mode,
    rooms_state: ListState,
}

impl Home {
    pub fn new(peer: Peer) -> Self {
        Self {
            command_tx: None,
            config: Config::default(),
            rooms: HashMap::new(),
            actual_room: None,
            peer,
            mode: Default::default(),
            rooms_state: ListState::default(),
        }
    }

    fn room_navigate(&mut self, up: bool) {
        match up {
            true => self.rooms_state.select_previous(),
            false => self.rooms_state.select_next(),
        }
    }

    fn chnage_focus(&mut self) {
        match &self.mode {
            Mode::Chat => self.mode = Mode::Rooms,
            Mode::Rooms => self.mode = Mode::Chat,
        }
    }
}

impl Component for Home {
    fn init(&mut self, _: Size) -> Result<()> {
        let listener = self.peer.subscribe();
        tokio::spawn(handle_peer_events(
            listener,
            self.command_tx.clone().unwrap(),
        ));

        self.rooms
            .entry("sala 1".to_string())
            .insert_entry(Default::default());

        self.rooms
            .entry("sala 2".to_string())
            .insert_entry(Default::default());

        self.rooms_state.select_first();
        Ok(())
    }

    fn register_action_handler(
        &mut self,
        tx: UnboundedSender<Action>,
    ) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        match (&self.mode, key.code, key.modifiers) {
            (Mode::Rooms, KeyCode::Down, KeyModifiers::NONE) => {
                self.room_navigate(false);
            }
            (Mode::Rooms, KeyCode::Up, KeyModifiers::NONE) => {
                self.room_navigate(true);
            }
            (_, KeyCode::Tab, KeyModifiers::NONE) => {
                self.chnage_focus();
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let (header, main, footer) = vertical_layout(area);
        let (rooms, chat_panel) = horizontal_layout(main);
        let (chat, input, participants) = chat_layout(chat_panel);
        frame.render_widget(HeaderWidget, header);
        frame.render_stateful_widget(RoomsWidget, rooms, self);
        frame.render_stateful_widget(ChatWidget, chat_panel, self);

        Ok(())
    }
}

fn vertical_layout(area: Rect) -> (Rect, Rect, Rect) {
    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(area);

    (areas[0], areas[1], areas[2])
}

fn horizontal_layout(area: Rect) -> (Rect, Rect) {
    let areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(20), Constraint::Min(1)])
        .split(area);

    (areas[0], areas[1])
}

fn chat_layout(area: Rect) -> (Rect, Rect, Rect) {
    let hr = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Min(1), Constraint::Length(10)])
        .split(area);

    let vr = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Min(1), Constraint::Length(3)])
        .split(hr[0]);

    (vr[0], vr[1], hr[1])
}
