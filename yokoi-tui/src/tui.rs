use crate::Error;
use ratatui::{
    DefaultTerminal,
    prelude::*,
    widgets::{Block, Widget},
};

pub fn run(mut term: DefaultTerminal, mut system: yokoi::system::System) -> Result<(), Error> {
    let mut screen = GameScreen::default();
    loop {
        std::thread::sleep(std::time::Duration::from_millis(1000 / 60));
        screen.frame = system
            .next_frame(Default::default())
            .map_err(Error::System)?;
        term.draw(|f| {
            f.render_widget(&screen, f.area());
        })
        .map_err(Error::Io)?;
    }
}

#[derive(Default)]
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
        let rows = (0..).take_while(|&y| y < area.height).zip(&*self.frame.0);
        for (y, row) in rows {
            let pixels = (0..).take_while(|&x| x < area.width).zip(row);
            for (x, pixel) in pixels {
                let yokoi::frame::Pixel(r, g, b) = pixel.get();
                buf.cell_mut((area.x + x, area.y + y)).unwrap().bg = Color::Rgb(r, g, b);
            }
        }
    }
}
