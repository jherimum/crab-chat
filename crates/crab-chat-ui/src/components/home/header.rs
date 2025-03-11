use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Stylize},
    widgets::{Block, Paragraph, Widget},
};

pub struct HeaderWidget;

impl Widget for HeaderWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Paragraph::new("Crab Chat")
            .alignment(Alignment::Center)
            .bold()
            .block(Block::bordered())
            .black()
            .bg(Color::Green)
            .render(area, buf);
    }
}
