use ratatui::{
    prelude::*,
    widgets::{Block, Widget},
};

pub struct GameScreen {
    pub frame: yokoi::frame::Frame,
    pub block: Block<'static>,
}

impl Widget for &GameScreen {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        (&self.block).render(area, buf);
        let area = self.block.inner(area);
        for (y, row) in self.frame.0.iter().enumerate() {
            for (x, pixel) in row.iter().enumerate() {
                //
            }
        }
    }
}
