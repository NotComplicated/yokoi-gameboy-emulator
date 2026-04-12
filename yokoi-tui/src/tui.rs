use crate::Error;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    DefaultTerminal,
    prelude::*,
    widgets::{Block, Widget},
};
use std::time::{Duration, Instant};
use yokoi::{
    frame::{Frame, Pixel},
    system::{Input, System},
};

pub fn run(mut term: DefaultTerminal, mut system: System) -> Result<(), Error> {
    let mut screen = GameScreen::default();
    let delta_time = Duration::from_millis(1000 / 60);
    let mut input = Input::default();
    loop {
        let next_frame_at = Instant::now() + delta_time;
        loop {
            if crossterm::event::poll(next_frame_at - Instant::now())? {
                match crossterm::event::read()?.as_key_event() {
                    Some(KeyEvent { code, kind, .. }) => match code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('w') | KeyCode::Up => input.joypad.up = kind.is_press(),
                        KeyCode::Char('s') | KeyCode::Down => input.joypad.down = kind.is_press(),
                        KeyCode::Char('a') | KeyCode::Left => input.joypad.left = kind.is_press(),
                        KeyCode::Char('d') | KeyCode::Right => input.joypad.right = kind.is_press(),
                        KeyCode::Enter => input.joypad.start = kind.is_press(),
                        KeyCode::Char(' ') | KeyCode::Char('z') => input.joypad.a = kind.is_press(),
                        KeyCode::Char('x') => input.joypad.b = kind.is_press(),
                        _ => {}
                    },
                    _ => {}
                }
            }
            if Instant::now() >= next_frame_at {
                let input = Input {
                    joypad: input.joypad,
                    ..Default::default()
                };
                screen.frame = system.next_frame(input).map_err(Error::System)?;
                term.draw(|f| {
                    f.render_widget(&screen, f.area());
                })?;
                break;
            }
        }
    }
}

#[derive(Default)]
pub struct GameScreen {
    pub frame: Frame,
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
                let Pixel(r, g, b) = pixel.get();
                buf.cell_mut((area.x + x, area.y + y)).unwrap().bg = Color::Rgb(r, g, b);
            }
        }
    }
}
