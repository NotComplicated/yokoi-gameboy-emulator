use crate::{
    frame::Frame,
    memory::{self, LY_REG, Memory},
    system::{Interrupt, Mode},
};

const LY_END: u8 = 154;
const DOT_END: u16 = 456;
const VBLANK_LY_START: u8 = 144;
const OAM_SCAN_DOT_START: u16 = 0;
const OAM_SCAN_DOT_END: u16 = 79;
const DRAWING_DOT_START: u16 = 80;
const HBLANK_DOT_START_MIN: u16 = 252;
const HBLANK_DOT_START_MAX: u16 = 369;

const MAP_LOWER_START: u16 = 0x9800;
const MAP_UPPER_START: u16 = 0x9C00;
const DATA_LOWER_START: u16 = 0x8000;
const DATA_UPPER_START: u16 = 0x8800;

const FETCH_STEPS: u8 = 6;

pub struct Ppu {
    mode: Mode,
    state: State,
    ly: u8,
    dot: u16,
    enabled: bool,
    window_enabled: bool,
    window_latched: bool,
    window_counter: u8,
    obj_enabled: bool,
    bg_w_priority: bool,
    w_map_addr: u16,
    bg_map_addr: u16,
    bg_w_data_addr: u16,
    obj_height: u8,
    stat_lyc: bool,
    stat_modes: [bool; 3],
    frame: Frame,
}

#[derive(Debug)]
enum State {
    Hblank,
    Vblank,
    OamScan {
        oam: OamBuf,
    },
    // first fetch is discarded, track it separately
    FirstFetch {
        oam: OamBuf,
        progress: u8,
    },
    Drawing {
        oam: OamBuf,
        fifo: Fifo,
        x: u8,
        in_window: bool,
        fetcher: Fetcher,
        discard: u8,
    },
}

#[derive(Debug)]
pub enum Error {
    Memory(memory::Error),
}

impl From<memory::Error> for Error {
    fn from(err: memory::Error) -> Self {
        Self::Memory(err)
    }
}

#[derive(Copy, Clone, Default, Debug)]
struct OamBuf {
    buffer: [Object; 10],
    len: usize,
}

#[derive(Copy, Clone, Default, Debug)]
struct Object {
    y: u8,
    x: u8,
    tile: u8,
    priority: bool,
    y_flip: bool,
    x_flip: bool,
    palette: u8,
    bank: u8,
}

#[derive(Debug)]
enum Fetcher {
    Bg {
        x: u8,
        progress: u8,
        cached: Option<[Pixel; 8]>,
    },
    Window {
        x: u8,
        progress: u8,
        cached: Option<[Pixel; 8]>,
    },
    Object {
        x: u8,
        progress: u8,
        index: usize,
    },
}

impl Fetcher {
    fn get_x(&self) -> u8 {
        match self {
            Fetcher::Bg { x, .. } => *x,
            Fetcher::Window { x, .. } => *x,
            Fetcher::Object { x, .. } => *x,
        }
    }
}

#[derive(Debug)]
struct Fifo {
    buffer: [Pixel; 16],
    len: usize,
    front: usize,
    back: usize,
}

impl Fifo {
    fn new() -> Self {
        Self {
            buffer: [Pixel::Tile {
                color: 0,
                palette: 0,
                priority: 0,
            }; 16],
            len: 0,
            front: 0,
            back: 0,
        }
    }

    fn push_8(&mut self, pixels: [Pixel; 8]) -> Result<(), ()> {
        if self.len + 8 > self.buffer.len() {
            Err(())
        } else {
            for pixel in pixels {
                self.buffer[self.back] = pixel;
                self.back = (self.back + 1) % self.buffer.len();
            }
            self.len += 8;
            Ok(())
        }
    }

    fn pop(&mut self) -> Option<Pixel> {
        if self.len == 0 {
            None
        } else {
            let pixel = self.buffer[self.front];
            self.front = (self.front + 1) % self.buffer.len();
            self.len -= 1;
            Some(pixel)
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum Pixel {
    Tile {
        color: u8,
        palette: u8,
        priority: u8,
    },
    Object {
        color: u8,
        palette: u8,
        priority: u8,
    },
}

impl Ppu {
    pub fn init(mode: Mode) -> Self {
        Self {
            mode,
            state: State::OamScan {
                oam: Default::default(),
            },
            ly: 0,
            dot: 0,
            enabled: false,
            window_enabled: false,
            window_latched: false,
            window_counter: 0,
            obj_enabled: false,
            bg_w_priority: false,
            w_map_addr: MAP_LOWER_START,
            bg_map_addr: MAP_LOWER_START,
            bg_w_data_addr: DATA_LOWER_START,
            obj_height: 8,
            stat_lyc: false,
            stat_modes: [false; 3],
            frame: Default::default(),
        }
    }

    pub fn tick(&mut self, memory: &mut Memory) -> Result<Option<Frame>, Error> {
        let stats_set_before = self
            .stat_modes
            .iter()
            .copied()
            .chain([self.stat_lyc])
            .any(|stat| stat);
        self.read_lcdc(memory)?;
        let mut frame = None;

        // TODO set stat_ members, update STAT reg, set vblank interrupt

        match &mut self.state {
            State::Hblank => {
                if self.dot == DOT_END - 1 {
                    self.ly += 1;
                    memory.write(memory::LY_REG, self.ly)?;
                    if self.ly < VBLANK_LY_START {
                        if !self.window_latched {
                            self.window_latched = self.ly == memory.read(memory::WINDOW_Y_REG)?;
                        }
                        self.state = State::OamScan {
                            oam: Default::default(),
                        };
                    } else {
                        frame = Some(std::mem::take(&mut self.frame));
                        //TODO STAT mode transitions to 01 (Mode 1 — VBlank).
                        //TODO VBlank Interrupt fires ($FF0F bit 0 is set). This is the primary signal for the CPU to update graphics, run game logic, etc.
                        //TODO STAT Mode 1 IRQ (bit 4) is not enabled in our scenario → no STAT IRQ.
                        //TODO LYC=LY: LYC=72, LY=144 → no match.
                        self.state = State::Vblank;
                    };
                }
            }

            State::Vblank => {
                if self.dot == DOT_END - 1 {
                    self.ly += 1;
                    if self.ly == LY_END {
                        self.ly = 0;
                        self.window_latched = false;
                        self.window_counter = 0;
                        self.state = State::OamScan {
                            oam: Default::default(),
                        };
                    }
                    memory.write(memory::LY_REG, self.ly)?;
                }
            }

            State::OamScan { oam } => {
                match self.dot {
                    OAM_SCAN_DOT_START => {
                        memory.write(LY_REG, self.ly)?;
                        //TODO STAT mode field (bits 1–0) transitions to 10 (Mode 2).
                        //TODO The STAT interrupt line is checked. Mode 2 IRQ (bit 5) is not enabled, so no STAT IRQ fires.
                        //TODO LYC (=72) is compared against LY (=0). No match → STAT bit 2 (LYC=LY flag) cleared.
                        memory.lock(memory::Lock::Oam);

                        for &[y, x, tile, flags] in memory.oam().as_chunks::<4>().0 {
                            if (y..y + self.obj_height).contains(&self.ly) {
                                // This object is within the current scanline, add to OAM buffer
                                oam.buffer[oam.len] = Object {
                                    y,
                                    x,
                                    tile,
                                    priority: flags & 0b10000000 != 0,
                                    y_flip: flags & 0b01000000 != 0,
                                    x_flip: flags & 0b00100000 != 0,
                                    palette: if self.mode == Mode::Cgb {
                                        flags & 0b00000111
                                    } else {
                                        flags & 0b00010000 >> 4
                                    },
                                    bank: flags & 0b00001000 >> 3,
                                };
                                oam.len += 1;
                                if oam.len == oam.buffer.len() {
                                    break;
                                }
                            }
                        }
                    }

                    OAM_SCAN_DOT_END => {
                        self.state = State::FirstFetch {
                            oam: *oam,
                            progress: FETCH_STEPS,
                        };
                        //TODO STAT mode field transitions to 11 (Mode 3).
                        memory.lock(memory::Lock::VramOam);
                    }

                    _ => {}
                }
            }

            State::FirstFetch { oam, progress: 0 } => {
                self.state = State::Drawing {
                    oam: *oam,
                    fifo: Fifo::new(),
                    x: 0,
                    in_window: false,
                    fetcher: Fetcher::Bg {
                        x: 0,
                        progress: FETCH_STEPS,
                        cached: None,
                    },
                    discard: memory.read(memory::SCROLL_X_REG)? % 8,
                }
            }

            State::FirstFetch { progress, .. } => {
                *progress -= 1;
            }

            State::Drawing {
                oam,
                fifo,
                x,
                in_window,
                fetcher,
                discard,
            } => {
                let scroll_x = memory.read(memory::SCROLL_X_REG)?;
                let scroll_y = memory.read(memory::SCROLL_Y_REG)?;

                match fetcher {
                    Fetcher::Bg {
                        x,
                        cached: Some(pixels),
                        ..
                    } => {
                        if fifo.push_8(*pixels).is_ok() {
                            *fetcher = Fetcher::Bg {
                                x: *x + 1,
                                progress: FETCH_STEPS,
                                cached: None,
                            };
                        }
                    }
                    Fetcher::Bg {
                        x,
                        progress: 0,
                        cached: None,
                    } => {
                        //TODO CGB reads BG tilemap attrs
                        let row = (scroll_y + self.ly) as u16 >> 3;
                        let col = ((scroll_x >> 3) + *x) as u16;
                        let bg_tile_addr = self.bg_map_addr + (row << 5) + col;
                        let bg_tile = memory.read(bg_tile_addr)?;
                        let ysub = (scroll_y + self.ly) as u16 % 8;
                        let data_addr = self.bg_w_data_addr + (bg_tile as u16 * 16) + (ysub * 2);
                        let (data_lo, data_hi) =
                            (memory.read(data_addr)?, memory.read(data_addr + 1)?);
                        let pixels = std::array::from_fn(|i| Pixel::Tile {
                            color: ((data_lo >> (7 - i)) % 2) * 2 + ((data_hi >> (7 - i)) % 2),
                            palette: 0,
                            priority: 0,
                        });
                        if fifo.push_8(pixels).is_ok() {
                            *fetcher = Fetcher::Bg {
                                x: *x + 1,
                                progress: FETCH_STEPS,
                                cached: None,
                            };
                        } else {
                            *fetcher = Fetcher::Bg {
                                x: *x,
                                progress: 0,
                                cached: Some(pixels),
                            };
                        }
                    }
                    Fetcher::Window { progress: 0, .. } => todo!(),
                    Fetcher::Object { progress: 0, .. } => todo!(),
                    Fetcher::Bg { progress, .. }
                    | Fetcher::Window { progress, .. }
                    | Fetcher::Object { progress, .. } => *progress -= 1,
                }

                // first SCX%8 columns of the scanline
                while *discard > 0 {
                    fifo.pop();
                    *discard -= 1;
                }

                let theme = crate::frame::Theme::Classic; // TODO
                let frame_pixel = match (fifo.pop(), self.mode) {
                    (None, _) => None,
                    (Some(Pixel::Object { color, palette, .. }), Mode::Dmg) => todo!(),
                    (Some(Pixel::Object { color, palette, .. }), Mode::Cgb) => todo!(),
                    (Some(Pixel::Tile { color, .. }), Mode::Dmg) => {
                        let bgp = memory.read(memory::BG_PALETTE_REG)?;
                        let color = bgp >> (color * 2) & 0b00000011;
                        Some(crate::frame::Pixel::from_2bit(color, theme))
                    }
                    (Some(Pixel::Tile { color, palette, .. }), Mode::Cgb) => todo!(),
                };
                if let Some(frame_pixel) = frame_pixel {
                    self.frame[(*x as _, self.ly as _)] = frame_pixel;
                    *x += 1;
                    if self.window_enabled && self.window_latched && !*in_window {
                        if memory.read(memory::WINDOW_X_REG)? == *x + 7 {
                            *fetcher = Fetcher::Window {
                                x: fetcher.get_x(),
                                progress: FETCH_STEPS,
                                cached: None,
                            };
                            *in_window = true;
                        }
                    }
                    for i in 0..oam.len {
                        if *x + scroll_x == oam.buffer[i].x.saturating_sub(8) {
                            *fetcher = Fetcher::Object {
                                x: fetcher.get_x(),
                                progress: FETCH_STEPS,
                                index: i,
                            };
                            break;
                        }
                    }
                }
            }
        }

        self.dot = (self.dot + 1) % DOT_END;
        let stats_set_after = self
            .stat_modes
            .iter()
            .copied()
            .chain([self.stat_lyc])
            .any(|stat| stat);
        // enable interrupt handler on rising edge
        if !stats_set_before && stats_set_after {
            memory.write(
                memory::INTERRUPTS_REG,
                memory.read(memory::INTERRUPTS_REG)? | 0b00000010,
            )?;
        }
        Ok(frame)
    }

    fn read_lcdc(&mut self, memory: &Memory) -> Result<(), Error> {
        let lcdc = memory.read(memory::LCD_CTRL_REG)?;
        self.enabled = lcdc & 0b10000000 != 0;
        self.window_enabled = lcdc & 0b00100000 != 0;
        self.obj_enabled = lcdc & 0b00000010 != 0;
        self.bg_w_priority = lcdc & 0b00000001 != 0;
        self.w_map_addr = if lcdc & 0b01000000 == 0 {
            MAP_LOWER_START
        } else {
            MAP_UPPER_START
        };
        self.bg_map_addr = if lcdc & 0b00001000 == 0 {
            MAP_LOWER_START
        } else {
            MAP_UPPER_START
        };
        self.bg_w_data_addr = if lcdc & 0b00010000 == 0 {
            DATA_UPPER_START
        } else {
            DATA_LOWER_START
        };
        self.obj_height = if lcdc & 0b00000100 == 0 { 8 } else { 16 };
        Ok(())
    }
}
