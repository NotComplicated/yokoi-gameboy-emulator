use crate::{
    cart::Cart,
    frame::Frame,
    memory::{self, Memory},
    register::RegisterSet,
};

pub struct System {
    reg_set: RegisterSet,
    memory: Memory,
}

#[derive(Debug)]
pub enum Mode {
    Dmg,
    Gbc,
}

#[derive(Debug)]
pub enum Error {
    Memory(memory::Error),
}

impl System {
    pub fn init(boot_rom: Vec<u8>, cart: Cart) -> Self {
        let mode = Mode::Dmg;
        Self {
            reg_set: Default::default(),
            memory: Memory::init(boot_rom, cart, mode),
        }
    }

    pub fn next_frame(&mut self) -> Result<Frame, Error> {
        Ok(todo!())
    }
}
