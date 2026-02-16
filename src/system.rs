use std::ops::Shl;

use crate::{
    cart::Cart,
    frame::Frame,
    memory::{self, Memory},
    opcode::{A16, Cond, E8, N8, N16, Op, R8, R16, R16Mem},
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

impl From<memory::Error> for Error {
    fn from(err: memory::Error) -> Self {
        Self::Memory(err)
    }
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

    fn read_r16(&self, r16: R16) -> u16 {
        match r16 {
            R16::Bc => self.reg_set.bc(),
            R16::De => self.reg_set.de(),
            R16::Hl => self.reg_set.hl(),
            R16::Sp => self.reg_set.sp,
        }
    }

    fn write_r16(&mut self, r16: R16, data: u16) {
        match r16 {
            R16::Bc => self.reg_set.set_bc(data),
            R16::De => self.reg_set.set_de(data),
            R16::Hl => self.reg_set.set_hl(data),
            R16::Sp => self.reg_set.sp = data,
        }
    }

    fn read_r8(&self, r8: R8) -> Result<u8, Error> {
        match r8 {
            R8::B => Ok(self.reg_set.b),
            R8::C => Ok(self.reg_set.c),
            R8::D => Ok(self.reg_set.d),
            R8::E => Ok(self.reg_set.e),
            R8::H => Ok(self.reg_set.h),
            R8::L => Ok(self.reg_set.l),
            R8::HlDeref => Ok(self.memory.read(self.reg_set.hl())?),
            R8::A => Ok(self.reg_set.a),
        }
    }

    fn write_r8(&mut self, r8: R8, data: u8) -> Result<(), Error> {
        match r8 {
            R8::B => self.reg_set.b = data,
            R8::C => self.reg_set.c = data,
            R8::D => self.reg_set.d = data,
            R8::E => self.reg_set.e = data,
            R8::H => self.reg_set.h = data,
            R8::L => self.reg_set.l = data,
            R8::HlDeref => self.memory.write(self.reg_set.hl(), data)?,
            R8::A => self.reg_set.a = data,
        }
        Ok(())
    }

    fn ret(&mut self) -> Result<(), Error> {
        let sp = self.reg_set.sp;
        self.reg_set.pc = u16::from_le_bytes([self.memory.read(sp)?, self.memory.read(sp + 1)?]);
        self.reg_set.sp += 2;
        Ok(())
    }

    fn tick(&mut self) -> Result<(), Error> {
        let (op, pc) = self.memory.read_op(self.reg_set.pc)?;
        self.reg_set.pc = pc;

        match op {
            Op::Nop => {}
            Op::LdR16N16(r16, N16(n16)) => self.write_r16(r16, n16),
            Op::LdR16MemA(r16_mem) => match r16_mem {
                R16Mem::Bc => self.memory.write(self.reg_set.bc(), self.reg_set.a)?,
                R16Mem::De => self.memory.write(self.reg_set.de(), self.reg_set.a)?,
                R16Mem::Hli => {
                    let hl = self.reg_set.hl();
                    self.reg_set.set_hl(hl + 1);
                    self.memory.write(hl, self.reg_set.a)?;
                }
                R16Mem::Hld => {
                    let hl = self.reg_set.hl();
                    self.reg_set.set_hl(hl - 1);
                    self.memory.write(hl, self.reg_set.a)?;
                }
            },
            Op::LdAR16Mem(r16_mem) => {
                self.reg_set.a = match r16_mem {
                    R16Mem::Bc => self.memory.read(self.reg_set.bc())?,
                    R16Mem::De => self.memory.read(self.reg_set.de())?,
                    R16Mem::Hli => {
                        let hl = self.reg_set.hl();
                        self.reg_set.set_hl(hl + 1);
                        self.memory.read(hl)?
                    }
                    R16Mem::Hld => {
                        let hl = self.reg_set.hl();
                        self.reg_set.set_hl(hl - 1);
                        self.memory.read(hl)?
                    }
                }
            }
            Op::LdA16Sp(A16(a16)) => self
                .memory
                .write_slice(a16, &self.reg_set.sp.to_le_bytes())?,
            Op::IncR16(r16) => self.write_r16(r16, self.read_r16(r16).wrapping_add(1)),
            Op::DecR16(r16) => self.write_r16(r16, self.read_r16(r16).wrapping_sub(1)),
            Op::AddHlR16(r16) => {
                let operand = match r16 {
                    R16::Bc => self.reg_set.bc(),
                    R16::De => self.reg_set.de(),
                    R16::Hl => self.reg_set.hl(),
                    R16::Sp => self.reg_set.sp,
                };
                let hl = self.reg_set.hl();
                let (sum, carry) = hl.overflowing_add(operand);
                self.reg_set.set_hl(sum);
                self.reg_set.set_sub(false);
                self.reg_set
                    .set_half_carry(((hl & 0x0FFF) + (operand & 0x0FFF)) & 0x1000 != 0);
                self.reg_set.set_carry(carry);
            }
            Op::IncR8(r8) => {
                let (result, carry) = self.read_r8(r8)?.overflowing_add(1);
                self.write_r8(r8, result)?;
                self.reg_set.set_zero(carry);
                self.reg_set.set_sub(false);
                self.reg_set.set_half_carry(result == 0x10);
                self.reg_set.set_carry(carry);
            }
            Op::DecR8(r8) => {
                let result = self.read_r8(r8)?.wrapping_sub(1);
                self.write_r8(r8, result)?;
                self.reg_set.set_zero(result == 0x00);
                self.reg_set.set_sub(true);
                self.reg_set.set_half_carry(result & 0x0F == 0x0F);
            }
            Op::LdR8N8(r8, N8(n8)) => self.write_r8(r8, n8)?,
            Op::Rlca => {
                self.reg_set.a = self.reg_set.a.rotate_left(1);
                self.reg_set.f = 0x00;
                self.reg_set.set_carry(self.reg_set.a % 2 == 1);
            }
            Op::Rrca => {
                self.reg_set.f = 0x00;
                self.reg_set.set_carry(self.reg_set.a % 2 == 1);
                self.reg_set.a = self.reg_set.a.rotate_right(1);
            }
            Op::Rla => {
                self.reg_set.f = 0x00;
                self.reg_set.set_carry(self.reg_set.a & 0b10000000 != 0);
                self.reg_set.a = (self.reg_set.a << 1) + self.reg_set.carry() as u8;
            }
            Op::Rra => {
                self.reg_set.f = 0x00;
                self.reg_set.set_carry(self.reg_set.a % 2 == 1);
                self.reg_set.a = ((self.reg_set.carry() as u8) << 7) + (self.reg_set.a >> 1);
            }
            Op::Daa => {
                // TODO https://blog.ollien.com/posts/gb-daa/
            }
            Op::Cpl => {
                self.reg_set.a = !self.reg_set.a;
                self.reg_set.set_sub(true);
                self.reg_set.set_half_carry(true);
            }
            Op::Scf => {
                self.reg_set.set_sub(false);
                self.reg_set.set_half_carry(false);
                self.reg_set.set_carry(true);
            }
            Op::Ccf => {
                self.reg_set.set_sub(false);
                self.reg_set.set_half_carry(false);
                self.reg_set.set_carry(!self.reg_set.carry());
            }
            Op::JrE8(E8(e8)) => self.reg_set.pc = self.reg_set.pc.wrapping_add_signed(e8.into()),
            Op::JrCondE8(Cond::Z, E8(e8)) if self.reg_set.zero() => {
                self.reg_set.pc = self.reg_set.pc.wrapping_add_signed(e8.into())
            }
            Op::JrCondE8(Cond::Nz, E8(e8)) if !self.reg_set.zero() => {
                self.reg_set.pc = self.reg_set.pc.wrapping_add_signed(e8.into())
            }
            Op::JrCondE8(Cond::C, E8(e8)) if self.reg_set.carry() => {
                self.reg_set.pc = self.reg_set.pc.wrapping_add_signed(e8.into())
            }
            Op::JrCondE8(Cond::Nc, E8(e8)) if !self.reg_set.carry() => {
                self.reg_set.pc = self.reg_set.pc.wrapping_add_signed(e8.into())
            }
            Op::JrCondE8(..) => {}
            Op::Stop(_) => {
                // TODO
            }
            Op::LdR8R8(r8_src, r8_dest) => self.write_r8(r8_dest, self.read_r8(r8_src)?)?,
            Op::Halt => {
                // TODO
            }
            Op::AddR8(r8) => {
                let operand = self.read_r8(r8)?;
                let (result, carry) = self.reg_set.a.overflowing_add(operand);
                self.reg_set.set_zero(result == 0x00);
                self.reg_set.set_sub(false);
                self.reg_set
                    .set_half_carry(((self.reg_set.a & 0x0F) + (operand & 0x0F)) & 0x10 != 0);
                self.reg_set.set_carry(carry);
                self.reg_set.a = result;
            }
            Op::AdcR8(r8) => {
                let operand = self.read_r8(r8)?;
                let (result, carry) = self.reg_set.a.carrying_add(operand, self.reg_set.carry());
                self.reg_set.set_zero(result == 0x00);
                self.reg_set.set_sub(false);
                self.reg_set
                    .set_half_carry(((self.reg_set.a & 0x0F) + (operand & 0x0F)) & 0x10 != 0);
                self.reg_set.set_carry(carry);
                self.reg_set.a = result;
            }
            Op::SubR8(r8) => {
                let operand = self.read_r8(r8)?;
                let (result, carry) = self.reg_set.a.overflowing_sub(operand);
                self.reg_set.set_zero(result == 0x00);
                self.reg_set.set_sub(true);
                self.reg_set
                    .set_half_carry((self.reg_set.a & 0x0F).overflowing_sub(operand & 0x0F).1);
                self.reg_set.set_carry(carry);
                self.reg_set.a = result;
            }
            Op::SbcR8(r8) => {
                let operand = self.read_r8(r8)?;
                let (result, carry) = self.reg_set.a.borrowing_sub(operand, self.reg_set.carry());
                self.reg_set.set_zero(result == 0x00);
                self.reg_set.set_sub(true);
                self.reg_set.set_half_carry(
                    (self.reg_set.a & 0x0F)
                        .borrowing_sub(operand & 0x0F, self.reg_set.carry())
                        .1,
                );
                self.reg_set.set_carry(carry);
                self.reg_set.a = result;
            }
            Op::AndR8(r8) => {
                self.reg_set.a &= self.read_r8(r8)?;
                self.reg_set.set_zero(self.reg_set.a == 0x00);
                self.reg_set.set_sub(false);
                self.reg_set.set_half_carry(true);
                self.reg_set.set_carry(false);
            }
            Op::XorR8(r8) => {
                self.reg_set.a ^= self.read_r8(r8)?;
                self.reg_set.f = 0x00;
                self.reg_set.set_zero(self.reg_set.a == 0x00);
            }
            Op::OrR8(r8) => {
                self.reg_set.a |= self.read_r8(r8)?;
                self.reg_set.f = 0x00;
                self.reg_set.set_zero(self.reg_set.a == 0x00);
            }
            Op::CpR8(r8) => {
                let operand = self.read_r8(r8)?;
                let (result, carry) = self.reg_set.a.overflowing_sub(operand);
                self.reg_set.set_zero(result == 0x00);
                self.reg_set.set_sub(true);
                self.reg_set
                    .set_half_carry((self.reg_set.a & 0x0F).overflowing_sub(operand & 0x0F).1);
                self.reg_set.set_carry(carry);
            }
            Op::AddN8(N8(n8)) => {
                let (result, carry) = self.reg_set.a.overflowing_add(n8);
                self.reg_set.set_zero(result == 0x00);
                self.reg_set.set_sub(false);
                self.reg_set
                    .set_half_carry(((self.reg_set.a & 0x0F) + (n8 & 0x0F)) & 0x10 != 0);
                self.reg_set.set_carry(carry);
                self.reg_set.a = result;
            }
            Op::AdcN8(N8(n8)) => {
                let (result, carry) = self.reg_set.a.carrying_add(n8, self.reg_set.carry());
                self.reg_set.set_zero(result == 0x00);
                self.reg_set.set_sub(false);
                self.reg_set
                    .set_half_carry(((self.reg_set.a & 0x0F) + (n8 & 0x0F)) & 0x10 != 0);
                self.reg_set.set_carry(carry);
                self.reg_set.a = result;
            }
            Op::SubN8(N8(n8)) => {
                let (result, carry) = self.reg_set.a.overflowing_sub(n8);
                self.reg_set.set_zero(result == 0x00);
                self.reg_set.set_sub(true);
                self.reg_set
                    .set_half_carry((self.reg_set.a & 0x0F).overflowing_sub(n8 & 0x0F).1);
                self.reg_set.set_carry(carry);
                self.reg_set.a = result;
            }
            Op::SbcN8(N8(n8)) => {
                let (result, carry) = self.reg_set.a.borrowing_sub(n8, self.reg_set.carry());
                self.reg_set.set_zero(result == 0x00);
                self.reg_set.set_sub(true);
                self.reg_set.set_half_carry(
                    (self.reg_set.a & 0x0F)
                        .borrowing_sub(n8 & 0x0F, self.reg_set.carry())
                        .1,
                );
                self.reg_set.set_carry(carry);
                self.reg_set.a = result;
            }
            Op::AndN8(N8(n8)) => {
                self.reg_set.a &= n8;
                self.reg_set.set_zero(self.reg_set.a == 0x00);
                self.reg_set.set_sub(false);
                self.reg_set.set_half_carry(true);
                self.reg_set.set_carry(false);
            }
            Op::XorN8(N8(n8)) => {
                self.reg_set.a ^= n8;
                self.reg_set.f = 0x00;
                self.reg_set.set_zero(self.reg_set.a == 0x00);
            }
            Op::OrN8(N8(n8)) => {
                self.reg_set.a |= n8;
                self.reg_set.f = 0x00;
                self.reg_set.set_zero(self.reg_set.a == 0x00);
            }
            Op::CpN8(N8(n8)) => {
                let (result, carry) = self.reg_set.a.overflowing_sub(n8);
                self.reg_set.set_zero(result == 0x00);
                self.reg_set.set_sub(true);
                self.reg_set
                    .set_half_carry((self.reg_set.a & 0x0F).overflowing_sub(n8 & 0x0F).1);
                self.reg_set.set_carry(carry);
            }
            Op::RetCond(Cond::Z) if self.reg_set.zero() => self.ret()?,
            Op::RetCond(Cond::Nz) if !self.reg_set.zero() => self.ret()?,
            Op::RetCond(Cond::C) if self.reg_set.carry() => self.ret()?,
            Op::RetCond(Cond::Nc) if !self.reg_set.carry() => self.ret()?,
            Op::RetCond(_) => {}
            Op::Ret => self.ret()?,
            Op::Reti => {
                self.ret()?;
                self.memory.write(memory::INTERRUPTS_REG, 0x01)?;
            }
            Op::JpCondA16(Cond::Z, A16(a16)) if self.reg_set.zero() => self.reg_set.pc = a16,
            Op::JpCondA16(Cond::Nz, A16(a16)) if !self.reg_set.zero() => self.reg_set.pc = a16,
            Op::JpCondA16(Cond::C, A16(a16)) if self.reg_set.carry() => self.reg_set.pc = a16,
            Op::JpCondA16(Cond::Nc, A16(a16)) if !self.reg_set.carry() => self.reg_set.pc = a16,
            Op::JpCondA16(..) => {}
            Op::JpA16(A16(a16)) => self.reg_set.pc = a16,
            Op::JpHl => self.reg_set.pc = self.reg_set.hl(),
            Op::CallCondA16(cond, a16) => {
                // TODO
            }
            Op::CallA16(a16) => {
                // TODO
            }
            Op::Rst(tgt3) => {
                // TODO
            }
            Op::Pop(r16_stk) => {
                // TODO
            }
            Op::Push(r16_stk) => {
                // TODO
            }
            Op::Prefix(prefixed, r8) => {
                // TODO
            }
            Op::LdhCA => {
                // TODO
            }
            Op::LdhA8A(a8) => {
                // TODO
            }
            Op::LdA16A(a16) => {
                // TODO
            }
            Op::LdhAC => {
                // TODO
            }
            Op::LdhAA8(a8) => {
                // TODO
            }
            Op::LdAA16(a16) => {
                // TODO
            }
            Op::AddSpE8(e8) => {
                // TODO
            }
            Op::LdHlSpPlusE8(e8) => {
                // TODO
            }
            Op::LdSpHl => self.reg_set.sp = self.reg_set.hl(),
            Op::Di => self.memory.write(memory::INTERRUPTS_REG, 0x00)?,
            Op::Ei => self.memory.write(memory::INTERRUPTS_REG, 0x01)?,
        }

        Ok(())
    }
}
