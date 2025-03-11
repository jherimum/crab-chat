use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Borders, List, StatefulWidget},
};
use super::{Home, Mode};

pub struct RoomsWidget;

impl StatefulWidget for RoomsWidget {
    type State = Home;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let style = match state.mode {
            Mode::Rooms => Style::default().fg(Color::White),
            _ => Style::default().fg(Color::Green),
        };

        let rooms = List::new(state.rooms.keys().cloned().collect::<Vec<_>>())
            .highlight_style(
                Style::new().bg(Color::LightGreen).fg(Color::Black).italic(),
            )
            .highlight_symbol(">> ")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(style)
                    .border_type(BorderType::Rounded)
                    .bg(Color::Black)
                    .title("Rooms")
                    .title_alignment(Alignment::Center)
                    .title_style(Style::default().fg(Color::Green)),
            );

        rooms.render(area, buf, &mut state.rooms_state);
    }
}
