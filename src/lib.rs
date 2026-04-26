use crate::frame::Theme;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display, Formatter},
    io::Write,
};

mod audio;
mod mem;
mod opcode;
mod register;
mod render;
mod timer;
mod util;

pub mod cart;
pub mod frame;
pub mod system;

#[derive(Default)]
pub struct Input {
    pub joypad: Joypad,
    pub save_state: Option<Box<dyn Write>>,
}

#[derive(Copy, Clone, PartialEq, Default, Serialize, Deserialize, Debug)]
pub struct Joypad {
    pub start: bool,
    pub select: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub a: bool,
    pub b: bool,
}

#[derive(Copy, Clone, PartialEq, Default, Serialize, Deserialize, Debug)]
pub enum Mode {
    #[default]
    Dmg,
    Cgb,
}

#[derive(Default)]
pub struct Options {
    pub theme: Theme,
    pub short_circuit: Option<u64>,
    pub debug: bool,
    pub strict_mem_access: bool,
    pub skip_boot: bool,
    pub symbols: Option<String>,
    pub breakpoints: Vec<String>,
}

impl Display for Options {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "theme - {:?}, short_circuit - {:?}, debug - {}, strict_mem_access - {}, skip_boot - {}, symbols - {}, breakpoints - {}",
            self.theme,
            self.short_circuit,
            self.debug,
            self.strict_mem_access,
            self.skip_boot,
            self.symbols.is_some(),
            self.breakpoints.len()
        )
    }
}

#[derive(Debug)]
pub enum SymbolError {
    Io(std::io::Error),
    Parse(std::num::ParseIntError),
    BreakpointNotFound(String),
    NoneLoaded,
}
