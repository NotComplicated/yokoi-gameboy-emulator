use crate::Error;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
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
    let delta_time = Duration::from_millis(10);
    'game_loop: loop {
        let mut now = Instant::now();
        let next_frame_at = now + delta_time;
        let mut input = Input::default();
        while now < next_frame_at {
            if crossterm::event::poll(next_frame_at - now)? {
                match crossterm::event::read()?.as_key_event() {
                    Some(KeyEvent {
                        code,
                        kind: KeyEventKind::Press,
                        ..
                    }) => match code {
                        KeyCode::Char('q') => break 'game_loop,
                        KeyCode::Char('w') | KeyCode::Up => input.joypad.up = true,
                        KeyCode::Char('s') | KeyCode::Down => input.joypad.down = true,
                        KeyCode::Char('a') | KeyCode::Left => input.joypad.left = true,
                        KeyCode::Char('d') | KeyCode::Right => input.joypad.right = true,
                        KeyCode::Char('c') | KeyCode::Enter => input.joypad.start = true,
                        KeyCode::Char('v') => input.joypad.select = true,
                        KeyCode::Char(' ') | KeyCode::Char('z') => input.joypad.a = true,
                        KeyCode::Char('x') => input.joypad.b = true,
                        _ => {}
                    },
                    _ => {}
                }
                log::debug!(joypad:? = input.joypad;"");
            }
            now = Instant::now();
        }
        let input = Input {
            joypad: input.joypad,
            ..Default::default()
        };
        screen.frame = system.next_frame(input).map_err(Error::System)?;
        term.draw(|f| {
            f.render_widget(&screen, f.area());
        })?;
    }
    for (i, frame) in system.stack_frames().iter().enumerate() {
        log::info!(
            frame = i,
            bank = frame.bank,
            address = format!("{:04X}", frame.addr),
            symbol = frame.latest_symbol
            ;""
        );
    }
    Ok(())
}

#[derive(Default)]
pub struct GameScreen {
    pub frame: Frame,
    pub block: Block<'static>,
}

impl Widget for &GameScreen {
    fn render(self, area: Rect, buf: &mut Buffer) {
        (&self.block).render(area, buf);
        let area = self.block.inner(area);
        let rows = (0..).take_while(|&y| y < area.height).zip(&*self.frame.0);
        for (y, row) in rows {
            let pixels = (0..).take_while(|&x| x < area.width / 2).zip(row);
            for (x, pixel) in pixels {
                let Pixel(r, g, b) = pixel.get();
                buf.cell_mut((area.x + x * 2, area.y + y)).unwrap().bg = Color::Rgb(r, g, b);
                buf.cell_mut((area.x + x * 2 + 1, area.y + y)).unwrap().bg = Color::Rgb(r, g, b);
            }
        }
    }
}
