use crate::{
    frame::Frame,
    memory::{self, LY_REG, Memory},
    system::Mode,
};

const LY_END: u8 = 154;
const LX_END: u16 = 456;
const VBLANK_LY_START: u8 = 144;
const OAM_SCAN_LX_START: u16 = 0;
const OAM_SCAN_LX_END: u16 = 79;
const DRAWING_LX_START: u16 = 80;
const HBLANK_LX_START_MIN: u16 = 252;
const HBLANK_LX_START_MAX: u16 = 369;
const HBLANK_LX_END: u16 = 455;

const MAP_LOWER_START: u16 = 0x9800;
const MAP_UPPER_START: u16 = 0x9C00;
const DATA_LOWER_START: u16 = 0x8000;
const DATA_UPPER_START: u16 = 0x8800;

pub struct Ppu {
    mode: Mode,
    state: State,
    ly: u8,
    lx: u16,
    wy: u8,
    enabled: bool,
    w_enabled: bool,
    obj_enabled: bool,
    bg_w_priority: bool,
    w_map_addr: u16,
    bg_map_addr: u16,
    bg_w_data_addr: u16,
    obj_height: u8,
    frame: Frame,
}

#[derive(Debug)]
enum State {
    Hblank,
    Vblank,
    OamScan {
        oam: OamBuf,
    },
    Drawing {
        oam: OamBuf,
        bg_fifo: Fifo,
        obj_fifo: Fifo,
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

#[derive(Default, Debug)]
struct Fifo {
    buffer: [Pixel; 16],
    front: usize,
    back: usize,
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

#[derive(Default, Debug)]
struct Pixel {
    color: u8,
    palette: u8,
    obj_priority: bool,
    bg_priority: bool,
}

impl Ppu {
    pub fn init(mode: Mode) -> Self {
        Self {
            mode,
            state: State::OamScan {
                oam: Default::default(),
            },
            ly: 0,
            lx: 0,
            wy: 0,
            enabled: false,
            w_enabled: false,
            obj_enabled: false,
            bg_w_priority: false,
            w_map_addr: MAP_LOWER_START,
            bg_map_addr: MAP_LOWER_START,
            bg_w_data_addr: DATA_LOWER_START,
            obj_height: 8,
            frame: Default::default(),
        }
    }

    pub fn tick(&mut self, memory: &mut Memory) -> Result<Option<Frame>, Error> {
        self.read_lcdc(memory)?;

        match &mut self.state {
            State::Hblank => {
                if self.lx == HBLANK_LX_END {
                    self.ly += 1;
                    if self.ly < VBLANK_LY_START {
                        self.state = State::OamScan {
                            oam: Default::default(),
                        };
                    } else {
                        self.state = State::Vblank;
                    }
                }
            }

            State::Vblank => {
                //TODO
            }

            State::OamScan { oam } => {
                match self.lx {
                    OAM_SCAN_LX_START => {
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
                                    palette: if self.mode == Mode::Gbc {
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

                    OAM_SCAN_LX_END => {
                        self.state = State::Drawing {
                            oam: *oam,
                            bg_fifo: Default::default(),
                            obj_fifo: Default::default(),
                        };
                        //TODO STAT mode field transitions to 11 (Mode 3).
                        //TODO No STAT IRQ for Mode 3 entry (there is no Mode 3 IRQ source).
                        memory.lock(memory::Lock::VramOam);
                        //TODO The Background Fetcher is reset and its X counter (fetcher_x) is set to SCX >> 3 = 3 >> 3 = 0 (the tilemap column to start from).
                        //TODO The fetcher's internal step counter goes to Step 1.
                        //TODO The pixel output counter (pixels_pushed) resets to 0.
                        //TODO The "discard counter" is set to SCX % 8 = 3 (3 pixels will be discarded from the first FIFO push).
                    }

                    _ => {}
                }
            }

            State::Drawing {
                oam,
                bg_fifo,
                obj_fifo,
            } => {
                //TODO
            }
        }

        self.lx = (self.lx + 1) % LX_END;
        Ok(None)
    }

    fn read_lcdc(&mut self, memory: &Memory) -> Result<(), Error> {
        let lcdc = memory.read(memory::LCD_CTRL_REG)?;
        self.enabled = lcdc & 0b10000000 != 0;
        self.w_enabled = lcdc & 0b00100000 != 0;
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
