use crate::{
    cart::Cart,
    opcode::{self, Op},
    system::Mode,
};

#[derive(Debug)]
pub struct Memory {
    mode: Mode,
    boot_rom: Vec<u8>,
    cart: Option<Cart>,
    rom: Rom,
    vram: [u8; 8 * 1024],
    sram: [u8; 8 * 1024],
    wram0: [u8; 4 * 1024],
    wramx: [[u8; 4 * 1024]; 8],
    oam: [u8; 160],
    hram: [u8; 256],
}

#[derive(Debug)]
pub struct Rom;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("during op read: {0}")]
    Op(#[from] opcode::Error),
}

impl Memory {
    pub fn init(boot_rom: Vec<u8>, mode: Mode) -> Self {
        Self {
            mode,
            boot_rom,
            cart: None,
            rom: Rom,
            vram: [0; _],
            sram: [0; _],
            wram0: [0; _],
            wramx: [[0; _]; _],
            oam: [0; _],
            hram: [0; _],
        }
    }

    pub fn load_cart(&mut self, cart: Cart) {
        self.cart = Some(cart);
    }

    pub fn read(&self, addr: u16) -> Result<u8, Error> {
        Ok(0)
    }

    pub fn read_op(&self, pc: u16) -> Result<(Op, u16), Error> {
        Ok((Op::Nop, pc))
    }

    pub fn write(&mut self, addr: u16, data: u8) -> Result<(), Error> {
        Ok(())
    }
}
