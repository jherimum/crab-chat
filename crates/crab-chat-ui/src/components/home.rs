use std::collections::HashMap;
use color_eyre::Result;
use crab_chat_peer::{Peer, PeerEventListener};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{action::Action, config::Config};

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

pub struct Home {
    peer: Peer,
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    text: String,
    rooms: HashMap<String, Room>,
    actual_room: Option<String>,
    show_rooms: bool,
}

impl Home {
    pub fn new(peer: Peer) -> Self {
        Self {
            command_tx: None,
            config: Config::default(),
            text: String::new(),
            rooms: HashMap::new(),
            actual_room: None,
            peer,
            show_rooms: false,
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
        match (key.code, key.modifiers) {
            (KeyCode::Char('r'), KeyModifiers::CONTROL) => {
                return Ok(Some(Action::Rooms));
            }
            _ => Ok(None),
        }
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Rooms => {
                self.show_rooms = true;
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let (header, main, footer) = vertical_layout(area);
        let (rooms, chat_panel) = horizontal_layout(main);
        let (my_rooms, network_rooms) = rooms_panel(rooms);
        let (chat, input, participants) = chat_layout(chat_panel);

        frame.render_widget(
            Paragraph::new("Crab Chat")
                .alignment(Alignment::Center)
                .bold()
                .block(Block::bordered())
                .black()
                .bg(Color::Green),
            header,
        );

        frame.render_widget(
            Block::default()
                .borders(Borders::ALL)
                .bg(Color::Black)
                .title("My Rooms")
                .title_alignment(Alignment::Center)
                .title_style(Style::default().fg(Color::Green)),
            my_rooms,
        );

        frame.render_widget(
            Block::default()
                .borders(Borders::ALL)
                .bg(Color::Black)
                .title("Network Rooms")
                .title_alignment(Alignment::Center)
                .title_style(Style::default().fg(Color::Green)),
            network_rooms,
        );

        frame.render_widget(
            Block::default()
                .borders(Borders::ALL)
                .bg(Color::Black)
                .title("Chat")
                .title_alignment(Alignment::Center)
                .title_style(Style::default().fg(Color::Green)),
            chat_panel,
        );

        // frame.render_widget(
        //     Block::new()
        //         .borders(Borders::ALL)
        //         .green()
        //         .style(Style::default())
        //         .bg(Color::Black),
        //     inner[0],
        // );

        // frame.render_widget(
        //     Paragraph::new(self.text.as_str())
        //         .block(Block::new().borders(Borders::ALL).black())
        //         .style(Style::default().bg(Color::Green).fg(Color::White)),
        //     outer[2],
        // );

        // if self.show_rooms {}

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

fn rooms_panel(area: Rect) -> (Rect, Rect) {
    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
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

pub struct Room {
    name: String,
    chat: Chat,
}

pub struct Chat {
    messages: Vec<String>,
    input: String,
}
