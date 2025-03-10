use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Borders, Paragraph},
};
use tui_textarea::TextArea;

use super::app::App;

pub fn render(app: &mut App, frame: &mut Frame) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Percentage(100),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(frame.area());

    let inner = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints(
            [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
        )
        .split(outer[1]);

    frame.render_widget(
        Paragraph::new("Crab chat -  A decentralized chat")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Black).bg(Color::Green).bold()),
        outer[0],
    );

    frame.render_widget(
        Block::default()
            .title("Rooms")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green))
            .style(Style::default().bg(Color::Black)),
        inner[0],
    );

    frame.render_widget(
        Block::default()
            .title("Rooms")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green))
            .style(Style::default().bg(Color::Black)),
        inner[1],
    );

    app.input.set_block(
        Block::default()
            .borders(Borders::ALL)
            .title("Crossterm Minimal Example"),
    );

    frame.render_widget(&app.input, outer[2]);
}
