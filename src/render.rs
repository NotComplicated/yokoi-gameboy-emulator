use crate::{
    frame::Frame,
    mem::{self, LY_REG, Memory},
    system::Mode,
};

const X_END: u8 = 160;
const LY_END: u8 = 154;
const DOT_END: u16 = 456;
const VBLANK_LY_START: u8 = 144;
const OAM_SCAN_DOT_START: u16 = 0;
const OAM_SCAN_DOT_END: u16 = 79;

const MAP_LOWER_START: u16 = 0x9800;
const MAP_UPPER_START: u16 = 0x9C00;
const DATA_0_START: u16 = 0x8000;
const DATA_1_START: u16 = 0x8800;
const DATA_2_START: u16 = 0x9000;

const FETCH_STEPS: u8 = 6;

pub struct Ppu {
    mode: Mode,
    state: State,
    ly: u8,
    dot: u16,
    enabled: bool,
    window_enabled: bool,
    window_latched: bool,
    window_counter: u16,
    obj_enabled: bool,
    bg_w_priority: bool,
    w_map_addr: u16,
    bg_map_addr: u16,
    bg_w_data_addr: u16,
    obj_height: u8,
    lyc_int_enable: bool,
    mode_int_enable: [bool; 3],
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
    Memory(mem::Error),
}

impl From<mem::Error> for Error {
    fn from(err: mem::Error) -> Self {
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
        obj_queued: Option<usize>,
    },
    Window {
        x: u8,
        progress: u8,
        cached: Option<[Pixel; 8]>,
        obj_queued: Option<usize>,
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
        priority: bool,
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
            bg_w_data_addr: DATA_0_START,
            obj_height: 8,
            lyc_int_enable: false,
            mode_int_enable: [false; _],
            frame: Default::default(),
        }
    }

    pub fn tick(&mut self, memory: &mut Memory) -> Result<Option<Frame>, Error> {
        self.read_lcdc_stat(memory)?;
        if !self.enabled {
            return Ok(None);
        }
        let mut frame = None;
        let mut lyc_match = false;

        match &mut self.state {
            State::Hblank => {
                if self.dot == DOT_END - 1 {
                    self.ly += 1;
                    memory.write(mem::LY_REG, self.ly)?;
                    if self.ly < VBLANK_LY_START {
                        if !self.window_latched {
                            self.window_latched = self.ly == memory.read(mem::WINDOW_Y_REG)?;
                        }
                        self.state = State::OamScan {
                            oam: Default::default(),
                        };
                    } else {
                        frame = Some(self.frame.clone());
                        memory.write(mem::IF_REG, memory.read(mem::IF_REG)? | 0b00000001)?;
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
                    memory.write(mem::LY_REG, self.ly)?;
                }
            }

            State::OamScan { oam } => {
                match self.dot {
                    OAM_SCAN_DOT_START => {
                        memory.write(LY_REG, self.ly)?;
                        lyc_match = self.ly == memory.read(mem::LYC_REG)?;
                        memory.set_lock(mem::Lock::Oam);

                        for &[y, x, tile, flags] in memory.oam().as_chunks::<4>().0 {
                            if (y.saturating_sub(16)..(y + self.obj_height).saturating_sub(16))
                                .contains(&self.ly)
                            {
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
                        memory.set_lock(mem::Lock::VramOam);
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
                        obj_queued: None,
                    },
                    discard: memory.read(mem::SCROLL_X_REG)? % 8,
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
                let x_tile_last = (X_END / 8) - 1;
                let scroll_x = memory.read(mem::SCROLL_X_REG)?;
                let scroll_y = memory.read(mem::SCROLL_Y_REG)?;
                let get_pixels = |lo, hi| {
                    std::array::from_fn(|i| Pixel::Tile {
                        color: ((lo >> (7 - i)) % 2) * 2 + ((hi >> (7 - i)) % 2),
                        palette: 0,
                        priority: 0,
                    })
                };

                match fetcher {
                    Fetcher::Bg {
                        x,
                        cached: Some(pixels),
                        obj_queued,
                        ..
                    } => {
                        if fifo.push_8(*pixels).is_ok() {
                            *fetcher = if let Some(index) = obj_queued {
                                Fetcher::Object {
                                    x: *x,
                                    progress: FETCH_STEPS,
                                    index: *index,
                                }
                            } else {
                                Fetcher::Bg {
                                    x: *x + 1,
                                    progress: FETCH_STEPS,
                                    cached: None,
                                    obj_queued: None,
                                }
                            };
                        }
                    }
                    Fetcher::Window {
                        x,
                        cached: Some(pixels),
                        obj_queued,
                        ..
                    } => {
                        if fifo.push_8(*pixels).is_ok() {
                            *fetcher = if let Some(index) = obj_queued {
                                Fetcher::Object {
                                    x: *x,
                                    progress: FETCH_STEPS,
                                    index: *index,
                                }
                            } else {
                                Fetcher::Window {
                                    x: *x + 1,
                                    progress: FETCH_STEPS,
                                    cached: None,
                                    obj_queued: None,
                                }
                            };
                        }
                    }
                    Fetcher::Bg {
                        x,
                        progress: 0,
                        obj_queued,
                        ..
                    } => {
                        //TODO CGB reads BG tilemap attrs
                        let row = (scroll_y + self.ly) as u16 >> 3;
                        let col = ((scroll_x >> 3) + *x) as u16;
                        let bg_tile_addr = self.bg_map_addr + (row << 5) + col;
                        let bg_tile = memory.read(bg_tile_addr)?;
                        let ysub = (scroll_y + self.ly) as u16 % 8;
                        let data_addr = if self.bg_w_data_addr == DATA_0_START {
                            DATA_0_START + 16 * (bg_tile as u16)
                        } else if bg_tile > 127 {
                            DATA_1_START + 16 * ((bg_tile - 127) as u16)
                        } else {
                            DATA_2_START + 16 * (bg_tile as u16)
                        } + 2 * ysub;
                        let pixels =
                            get_pixels(memory.read(data_addr)?, memory.read(data_addr + 1)?);
                        if fifo.push_8(pixels).is_ok() {
                            *fetcher = if let Some(index) = obj_queued {
                                Fetcher::Object {
                                    x: *x,
                                    progress: FETCH_STEPS,
                                    index: *index,
                                }
                            } else {
                                Fetcher::Bg {
                                    x: x_tile_last.min(*x + 1),
                                    progress: FETCH_STEPS,
                                    cached: None,
                                    obj_queued: None,
                                }
                            };
                        } else {
                            *fetcher = Fetcher::Bg {
                                x: *x,
                                progress: 0,
                                cached: Some(pixels),
                                obj_queued: None,
                            };
                        }
                    }
                    Fetcher::Window {
                        x,
                        progress: 0,
                        obj_queued,
                        ..
                    } => {
                        //TODO CGB reads window tilemap attrs
                        let w_tile_addr = self.w_map_addr + 32 * self.window_counter + *x as u16;
                        let w_tile = memory.read(w_tile_addr)?;
                        let data_addr = if self.bg_w_data_addr == DATA_0_START {
                            DATA_0_START + 16 * (w_tile as u16)
                        } else if w_tile > 127 {
                            DATA_1_START + 16 * ((w_tile - 127) as u16)
                        } else {
                            DATA_2_START + 16 * (w_tile as u16)
                        } + 2 * (self.window_counter % 8);
                        let pixels =
                            get_pixels(memory.read(data_addr)?, memory.read(data_addr + 1)?);
                        if fifo.push_8(pixels).is_ok() {
                            *fetcher = if let Some(index) = obj_queued {
                                Fetcher::Object {
                                    x: *x,
                                    progress: FETCH_STEPS,
                                    index: *index,
                                }
                            } else {
                                Fetcher::Window {
                                    x: x_tile_last.min(*x + 1),
                                    progress: FETCH_STEPS,
                                    cached: None,
                                    obj_queued: None,
                                }
                            };
                        } else {
                            *fetcher = Fetcher::Window {
                                x: *x,
                                progress: 0,
                                cached: Some(pixels),
                                obj_queued: None,
                            };
                        }
                    }
                    Fetcher::Object {
                        x,
                        progress: 0,
                        index,
                    } => {
                        let obj = oam.buffer[*index];
                        let pixels = if self.obj_height == 8 {
                            //TODO y-flip
                            let data_addr = DATA_0_START
                                + 16 * obj.tile as u16
                                + (self.ly - (obj.y.saturating_sub(16))) as u16;
                            let mut pixels =
                                get_pixels(memory.read(data_addr)?, memory.read(data_addr + 1)?);
                            if obj.x_flip {
                                pixels.reverse();
                            }
                            for pixel in &mut pixels {
                                let Pixel::Tile { color, .. } = *pixel else {
                                    unreachable!("get_pixel returns Tile")
                                };
                                *pixel = Pixel::Object {
                                    color,
                                    palette: obj.palette,
                                    priority: obj.priority,
                                };
                            }
                            pixels
                        } else {
                            todo!("16px")
                        };

                        for (i, &pixel) in pixels.iter().enumerate() {
                            let fifo_pixel = &mut fifo.buffer[(fifo.front + i) % fifo.buffer.len()];
                            //TODO reconcile priorities
                            *fifo_pixel = pixel;
                        }

                        if *in_window {
                            *fetcher = Fetcher::Window {
                                x: *x,
                                progress: FETCH_STEPS,
                                cached: None,
                                obj_queued: None,
                            };
                        } else {
                            *fetcher = Fetcher::Bg {
                                x: *x,
                                progress: FETCH_STEPS,
                                cached: None,
                                obj_queued: None,
                            };
                        }
                    }
                    Fetcher::Bg { progress, .. }
                    | Fetcher::Window { progress, .. }
                    | Fetcher::Object { progress, .. } => *progress -= 1,
                }

                // first SCX%8 columns of the scanline
                while *discard > 0 {
                    fifo.pop();
                    *discard -= 1;
                }

                let fetching_obj = matches!(
                    fetcher,
                    Fetcher::Object { .. }
                        | Fetcher::Bg {
                            obj_queued: Some(_),
                            ..
                        }
                        | Fetcher::Window {
                            obj_queued: Some(_),
                            ..
                        }
                );
                let theme = crate::frame::Theme::Classic; // TODO
                let frame_pixel = if fetching_obj {
                    // postpone fifo popping until fetcher is done with object
                    None
                } else {
                    match (fifo.pop(), self.mode) {
                        (None, _) => None,
                        (Some(Pixel::Object { color, palette, .. }), Mode::Dmg) => todo!(),
                        (Some(Pixel::Object { color, palette, .. }), Mode::Cgb) => todo!(),
                        (Some(Pixel::Tile { color, .. }), Mode::Dmg) => {
                            let bgp = memory.read(mem::BG_PALETTE_REG)?;
                            let color = bgp >> (color * 2) & 0b00000011;
                            Some(crate::frame::Pixel::from_2bit(color, theme))
                        }
                        (Some(Pixel::Tile { color, palette, .. }), Mode::Cgb) => todo!(),
                    }
                };
                if let Some(pixel) = frame_pixel {
                    self.frame.0[self.ly as usize][*x as usize].set(pixel);
                    *x += 1;
                    if *x == X_END {
                        if *in_window {
                            self.window_counter += 1;
                        }
                        self.state = State::Hblank;
                        memory.set_lock(mem::Lock::Unlocked);
                    } else {
                        if self.window_enabled && self.window_latched && !*in_window {
                            if memory.read(mem::WINDOW_X_REG)? == *x + 7 {
                                *fetcher = Fetcher::Window {
                                    x: 0,
                                    progress: FETCH_STEPS,
                                    cached: None,
                                    obj_queued: None,
                                };
                                *in_window = true;
                            }
                        }
                        if self.obj_enabled {
                            for i in 0..oam.len {
                                if *x + scroll_x == oam.buffer[i].x.saturating_sub(8) {
                                    if fifo.len >= 8 {
                                        *fetcher = Fetcher::Object {
                                            x: fetcher.get_x(),
                                            progress: FETCH_STEPS,
                                            index: i,
                                        };
                                    } else {
                                        match fetcher {
                                            Fetcher::Bg { obj_queued, .. }
                                            | Fetcher::Window { obj_queued, .. } => {
                                                *obj_queued = Some(i);
                                            }
                                            Fetcher::Object { .. } => {
                                                unreachable!("can't pop pixels during object fetch")
                                            }
                                        }
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        self.dot = (self.dot + 1) % DOT_END;
        let stat_bits = [
            true,
            self.lyc_int_enable,
            self.mode_int_enable[2],
            self.mode_int_enable[1],
            self.mode_int_enable[0],
            lyc_match,
            matches!(
                self.state,
                State::OamScan { .. } | State::FirstFetch { .. } | State::Drawing { .. }
            ),
            matches!(
                self.state,
                State::Vblank | State::FirstFetch { .. } | State::Drawing { .. }
            ),
        ];
        memory.write(
            mem::LCD_STAT_REG,
            stat_bits
                .into_iter()
                .map(u8::from)
                .fold(0u8, |acc, b| (acc << 1) | b),
        )?;
        // if LY=LYC or a mode interrupt is enabled, and the condition is met, set LCD IF
        match stat_bits {
            [_, true, _, _, _, true, _, _]
            | [_, _, true, _, _, _, true, false]
            | [_, _, _, true, _, _, false, true]
            | [_, _, _, _, true, _, false, false] => {
                memory.write(mem::IF_REG, memory.read(mem::IF_REG)? | 0b00000010)?
            }
            _ => {}
        }
        Ok(frame)
    }

    fn read_lcdc_stat(&mut self, memory: &Memory) -> Result<(), Error> {
        let lcdc = memory.read(mem::LCD_CTRL_REG)?;
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
            DATA_2_START
        } else {
            DATA_0_START
        };
        self.obj_height = if lcdc & 0b00000100 == 0 { 8 } else { 16 };
        let stat = memory.read(mem::LCD_STAT_REG)?;
        self.lyc_int_enable = stat & 0b01000000 != 0;
        self.mode_int_enable = [
            stat & 0b00001000 != 0,
            stat & 0b00010000 != 0,
            stat & 0b00100000 != 0,
        ];
        Ok(())
    }
}
