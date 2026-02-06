use std::ops::{Index, IndexMut};

#[derive(Debug)]
pub enum Register {
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(Debug)]
pub enum DblRegister {
    AF,
    BC,
    DE,
    HL,
    SP,
}

// big-endian
#[derive(Debug)]
pub struct RegisterSet {
    pub af: u16,
    pub bc: u16,
    pub de: u16,
    pub hl: u16,
    pub sp: u16,
    pub pc: u16,
}

impl RegisterSet {
    pub fn a(&self) -> &u8 {
        &bytemuck::bytes_of(&self.af)[0]
    }

    pub fn a_mut(&mut self) -> &mut u8 {
        &mut bytemuck::bytes_of_mut(&mut self.af)[0]
    }

    pub fn f(&self) -> &u8 {
        &bytemuck::bytes_of(&self.af)[1]
    }

    pub fn f_mut(&mut self) -> &mut u8 {
        &mut bytemuck::bytes_of_mut(&mut self.af)[1]
    }

    pub fn b(&self) -> &u8 {
        &bytemuck::bytes_of(&self.bc)[0]
    }

    pub fn b_mut(&mut self) -> &mut u8 {
        &mut bytemuck::bytes_of_mut(&mut self.bc)[0]
    }

    pub fn c(&self) -> &u8 {
        &bytemuck::bytes_of(&self.bc)[1]
    }

    pub fn c_mut(&mut self) -> &mut u8 {
        &mut bytemuck::bytes_of_mut(&mut self.bc)[1]
    }

    pub fn d(&self) -> &u8 {
        &bytemuck::bytes_of(&self.de)[0]
    }

    pub fn d_mut(&mut self) -> &mut u8 {
        &mut bytemuck::bytes_of_mut(&mut self.de)[0]
    }

    pub fn e(&self) -> &u8 {
        &bytemuck::bytes_of(&self.de)[1]
    }

    pub fn e_mut(&mut self) -> &mut u8 {
        &mut bytemuck::bytes_of_mut(&mut self.de)[1]
    }

    pub fn h(&self) -> &u8 {
        &bytemuck::bytes_of(&self.hl)[0]
    }

    pub fn h_mut(&mut self) -> &mut u8 {
        &mut bytemuck::bytes_of_mut(&mut self.hl)[0]
    }

    pub fn l(&self) -> &u8 {
        &bytemuck::bytes_of(&self.hl)[1]
    }

    pub fn l_mut(&mut self) -> &mut u8 {
        &mut bytemuck::bytes_of_mut(&mut self.hl)[1]
    }
}

impl Index<Register> for RegisterSet {
    type Output = u8;

    fn index(&self, register: Register) -> &Self::Output {
        match register {
            Register::A => self.a(),
            Register::F => self.f(),
            Register::B => self.b(),
            Register::C => self.c(),
            Register::D => self.d(),
            Register::E => self.e(),
            Register::H => self.h(),
            Register::L => self.l(),
        }
    }
}

impl IndexMut<Register> for RegisterSet {
    fn index_mut(&mut self, register: Register) -> &mut Self::Output {
        match register {
            Register::A => self.a_mut(),
            Register::F => self.f_mut(),
            Register::B => self.b_mut(),
            Register::C => self.c_mut(),
            Register::D => self.d_mut(),
            Register::E => self.e_mut(),
            Register::H => self.h_mut(),
            Register::L => self.l_mut(),
        }
    }
}

impl Index<DblRegister> for RegisterSet {
    type Output = u16;

    fn index(&self, register: DblRegister) -> &Self::Output {
        match register {
            DblRegister::AF => &self.af,
            DblRegister::BC => &self.bc,
            DblRegister::DE => &self.de,
            DblRegister::HL => &self.hl,
            DblRegister::SP => &self.sp,
        }
    }
}

impl IndexMut<DblRegister> for RegisterSet {
    fn index_mut(&mut self, register: DblRegister) -> &mut Self::Output {
        match register {
            DblRegister::AF => &mut self.af,
            DblRegister::BC => &mut self.bc,
            DblRegister::DE => &mut self.de,
            DblRegister::HL => &mut self.hl,
            DblRegister::SP => &mut self.sp,
        }
    }
}
