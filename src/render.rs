use crate::{frame::Frame, memory::Memory, system};

const LY_END: u8 = 154;
const LX_END: u16 = 456;
const VBLANK_LY_START: u8 = 144;
const DRAWING_LX_START: u16 = 80;
const HBLANK_LX_START_MIN: u16 = 252;
const HBLANK_LX_START_MAX: u16 = 369;

pub struct Ppu {
    sys_mode: system::Mode,
    ppu_mode: Mode,
    ly: u8,
    lx: u16,
    bg_fifo: Fifo,
    obj_fifo: Fifo,
    frame: Frame,
}

#[derive(Copy, Clone, Debug)]
pub enum Mode {
    Hblank,
    Vblank,
    OamScan,
    Drawing,
}

#[derive(Debug)]
pub enum Error {}

#[derive(Default, Debug)]
struct Fifo([u8; 16]);

enum Draw {}

impl Ppu {
    pub fn init(mode: system::Mode) -> Self {
        Self {
            sys_mode: mode,
            ppu_mode: Mode::OamScan,
            ly: 0,
            lx: 0,
            bg_fifo: Default::default(),
            obj_fifo: Default::default(),
            frame: Default::default(),
        }
    }

    pub fn tick(&mut self, memory: &mut Memory) -> Result<Option<Frame>, Error> {
        let mut frame = None;
        match self.ppu_mode {
            Mode::Hblank if self.lx == LX_END - 1 => {
                self.ppu_mode = if self.ly == VBLANK_LY_START - 1 {
                    frame = Some(self.frame.clone());
                    Mode::Vblank
                } else {
                    Mode::OamScan
                };
                self.ly += 1;
                self.lx = 0;
            }
            Mode::Hblank => {
                self.lx += 1;
            }
            Mode::Vblank if self.lx == LX_END - 1 => {
                if self.ly == LY_END - 1 {
                    self.ppu_mode = Mode::OamScan;
                    self.ly = 0;
                } else {
                    self.ly += 1;
                }
                self.lx = 0;
            }
            Mode::Vblank => {
                self.lx += 1;
            }
            Mode::OamScan => {
                if self.lx == DRAWING_LX_START - 1 {
                    self.ppu_mode = Mode::Drawing;
                }
                self.lx += 1;
            }
            Mode::Drawing => {
                self.draw(memory)?;
                self.lx += 1;
            }
        }
        Ok(frame)
    }

    pub fn mode(&self) -> Mode {
        self.ppu_mode
    }

    fn draw(&mut self, memory: &mut Memory) -> Result<(), Error> {
        Ok(())
    }
}
