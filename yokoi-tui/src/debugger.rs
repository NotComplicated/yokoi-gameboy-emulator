use crate::Error;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use image::{Rgb, RgbImage};
use yokoi::{
    Input, ScreenPos, SymbolError,
    frame::{Frame, Pixel},
    system::{Address, System},
};

const TILES_TEMPLATE: &[u8] = include_bytes!("../tiles_template.bmp");

const HELP_TEXT: &str = "q - quit
a - print current bank, address, and debug symbol
b - set a breakpoint on a debug symbol
c - continue running the emulator
d - display the current frame
e - display the current frame with 8-pixel guides
l - set log level
m - print main memory registers
n - step over the next instruction / function call
o - print OAM data
p - print background tile data
r - display the full background
s - step over the next instruction
t - print a stack trace
u - print PPU state
v - dump VRAM tile data to a bmp file";

const GUIDE_COLOR: Rgb<u8> = Rgb([135, 206, 235]);

fn viuer_config() -> viuer::Config {
    let (width, height) = viuer::terminal_size();
    viuer::Config {
        x: width / 2,
        restore_cursor: true,
        width: Some(width as u32 / 2),
        height: Some(height as u32 / 2),
        ..Default::default()
    }
}

pub struct Debugger {
    system: System,
    latest_frame: Option<Frame>,
}

enum HandleBreak {
    Quit,
    Continue,
}

impl Debugger {
    pub fn new(system: System) -> Self {
        Self {
            system,
            latest_frame: None,
        }
    }

    pub fn run(&mut self) -> Result<(), Error> {
        loop {
            let input = Input::default();
            match self.system.next_frame(input) {
                Ok(frame) => {
                    self.latest_frame = Some(frame.clone());
                }
                Err(yokoi::system::Error::Breakpoint(breakpoint)) => {
                    log::info!(breakpoint;"");
                    match self.handle_break()? {
                        HandleBreak::Quit => break Ok(()),
                        HandleBreak::Continue => {}
                    }
                }
                Err(yokoi::system::Error::ShortCircuit) => match self.handle_break()? {
                    HandleBreak::Quit => break Ok(()),
                    HandleBreak::Continue => {}
                },
                Err(err) => break Err(Error::System(err)),
            }
        }
    }

    fn display_frame(&self, guides: bool) -> Result<(), Error> {
        let mut image_buf = RgbImage::from_fn(160, 144, |x, y| {
            self.latest_frame
                .as_ref()
                .and_then(|frame| frame.0.get(y as usize))
                .and_then(|row| row.get(x as usize))
                .map(|cell| cell.get())
                .map(|Pixel(r, g, b)| [r, g, b])
                .unwrap_or([0, 0, 0])
                .into()
        });
        if guides {
            for x in (0..image_buf.width()).step_by(8).skip(1) {
                for y in (0..image_buf.height()).step_by(8).skip(1) {
                    image_buf.put_pixel(x - 1, y, GUIDE_COLOR);
                    image_buf.put_pixel(x + 1, y, GUIDE_COLOR);
                    image_buf.put_pixel(x, y - 1, GUIDE_COLOR);
                    image_buf.put_pixel(x, y + 1, GUIDE_COLOR);
                    image_buf.put_pixel(x, y, GUIDE_COLOR);
                }
            }
        }
        image_buf.save("frame.bmp").map_err(Error::Image)?;
        viuer::print(&image_buf.into(), &viuer_config()).map_err(Error::Viuer)?;
        Ok(())
    }

    fn handle_break(&mut self) -> Result<HandleBreak, Error> {
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
                    'a' => match self.system.address() {
                        Address {
                            bank: Some(bank),
                            addr,
                            latest_symbol: Some(symbol),
                        } => {
                            log::info!("bank - {bank:02}, addr - {addr:04X}, symbol - {symbol}");
                        }
                        Address {
                            bank: Some(bank),
                            addr,
                            latest_symbol: None,
                        } => {
                            log::info!("bank - {bank:02}, addr - {addr:04X}");
                        }
                        Address {
                            bank: None,
                            addr,
                            latest_symbol: Some(symbol),
                        } => {
                            log::info!("bank - 00, addr - {addr:04X}, symbol - {symbol}");
                        }
                        Address {
                            bank: None,
                            addr,
                            latest_symbol: None,
                        } => {
                            log::info!("bank - 00, addr - {addr:04X}");
                        }
                    },

                    'b' => {
                        log::info!("add breakpoint:");
                        let breakpoint = std::io::stdin().lines().next().unwrap()?;
                        match self.system.add_breakpoint(breakpoint.trim()) {
                            Ok((bank, addr)) => {
                                log::info!("breakpoint added. bank - {bank:02}, addr - {addr:04X}")
                            }
                            Err(SymbolError::BreakpointNotFound(_)) => {
                                log::error!("- Breakpoint not found -")
                            }
                            Err(SymbolError::NoneLoaded) => {
                                log::error!("- No symbol map loaded -")
                            }
                            _ => panic!("unexpected error"),
                        }
                    }

                    'c' => break Ok(HandleBreak::Continue),

                    'd' => self.display_frame(false)?,

                    'e' => self.display_frame(true)?,

                    'h' => log::info!("{HELP_TEXT}"),

                    'l' => {
                        log::info!("new level:");
                        if let Ok(filter) = std::io::stdin()
                            .lines()
                            .next()
                            .unwrap()?
                            .trim()
                            .parse::<log::LevelFilter>()
                        {
                            log::set_max_level(filter);
                            log::info!("log level set to '{filter}'");
                        } else {
                            log::error!("invalid log level");
                        }
                    }

                    'm' => self.system.log_mem_registers(),

                    'n' => match self.system.step_over() {
                        Ok(()) => {}
                        Err(yokoi::system::Error::Breakpoint(breakpoint)) => {
                            log::info!(breakpoint;"")
                        }
                        Err(err) => return Err(Error::System(err)),
                    },

                    'o' => self.system.log_oam(),

                    'p' => self.system.log_bg(),

                    'q' => break Ok(HandleBreak::Quit),

                    'r' => {
                        let background = self.system.background();
                        let mut image_buf = RgbImage::new(32 * 8, 32 * 8);
                        for (y, row) in (0..).step_by(8).zip(background) {
                            for (x, tile) in (0..).step_by(8).zip(row) {
                                for (&(lsb, msb), dy) in tile.iter().zip(0..) {
                                    for dx in 0..8 {
                                        let lower = (lsb >> (7 - dx)) & 1;
                                        let upper = (msb >> (7 - dx)) & 1;
                                        let c = 255 - 85 * (upper * 2 + lower);
                                        image_buf.put_pixel(x + dx, y + dy, [c, c, c].into());
                                    }
                                }
                            }
                        }
                        let [top_left, bottom_right] = self.system.bounds();
                        let offsets = [
                            (0, 0),
                            (0, 1),
                            (1, 0),
                            (1, 1),
                            (2, 0),
                            (2, 1),
                            (3, 0),
                            (3, 1),
                            (0, 2),
                            (0, 3),
                            (1, 2),
                            (1, 3),
                        ]
                        .map(|(x, y)| ScreenPos::new(x, y));
                        for point in offsets
                            .into_iter()
                            .map(|offset| top_left + offset)
                            .chain(offsets.into_iter().map(|offset| bottom_right - offset))
                        {
                            image_buf.put_pixel(point.x.0.into(), point.y.0.into(), GUIDE_COLOR);
                        }
                        image_buf.save("background.bmp").map_err(Error::Image)?;
                        viuer::print(&image_buf.into(), &viuer_config()).map_err(Error::Viuer)?;
                    }

                    's' => match self.system.step_in() {
                        Ok(()) => {}
                        Err(yokoi::system::Error::Breakpoint(breakpoint)) => {
                            log::info!(breakpoint;"")
                        }
                        Err(err) => return Err(Error::System(err)),
                    },

                    't' => {
                        for (i, frame) in self.system.stack_frames().iter().enumerate() {
                            log::info!(
                                frame = i,
                                bank = frame.bank,
                                address = format!("{:04X}", frame.addr),
                                symbol = frame.latest_symbol
                                ;""
                            );
                        }
                    }

                    'u' => self.system.log_ppu_state(),

                    'v' => {
                        let tiles = self.system.vram_tiles();
                        let cols = 24;
                        let rows = tiles.len() as u32 / cols;
                        let mut image_buf: RgbImage = image::load_from_memory(TILES_TEMPLATE)
                            .map_err(Error::Image)?
                            .into();
                        for row in 0..rows {
                            for col in 0..cols {
                                let tile = tiles[(row * cols + col) as usize];
                                let (x, y) = (col * 10 + 11, row * 10 + 11);
                                for (&(lsb, msb), dy) in tile.iter().zip(0..) {
                                    for dx in 0..8 {
                                        let lower = (lsb >> (7 - dx)) & 1;
                                        let upper = (msb >> (7 - dx)) & 1;
                                        let c = 255 - 85 * (upper * 2 + lower);
                                        image_buf.put_pixel(x + dx, y + dy, [c, c, c].into());
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
                        log::info!("saved tiles to {}", std::fs::canonicalize(&path)?.display());
                        viuer::print(&image_buf.into(), &viuer_config()).map_err(Error::Viuer)?;
                    }
                    _ => {}
                }
            }
        }
    }
}
