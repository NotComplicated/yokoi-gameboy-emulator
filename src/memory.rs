use crate::cart::Cart;

#[derive(Debug)]
pub struct Memory {
    boot_rom: Vec<u8>,
    cart: Option<Cart>,
    rom: Rom,
    vram: [u8; 8 * 1024],
    eram: [u8; 8 * 1024],
    wram: [u8; 8 * 1024],
    oam: [u8; 160],
    hram: [u8; 256],
}

#[derive(Debug)]
pub struct Rom;

#[derive(thiserror::Error, Debug)]
pub enum Error {}

impl Memory {
    pub fn init(boot_rom: Vec<u8>) -> Self {
        Self {
            boot_rom,
            cart: None,
            rom: Rom,
            vram: [0; _],
            eram: [0; _],
            wram: [0; _],
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

    pub fn write(&mut self, addr: u16, data: u8) -> Result<(), Error> {
        Ok(())
    }
}
