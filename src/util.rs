use crate::SymbolError;
use crate::{opcode::Op, system::Symbol};
use core::fmt;
use log::debug;
use std::num::Wrapping;
use std::ops::{Add, Sub};
use std::{
    collections::HashMap,
    fmt::{Debug, Formatter},
};

pub struct Hex<T: Debug>(pub T);

macro_rules! hex_impl {
    ($t:ty, $size:literal) => {
        impl Debug for Hex<$t> {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                write!(f, "{:0size$X?}", self.0, size = $size)
            }
        }
    };
}
hex_impl!(u8, 2);
hex_impl!(u16, 4);
hex_impl!(u32, 8);
hex_impl!(&[u8], 2);
hex_impl!(Op, 4);

#[derive(Copy, Clone, Debug)]
pub struct ScreenPos {
    pub x: Wrapping<u8>,
    pub y: Wrapping<u8>,
}

impl ScreenPos {
    pub fn new(x: u8, y: u8) -> Self {
        Self {
            x: Wrapping(x),
            y: Wrapping(y),
        }
    }
}

impl Add for ScreenPos {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for ScreenPos {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

pub fn read_symbols(
    symbols: &str,
    mut breakpoints: Vec<String>,
) -> Result<HashMap<(u16, u16), Symbol>, SymbolError> {
    let mut symbol_map = HashMap::new();
    for line in symbols.lines() {
        let mut fields = line.split_ascii_whitespace();
        let Some((bank, addr)) = fields.next().and_then(|s| s.split_once(':')) else {
            continue;
        };
        let bank = u16::from_str_radix(bank, 16).map_err(SymbolError::Parse)?;
        let addr = u16::from_str_radix(addr, 16).map_err(SymbolError::Parse)?;
        let name = fields.next().unwrap_or("n/a").to_string();
        let r#break = breakpoints
            .iter()
            .position(|breakpoint| breakpoint == &name)
            .map(|i| breakpoints.swap_remove(i))
            .is_some();
        symbol_map.insert((bank, addr), Symbol { name, r#break });
    }
    if let Some(not_found) = breakpoints.pop() {
        Err(SymbolError::BreakpointNotFound(not_found))
    } else {
        debug!(symbols = symbol_map.len(); "symbols loaded");
        Ok(symbol_map)
    }
}
