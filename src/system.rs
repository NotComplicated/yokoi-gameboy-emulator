use crate::{
    cart::Cart,
    frontend::Frontend,
    memory::{self, Memory},
    register::RegisterSet,
};

pub struct System<F: Frontend> {
    frontend: F,
    reg_set: RegisterSet,
    memory: Memory,
}

#[derive(thiserror::Error, Debug)]
pub enum Error<FE> {
    Frontend(FE),
    Memory(#[from] memory::Error),
}

impl<F: Frontend> System<F> {
    pub fn init(frontend: F, boot_rom: Vec<u8>) -> Self {
        Self {
            frontend,
            reg_set: RegisterSet::init(),
            memory: Memory::init(boot_rom),
        }
    }

    pub fn run(&mut self, cart: Cart) -> Result<(), Error<F::Error>> {
        self.memory.load_cart(cart);
        for tick in 0.. {
            let byte = self.memory.read(self.reg_set.pc)?;
        }

        Ok(())
    }
}
