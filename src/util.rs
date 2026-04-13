use tracing::debug;

use crate::system::{Symbol, SymbolError, SymbolRead};
use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
};

pub struct Hex<T: std::fmt::Debug>(pub T);

impl<T: std::fmt::Debug> std::fmt::Debug for Hex<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02X?}", self.0)
    }
}

pub fn read_symbols(
    symbols_reader: Box<dyn SymbolRead>,
    mut breakpoints: Vec<String>,
) -> Result<HashMap<(u16, u16), Symbol>, SymbolError> {
    let reader = BufReader::new(symbols_reader);
    let mut symbol_map = HashMap::new();
    for line in reader.lines() {
        let line = line.map_err(SymbolError::Io)?;
        let mut fields = line.split_ascii_whitespace();
        let Some((bank, addr)) = fields.next().and_then(|s| s.split_once(':')) else {
            continue;
        };
        let bank = u16::from_str_radix(bank, 16).map_err(SymbolError::Parse)?;
        let addr = u16::from_str_radix(addr, 16).map_err(SymbolError::Parse)?;
        let name = fields.next().unwrap_or("n/a").into();
        let r#break = breakpoints
            .iter()
            .position(|b| b == &name)
            .map(|pos| breakpoints.remove(pos))
            .is_some();
        symbol_map.insert((bank, addr), Symbol { name, r#break });
    }
    if let Some(not_found) = breakpoints.pop() {
        Err(SymbolError::BreakpointNotFound(not_found))
    } else {
        debug!(symbols = symbol_map.len(), "symbols loaded");
        Ok(symbol_map)
    }
}
