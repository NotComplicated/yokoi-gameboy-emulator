use crate::{frame::Frame, memory::Memory, system::Mode};

const LY_END: u8 = 154;
const LX_END: u16 = 456;
const VBLANK_LY_START: u8 = 144;
const DRAWING_LX_START: u16 = 80;
const HBLANK_LX_START_MIN: u16 = 252;
const HBLANK_LX_START_MAX: u16 = 369;

pub struct Ppu {
    mode: Mode,
    state: State,
    ly: u8,
    lx: u16,
    wy: u8,
    bg_fifo: Fifo,
    obj_fifo: Fifo,
    frame: Frame,
}

#[derive(Debug)]
enum State {
    Hblank,
    Vblank,
    OamScan,
    Drawing,
}

#[derive(Debug)]
pub enum Error {}

#[derive(Default, Debug)]
struct Fifo {
    buffer: [Pixel; 16],
    pos: usize,
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
            mode: mode,
            state: State::OamScan,
            ly: 0,
            lx: 0,
            wy: 0,
            bg_fifo: Default::default(),
            obj_fifo: Default::default(),
            frame: Default::default(),
        }
    }

    pub fn tick(&mut self, memory: &mut Memory) -> Result<Option<Frame>, Error> {
        Ok(None)
    }
}
