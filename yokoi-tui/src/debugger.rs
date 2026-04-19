use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use image::RgbImage;
use yokoi::system::Input;

use crate::Error;

const HELP_TEXT: &str = "q - quit
c - continue running the emulator
d - display the current frame
m - show main memory registers
o - show OAM data
s - step over to the next instruction
t - show a stack trace
u - show PPU state
v - dump VRAM tile data to a bmp file";

pub fn run(mut system: yokoi::system::System) -> Result<(), Error> {
    let mut latest_frame = None;
    loop {
        let (width, height) = viuer::terminal_size();
        let viuer_config = viuer::Config {
            x: width / 2,
            restore_cursor: true,
            width: Some(width as u32 / 2),
            height: Some(height as u32 / 2),
            ..Default::default()
        };
        let input = Input::default();
        match system.next_frame(input) {
            Ok(frame) => {
                latest_frame = Some(frame.clone());
            }
            Err(yokoi::system::Error::Breakpoint(breakpoint)) => {
                log::info!(breakpoint;"");
                log::info!("press 'h' for help, 'q' to quit");
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
                            'd' => {
                                let image_buf = RgbImage::from_fn(160, 144, |x, y| {
                                    latest_frame
                                        .as_ref()
                                        .and_then(|frame| frame.0.get(y as usize))
                                        .and_then(|row| row.get(x as usize))
                                        .map(|cell| cell.get())
                                        .map(|yokoi::frame::Pixel(r, g, b)| [r, g, b])
                                        .unwrap_or([0, 0, 0])
                                        .into()
                                });
                                image_buf.save("frame.bmp").map_err(Error::Image)?;
                                viuer::print(&image_buf.into(), &viuer_config)
                                    .map_err(Error::Viuer)?;
                            }
                            'h' => log::info!("{HELP_TEXT}"),
                            'm' => system.log_mem_registers(),
                            'o' => system.log_oam(),
                            'q' => return Ok(()),
                            's' => match system.step() {
                                Ok(()) => {}
                                Err(yokoi::system::Error::Breakpoint(breakpoint)) => {
                                    log::info!(breakpoint;"")
                                }
                                Err(err) => return Err(Error::System(err)),
                            },
                            't' => {
                                for (i, frame) in system.stack_frames().iter().enumerate() {
                                    log::info!(
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
                                let mut image_buf = RgbImage::from_pixel(
                                    cols * 10,
                                    rows * 10,
                                    [135, 206, 235].into(), //sky-blue background
                                );
                                for row in 0..rows {
                                    for col in 0..cols {
                                        let tile = tiles[(row * cols + col) as usize];
                                        let (x, y) = (col * 10 + 1, row * 10 + 1);
                                        for (&(lsb, msb), dy) in tile.iter().zip(0..) {
                                            for dx in 0..8 {
                                                let lower = (lsb >> (7 - dx)) & 1;
                                                let upper = (msb >> (7 - dx)) & 1;
                                                let c = 255 - 85 * (upper * 2 + lower);
                                                *image_buf.get_pixel_mut(x + dx, y + dy) =
                                                    [c; 3].into();
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
                                image_buf.save(&path).map_err(Error::Image)?;
                                log::info!(
                                    "saved tiles to {}",
                                    std::fs::canonicalize(&path)?.display()
                                );
                                viuer::print(&image_buf.into(), &viuer_config)
                                    .map_err(Error::Viuer)?;
                            }
                            _ => {}
                        }
                    }
                }
            }
            Err(err) => break Err(Error::System(err)),
        }
    }
}
