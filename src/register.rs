use crate::system::Mode;

#[derive(Debug)]
pub struct RegisterSet {
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
}

impl RegisterSet {
    pub fn init(mode: Mode) -> Self {
        let mut reg_set = Self {
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: 0xfffe,
            pc: 0x0100,
        };
        match mode {
            Mode::Dmg => {
                reg_set.set_af(0x01b0);
                reg_set.set_bc(0x0013);
                reg_set.set_de(0x00d8);
                reg_set.set_hl(0x014d);
            }
            Mode::Gbc => {
                reg_set.set_af(0x1180);
                reg_set.set_bc(0x0000);
                reg_set.set_de(0xff56);
                reg_set.set_hl(0x000d);
            }
        }
        reg_set
    }

    pub fn af(&self) -> u16 {
        u16::from_be_bytes([self.a, self.f])
    }

    pub fn bc(&self) -> u16 {
        u16::from_be_bytes([self.b, self.c])
    }

    pub fn de(&self) -> u16 {
        u16::from_be_bytes([self.d, self.e])
    }

    pub fn hl(&self) -> u16 {
        u16::from_be_bytes([self.h, self.l])
    }

    pub fn set_af(&mut self, af: u16) {
        [self.a, self.f] = af.to_be_bytes();
    }

    pub fn set_bc(&mut self, bc: u16) {
        [self.b, self.c] = bc.to_be_bytes();
    }

    pub fn set_de(&mut self, de: u16) {
        [self.d, self.e] = de.to_be_bytes();
    }

    pub fn set_hl(&mut self, hl: u16) {
        [self.h, self.l] = hl.to_be_bytes();
    }
}
