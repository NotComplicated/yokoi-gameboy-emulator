#[derive(Default, Debug)]
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

    pub fn zero(&self) -> bool {
        self.f & 0b10000000 != 0
    }

    pub fn sub(&self) -> bool {
        self.f & 0b01000000 != 0
    }

    pub fn half_carry(&self) -> bool {
        self.f & 0b00100000 != 0
    }

    pub fn carry(&self) -> bool {
        self.f & 0b00010000 != 0
    }

    pub fn set_zero(&mut self, set: bool) {
        self.set_flag(set, 0b10000000);
    }

    pub fn set_sub(&mut self, set: bool) {
        self.set_flag(set, 0b01000000);
    }

    pub fn set_half_carry(&mut self, set: bool) {
        self.set_flag(set, 0b00100000);
    }

    pub fn set_carry(&mut self, set: bool) {
        self.set_flag(set, 0b00010000);
    }

    fn set_flag(&mut self, set: bool, mask: u8) {
        self.f = (self.f & !mask) + if set { mask } else { 0 };
    }
}
