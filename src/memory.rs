use crate::{
    cart::Cart,
    opcode::{self, Op},
    system::Mode,
};

#[derive(Debug)]
pub struct Memory {
    mode: Mode,
    boot_rom: Vec<u8>,
    cart: Cart,
    rom_bank: u8,
    vram: [u8; 8 * 1024],
    sram: [u8; 8 * 1024],
    wram0: [u8; 4 * 1024],
    wramx: [[u8; 4 * 1024]; 8],
    wramx_bank: u8,
    oam: [u8; 160],
    hram: [u8; 256],
}

#[derive(Debug)]
pub struct Rom;

#[derive(Debug)]
pub enum Error {
    Op(opcode::Error),
    LenMismatch,
    OutOfBounds,
}

impl Memory {
    pub fn init(boot_rom: Vec<u8>, cart: Cart, mode: Mode) -> Self {
        Self {
            mode,
            boot_rom,
            cart,
            rom_bank: 1,
            vram: [0; _],
            sram: [0; _],
            wram0: [0; _],
            wramx: [[0; _]; _],
            wramx_bank: 1,
            oam: [0; _],
            hram: [0; _],
        }
    }

    pub fn read(&self, addr: u16) -> Result<u8, Error> {
        self.read_inner(addr, 1).map(|mem| mem[0])
    }

    pub fn read_op(&self, pc: u16) -> Result<(Op, u16), Error> {
        let mem = self.read_inner(pc, 1)?;
        Op::read(mem)
            .map(|(op, new_mem)| (op, pc + (new_mem.len() - mem.len()) as u16))
            .map_err(Error::Op)
    }

    fn read_inner(&self, addr: u16, len: u16) -> Result<&[u8], Error> {
        match addr {
            _ => Err(Error::OutOfBounds),
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) -> Result<(), Error> {
        self.write_inner(addr, 1, &[data])
    }

    pub fn write_inner(&mut self, addr: u16, len: u16, data: &[u8]) -> Result<(), Error> {
        if data.len() != len.into() {
            return Err(Error::LenMismatch);
        }
        match addr {
            _ => Err(Error::OutOfBounds),
        }
    }
}
