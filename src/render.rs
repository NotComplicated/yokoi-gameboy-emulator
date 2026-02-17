use crate::{frame::Frame, system};

pub struct Ppu {
    sys_mode: system::Mode,
    ppu_mode: Mode,
    frame: Frame,
}

pub enum Mode {
    Hblank,
    Vblank,
    OamScan,
    Drawing,
}

#[derive(Debug)]
pub enum Error {}

impl Ppu {
    pub fn init(mode: system::Mode) -> Self {
        Self {
            sys_mode: mode,
            ppu_mode: Mode::OamScan,
            frame: Default::default(),
        }
    }
    pub fn tick(&mut self) -> Result<Option<Frame>, Error> {
        Ok(None)
    }
}
