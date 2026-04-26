use crate::SymbolError;
use crate::{opcode::Op, system::Symbol};
use core::fmt;
use log::debug;
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
