use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Borders, StatefulWidget, Widget},
};

use super::{Home, Mode};

pub struct ChatWidget;

impl StatefulWidget for ChatWidget {
    type State = Home;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let border = match state.mode {
            Mode::Chat => BorderType::Thick,
            _ => BorderType::Plain,
        };

        Block::default()
            .borders(Borders::ALL)
            .border_type(border)
            .bg(Color::Black)
            .title(state.actual_room.as_deref().unwrap_or("Chat"))
            .title_alignment(Alignment::Center)
            .title_style(Style::default().fg(Color::Green))
            .render(area, buf);
    }
}
