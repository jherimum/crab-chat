use ratatui::widgets::{Block, Clear, Widget};

pub struct RoomsPopup;

impl Widget for RoomsPopup {
    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) where
        Self: Sized,
    {
        Clear.render(area, buf);
        let block = Block::default();

        todo!()
    }
}
