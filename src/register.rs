use std::ops::{Index, IndexMut};

#[derive(Debug)]
pub enum Reg {
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
pub enum DblReg {
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

impl Index<Reg> for RegisterSet {
    type Output = u8;

    fn index(&self, register: Reg) -> &Self::Output {
        match register {
            Reg::A => self.a(),
            Reg::F => self.f(),
            Reg::B => self.b(),
            Reg::C => self.c(),
            Reg::D => self.d(),
            Reg::E => self.e(),
            Reg::H => self.h(),
            Reg::L => self.l(),
        }
    }
}

impl IndexMut<Reg> for RegisterSet {
    fn index_mut(&mut self, register: Reg) -> &mut Self::Output {
        match register {
            Reg::A => self.a_mut(),
            Reg::F => self.f_mut(),
            Reg::B => self.b_mut(),
            Reg::C => self.c_mut(),
            Reg::D => self.d_mut(),
            Reg::E => self.e_mut(),
            Reg::H => self.h_mut(),
            Reg::L => self.l_mut(),
        }
    }
}

impl Index<DblReg> for RegisterSet {
    type Output = u16;

    fn index(&self, register: DblReg) -> &Self::Output {
        match register {
            DblReg::AF => &self.af,
            DblReg::BC => &self.bc,
            DblReg::DE => &self.de,
            DblReg::HL => &self.hl,
            DblReg::SP => &self.sp,
        }
    }
}

impl IndexMut<DblReg> for RegisterSet {
    fn index_mut(&mut self, register: DblReg) -> &mut Self::Output {
        match register {
            DblReg::AF => &mut self.af,
            DblReg::BC => &mut self.bc,
            DblReg::DE => &mut self.de,
            DblReg::HL => &mut self.hl,
            DblReg::SP => &mut self.sp,
        }
    }
}
