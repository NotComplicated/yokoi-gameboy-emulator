use crate::{
    frame::{self, Frame, Theme},
    mem::{self, Memory},
    render::{
        self, Error, Fifo, OamBuf, Object,
        fetcher::{self, Fetcher},
    },
    system::Mode,
};
use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize)]
pub struct Ppu {
    mode: Mode,
    #[serde(skip)]
    theme: Theme,
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
    prev_stat: u8,
    frame: Frame,
}

#[derive(Serialize, Deserialize, Debug)]
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
        px: u8,
        in_window: bool,
        fetcher: Fetcher,
        discard: u8,
    },
}

impl Ppu {
    pub fn init(mode: Mode, theme: Theme) -> Self {
        Self {
            mode,
            theme,
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
            prev_stat: 0,
            frame: Default::default(),
        }
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
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
                    memory.write_ppu(mem::LY_REG, self.ly)?;
                    if self.ly < VBLANK_LY_START {
                        if !self.window_latched {
                            self.window_latched = self.ly == memory.read(mem::WINDOW_Y_REG)?;
                        }
                        self.state = State::OamScan {
                            oam: Default::default(),
                        };
                    } else {
                        frame = Some(self.frame.clone());
                        memory.write_ppu(mem::IF_REG, memory.read(mem::IF_REG)? | 0b00000001)?;
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
                    memory.write_ppu(mem::LY_REG, self.ly)?;
                }
            }

            State::OamScan { oam } => {
                match self.dot {
                    OAM_SCAN_DOT_START => {
                        lyc_match = self.ly == memory.read(mem::LYC_REG)?;
                        memory.set_lock(mem::Lock::Oam);

                        for &[y, x, tile, flags] in memory.oam().as_chunks::<4>().0 {
                            let y_lower = y.saturating_sub(16) as u16;
                            let y_upper = y_lower + self.obj_height as u16;
                            if (y_lower..y_upper).contains(&(self.ly.into())) {
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
                            progress: fetcher::FETCH_STEPS,
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
                    px: 0,
                    in_window: false,
                    fetcher: Fetcher::Bg {
                        tile_x: 0,
                        progress: fetcher::FETCH_STEPS,
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
                px,
                in_window,
                fetcher,
                discard,
            } => {
                let x_tile_last = (X_END / 8) - 1;
                let scroll_x = memory.read(mem::SCROLL_X_REG)?;
                let scroll_y = memory.read(mem::SCROLL_Y_REG)?;

                match fetcher {
                    &mut Fetcher::Bg {
                        tile_x,
                        cached: Some(pixels),
                        obj_queued,
                        ..
                    } => {
                        if fifo.push_8(pixels).is_ok() {
                            *fetcher = if let Some(index) = obj_queued {
                                Fetcher::Object {
                                    tile_x,
                                    progress: fetcher::FETCH_STEPS,
                                    index,
                                }
                            } else {
                                Fetcher::Bg {
                                    tile_x: tile_x + 1,
                                    progress: fetcher::FETCH_STEPS,
                                    cached: None,
                                    obj_queued: None,
                                }
                            };
                        }
                    }
                    &mut Fetcher::Window {
                        tile_x,
                        cached: Some(pixels),
                        obj_queued,
                        ..
                    } => {
                        if fifo.push_8(pixels).is_ok() {
                            *fetcher = if let Some(index) = obj_queued {
                                Fetcher::Object {
                                    tile_x,
                                    progress: fetcher::FETCH_STEPS,
                                    index,
                                }
                            } else {
                                Fetcher::Window {
                                    tile_x: tile_x + 1,
                                    progress: fetcher::FETCH_STEPS,
                                    cached: None,
                                    obj_queued: None,
                                }
                            };
                        }
                    }
                    &mut Fetcher::Bg {
                        tile_x,
                        progress: 0,
                        obj_queued,
                        ..
                    } => {
                        //TODO CGB reads BG tilemap attrs
                        let y = scroll_y.wrapping_add(self.ly);
                        let row = (y >> 3) as u16;
                        let col = ((scroll_x >> 3) + tile_x) as u16 % 32;
                        let bg_tile_addr = self.bg_map_addr + (row << 5) + col;
                        let bg_tile = memory.read_ppu(bg_tile_addr)?;
                        let data_addr = if self.bg_w_data_addr == DATA_0_START {
                            DATA_0_START + 16 * (bg_tile as u16)
                        } else if bg_tile > 127 {
                            DATA_1_START + 16 * ((bg_tile - 127) as u16)
                        } else {
                            DATA_2_START + 16 * (bg_tile as u16)
                        } + 2 * (y as u16 % 8);
                        let pixels = fetcher::fetch_tile_pixels(memory, data_addr)?;
                        if fifo.push_8(pixels).is_ok() {
                            *fetcher = if let Some(index) = obj_queued {
                                Fetcher::Object {
                                    tile_x,
                                    progress: fetcher::FETCH_STEPS,
                                    index,
                                }
                            } else {
                                Fetcher::Bg {
                                    tile_x: x_tile_last.min(tile_x + 1),
                                    progress: fetcher::FETCH_STEPS,
                                    cached: None,
                                    obj_queued: None,
                                }
                            };
                        } else {
                            *fetcher = Fetcher::Bg {
                                tile_x,
                                progress: 0,
                                cached: Some(pixels),
                                obj_queued: None,
                            };
                        }
                    }
                    &mut Fetcher::Window {
                        tile_x,
                        progress: 0,
                        obj_queued,
                        ..
                    } => {
                        //TODO CGB reads window tilemap attrs
                        let w_tile_addr =
                            self.w_map_addr + 32 * self.window_counter + tile_x as u16;
                        let w_tile = memory.read_ppu(w_tile_addr)?;
                        let data_addr = if self.bg_w_data_addr == DATA_0_START {
                            DATA_0_START + 16 * (w_tile as u16)
                        } else if w_tile > 127 {
                            DATA_1_START + 16 * ((w_tile - 127) as u16)
                        } else {
                            DATA_2_START + 16 * (w_tile as u16)
                        } + 2 * (self.window_counter % 8);
                        let pixels = fetcher::fetch_tile_pixels(memory, data_addr)?;
                        if fifo.push_8(pixels).is_ok() {
                            *fetcher = if let Some(index) = obj_queued {
                                Fetcher::Object {
                                    tile_x,
                                    progress: fetcher::FETCH_STEPS,
                                    index,
                                }
                            } else {
                                Fetcher::Window {
                                    tile_x: x_tile_last.min(tile_x + 1),
                                    progress: fetcher::FETCH_STEPS,
                                    cached: None,
                                    obj_queued: None,
                                }
                            };
                        } else {
                            *fetcher = Fetcher::Window {
                                tile_x,
                                progress: 0,
                                cached: Some(pixels),
                                obj_queued: None,
                            };
                        }
                    }
                    &mut Fetcher::Object {
                        progress: 0,
                        index,
                        tile_x,
                    } => {
                        let obj = oam.buffer[index];
                        log::trace!(obj:?; "drawing object");
                        if self.mode == Mode::Cgb && obj.bank == 1 {
                            todo!("read tile from cgb bank 1")
                        }
                        let pixels =
                            {
                                let data_addr_offset = if obj.y_flip {
                                    ((self.obj_height - 1) as i16)
                                        - ((self.ly as i16) - (obj.y as i16) + 16)
                                } else {
                                    (self.ly as i16) - (obj.y as i16) + 16
                                };
                                let tile = if self.obj_height == 8 {
                                    obj.tile
                                } else {
                                    obj.tile & 0b11111110
                                } as u16;
                                let data_addr =
                                    DATA_0_START + 16 * tile + 2 * (data_addr_offset as u16);
                                let mut pixels = fetcher::fetch_tile_pixels(memory, data_addr)?
                                    .map(|pixel| render::Pixel {
                                        color: pixel.color,
                                        palette: obj.palette,
                                        priority: obj.priority.into(),
                                        from_obj: true,
                                    });
                                if obj.x_flip {
                                    pixels.reverse();
                                }
                                pixels
                            };

                        //TODO blending is not working!
                        for (i, &pixel) in pixels.iter().enumerate() {
                            let fifo_pixel = &mut fifo.buffer[(fifo.front + i) % fifo.buffer.len()];
                            // https://gbdev.io/pandocs/Tile_Maps.html#bg-to-obj-priority-in-cgb-mode
                            *fifo_pixel =
                                match (self.bg_w_priority, pixel.priority, fifo_pixel.priority) {
                                    (true, 1, 1) | (true, 1, 0) | (true, 0, 1)
                                        if fifo_pixel.color != 0 =>
                                    {
                                        *fifo_pixel
                                    }
                                    _ if pixel.color == 0 => *fifo_pixel,
                                    _ => pixel,
                                };
                        }

                        if *in_window {
                            *fetcher = Fetcher::Window {
                                tile_x,
                                progress: fetcher::FETCH_STEPS,
                                cached: None,
                                obj_queued: None,
                            };
                        } else {
                            *fetcher = Fetcher::Bg {
                                tile_x,
                                progress: fetcher::FETCH_STEPS,
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

                // postpone fifo popping until fetcher is done with object
                if !fetcher.fetching_obj()
                    && let Some(pixel) = fifo.pop()
                {
                    let frame_pixel = match (pixel, self.mode) {
                        (
                            render::Pixel {
                                color,
                                palette,
                                from_obj: true,
                                ..
                            },
                            Mode::Dmg,
                        ) => {
                            if color == 0 {
                                frame::Pixel::from_2bit(0, self.theme)
                            } else {
                                let objp = if palette == 0 {
                                    memory.read(mem::OBJ_PALETTE_0_REG)?
                                } else {
                                    memory.read(mem::OBJ_PALETTE_1_REG)?
                                };
                                let color = (objp >> (color * 2)) & 0b00000011;
                                frame::Pixel::from_2bit(color, self.theme)
                            }
                        }

                        #[expect(unused)]
                        (
                            render::Pixel {
                                color,
                                palette,
                                from_obj: true,
                                ..
                            },
                            Mode::Cgb,
                        ) => todo!("read from cgb obj palette"),

                        (_, Mode::Dmg) if !self.bg_w_priority => {
                            frame::Pixel::from_2bit(0, self.theme)
                        }

                        (render::Pixel { color, .. }, Mode::Dmg) => {
                            let bgp = memory.read(mem::BG_PALETTE_REG)?;
                            let color = (bgp >> (color * 2)) & 0b00000011;
                            frame::Pixel::from_2bit(color, self.theme)
                        }

                        #[expect(unused)]
                        (render::Pixel { color, palette, .. }, Mode::Cgb) => {
                            todo!("read from cgb bg palette")
                        }
                    };
                    self.frame.0[self.ly as usize][*px as usize].set(frame_pixel);

                    *px += 1;
                    if *px == X_END {
                        if *in_window {
                            self.window_counter += 1;
                        }
                        self.state = State::Hblank;
                        memory.set_lock(mem::Lock::Unlocked);
                    } else {
                        if self.window_enabled
                            && self.window_latched
                            && !*in_window
                            && memory.read(mem::WINDOW_X_REG)? == *px + 7
                        {
                            *fetcher = Fetcher::Window {
                                tile_x: 0,
                                progress: fetcher::FETCH_STEPS,
                                cached: None,
                                obj_queued: None,
                            };
                            *in_window = true;
                        }
                        if self.obj_enabled {
                            for i in 0..oam.len {
                                if px.wrapping_add(scroll_x) == oam.buffer[i].x.saturating_sub(8) {
                                    if fifo.len >= 8 {
                                        *fetcher = Fetcher::Object {
                                            tile_x: fetcher.tile_x(),
                                            progress: fetcher::FETCH_STEPS,
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
        let stat = stat_bits
            .into_iter()
            .map(u8::from)
            .fold(0u8, |acc, b| (acc << 1) | b);
        if stat != self.prev_stat {
            memory.write_ppu(mem::LCD_STAT_REG, stat)?;
            self.prev_stat = stat;
        }
        // if LY=LYC or a mode interrupt is enabled, and the condition is met, set LCD IF
        match stat_bits {
            [_, true, _, _, _, true, _, _]
            | [_, _, true, _, _, _, true, false]
            | [_, _, _, true, _, _, false, true]
            | [_, _, _, _, true, _, false, false] => {
                memory.write_ppu(mem::IF_REG, memory.read(mem::IF_REG)? | 0b00000010)?
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

    pub fn log_state(&self) {
        let Self {
            state,
            ly,
            dot,
            enabled,
            window_enabled,
            window_latched,
            window_counter,
            obj_enabled,
            bg_w_priority,
            w_map_addr,
            bg_map_addr,
            bg_w_data_addr,
            obj_height,
            ..
        } = self;

        log::info!("ly: {ly}, dot: {dot}, enabled: {enabled}, window_enabled: {window_enabled}");
        log::info!(
            "window_latched: {window_latched}, window_counter: {window_counter}, obj_enabled: {obj_enabled}"
        );
        log::info!(
            "bg_w_priority: {bg_w_priority}, w_map_addr: {w_map_addr:04X}, bg_map_addr: {bg_map_addr:04X}"
        );
        log::info!("bg_w_data_addr: {bg_w_data_addr:04X}, obj_height: {obj_height}");

        let oam = match state {
            State::Hblank => {
                log::info!("state: H-blank");
                None
            }
            State::Vblank => {
                log::info!("state: V-blank");
                None
            }
            State::OamScan { oam } => {
                log::info!("state: OAM scan");
                Some(oam)
            }
            State::FirstFetch { oam, progress } => {
                log::info!("state: first fetch, progress: {progress}");
                Some(oam)
            }
            State::Drawing {
                oam,
                px,
                in_window,
                discard,
                ..
            } => {
                log::info!(
                    "state: drawing, nth pixel: {px}, in_window: {in_window}, discard: {discard}"
                );
                Some(oam)
            }
        };
        if let Some(OamBuf { buffer, len }) = oam {
            for (
                i,
                Object {
                    y,
                    x,
                    tile,
                    priority,
                    y_flip,
                    x_flip,
                    palette,
                    bank,
                },
            ) in buffer[..*len].iter().enumerate()
            {
                log::info!("obj[{i}] - y: {y}, x: {x}, tile: {tile}, priority: {priority}");
                log::info!(
                    "           y_flip: {y_flip}, x_flip: {x_flip}, palette: {palette}, bank: {bank}"
                );
            }
        }
    }
}
