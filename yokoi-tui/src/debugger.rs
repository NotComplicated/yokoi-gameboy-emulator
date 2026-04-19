use std::time::Instant;

use bmp::{Image, Pixel};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use log::info;
use yokoi::system::Input;

use crate::Error;

const HELP_TEXT: &str = "q - quit
c - continue running the emulator
r - show main memory registers
s - step over to the next instruction
t - show a stack trace
u - show PPU state
v - dump VRAM tile data to a bmp file";

pub fn run(mut system: yokoi::system::System) -> Result<(), Error> {
    loop {
        let input = Input::default();
        if let Err(err) = system.next_frame(input) {
            if let yokoi::system::Error::Breakpoint(breakpoint) = err {
                info!(breakpoint;"");
                info!("press 'h' for help, 'q' to quit");
                loop {
                    crossterm::terminal::enable_raw_mode()?;
                    if let Event::Key(KeyEvent {
                        code: KeyCode::Char(key),
                        kind: KeyEventKind::Press,
                        ..
                    }) = crossterm::event::read()?
                    {
                        crossterm::terminal::disable_raw_mode()?;
                        match key {
                            'c' => break,
                            'h' => info!("{HELP_TEXT}"),
                            'q' => return Ok(()),
                            'r' => system.log_mem_registers(),
                            's' => match system.step() {
                                Ok(()) => {}
                                Err(yokoi::system::Error::Breakpoint(breakpoint)) => {
                                    info!(breakpoint;"")
                                }
                                Err(err) => return Err(Error::System(err)),
                            },
                            't' => {
                                for (i, frame) in system.stack_frames().iter().enumerate() {
                                    info!(
                                        frame = i,
                                        bank = frame.bank,
                                        address = format!("{:04X}", frame.addr),
                                        symbol = frame.latest_symbol
                                        ;""
                                    );
                                }
                            }
                            'u' => system.log_ppu_state(),
                            'v' => {
                                let tiles = system.vram_tiles();
                                let cols = 24;
                                let rows = tiles.len() as u32 / cols;
                                let mut bmp = Image::new(cols * 10, rows * 10);
                                for (x, y) in bmp.coordinates() {
                                    bmp.set_pixel(x, y, bmp::consts::SKYBLUE); // background
                                }
                                for row in 0..rows {
                                    for col in 0..cols {
                                        let tile = tiles[(row * cols + col) as usize];
                                        let (x, y) = (col * 10 + 1, row * 10 + 1);
                                        for (&(lsb, msb), dy) in tile.iter().zip(0..) {
                                            for dx in 0..8 {
                                                let lower = (lsb >> (7 - dx)) & 1;
                                                let upper = (msb >> (7 - dx)) & 1;
                                                let c = 255 - 85 * (upper * 2 + lower);
                                                bmp.set_pixel(x + dx, y + dy, Pixel::new(c, c, c));
                                            }
                                        }
                                    }
                                }
                                let path = format!(
                                    "tiles_{}.bmp",
                                    std::time::UNIX_EPOCH
                                        .elapsed()
                                        .expect("epoch < now")
                                        .as_millis()
                                );
                                bmp.save(&path)?;
                                info!("saved tiles to {}", std::fs::canonicalize(path)?.display());
                            }
                            _ => {}
                        }
                    }
                }
            } else {
                return Err(Error::System(err));
            }
        }
    }
}
