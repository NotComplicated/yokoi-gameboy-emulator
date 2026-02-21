use crate::{
    audio::Apu,
    cart::Cart,
    frame::Frame,
    memory::{self, Memory},
    opcode::*,
    register::RegisterSet,
    render::{self, Ppu},
    sound::Sound,
};

pub struct System {
    reg_set: RegisterSet,
    memory: Memory,
    current_op: Op,
    op_duration: Duration,
    ppu: Ppu,
    apu: Apu,
    state: State,
    ime: bool,
}

#[derive(Debug)]
pub enum Input {
    Start,
    Select,
    Up,
    Down,
    Left,
    Right,
    A,
    B,
}

#[derive(Debug)]
pub enum Error {
    Memory(memory::Error),
    Render(render::Error),
}

impl From<memory::Error> for Error {
    fn from(err: memory::Error) -> Self {
        Self::Memory(err)
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum Mode {
    Dmg,
    Gbc,
}

#[derive(Copy, Clone, Debug)]
enum State {
    Running,
    CondDelay(u8),
    Halted,
    Stopped,
}

#[derive(Debug)]
enum HandleOp {
    Handled,
    FalseCond,
}

impl System {
    pub fn init(boot_rom: Vec<u8>, cart: Cart) -> Result<Self, Error> {
        let mode = Mode::Dmg;
        let memory = Memory::init(boot_rom, cart, mode);
        let (current_op, pc) = memory.read_op(0)?;
        let op_duration = current_op.properties().duration;

        Ok(Self {
            reg_set: RegisterSet {
                pc,
                ..Default::default()
            },
            memory,
            current_op,
            op_duration,
            ppu: Ppu::init(mode),
            apu: Apu::init(),
            state: State::Running,
            ime: false,
        })
    }

    pub fn next_frame(&mut self, inputs: &[Input]) -> Result<(Frame, Sound), Error> {
        loop {
            if let Some((frame, sound)) = self.tick()? {
                break Ok((frame, sound));
            }
        }
    }

    fn tick(&mut self) -> Result<Option<(Frame, Sound)>, Error> {
        match (self.state, self.op_duration) {
            (State::Running, Duration::Const(1)) => {
                self.handle_op()?;
                (self.current_op, self.reg_set.pc) = self.memory.read_op(self.reg_set.pc)?;
                self.op_duration = self.current_op.properties().duration;
            }
            (State::Running, Duration::Const(ticks)) => {
                self.op_duration = Duration::Const(ticks - 1);
            }
            (State::Running, Duration::Cond(ticks, 1)) => match self.handle_op()? {
                HandleOp::Handled => {
                    self.state = State::CondDelay(ticks - 1);
                }
                HandleOp::FalseCond => {
                    (self.current_op, self.reg_set.pc) = self.memory.read_op(self.reg_set.pc)?;
                    self.op_duration = self.current_op.properties().duration;
                }
            },
            (State::Running, Duration::Cond(true_ticks, false_ticks)) => {
                self.op_duration = Duration::Cond(true_ticks - 1, false_ticks - 1);
            }
            (State::CondDelay(1), _) => {
                self.state = State::Running;
                (self.current_op, self.reg_set.pc) = self.memory.read_op(self.reg_set.pc)?;
                self.op_duration = self.current_op.properties().duration;
            }
            (State::CondDelay(ticks), _) => {
                self.state = State::CondDelay(ticks - 1);
            }
            (State::Halted | State::Stopped, _) => todo!("halt / stop handling"),
        }

        self.ppu
            .tick(&mut self.memory)
            .map(|maybe_frame| maybe_frame.map(|frame| (frame, Sound)))
            .map_err(Error::Render)
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

    fn call(&mut self, A16(a16): A16) -> Result<(), Error> {
        let [pc_upper, pc_lower] = self.reg_set.pc.to_be_bytes();
        self.reg_set.sp -= 1;
        self.memory.write(self.reg_set.pc, pc_upper)?;
        self.reg_set.sp -= 1;
        self.memory.write(self.reg_set.pc, pc_lower)?;
        self.reg_set.pc = a16;
        Ok(())
    }

    fn handle_op(&mut self) -> Result<HandleOp, Error> {
        match self.current_op {
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
                let result = self.read_r8(r8)?.wrapping_add(1);
                self.write_r8(r8, result)?;
                self.reg_set.set_zero(result == 0x00);
                self.reg_set.set_sub(false);
                self.reg_set.set_half_carry(result == 0x10);
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
                let tens = if (!self.reg_set.sub() && self.reg_set.a > 0x99) || self.reg_set.carry()
                {
                    self.reg_set.set_carry(true);
                    0x60
                } else {
                    self.reg_set.set_carry(false);
                    0x00
                };
                let ones = if (!self.reg_set.sub() && (self.reg_set.a & 0x0F) > 0x09)
                    || self.reg_set.half_carry()
                {
                    0x06
                } else {
                    0x00
                };
                let adjust = tens + ones;
                self.reg_set.a = if self.reg_set.sub() {
                    self.reg_set.a.wrapping_add(adjust)
                } else {
                    self.reg_set.a.wrapping_sub(adjust)
                };
                self.reg_set.set_zero(self.reg_set.a == 0);
                self.reg_set.set_half_carry(false);
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
                self.reg_set.pc = self.reg_set.pc.wrapping_add_signed(e8.into());
            }
            Op::JrCondE8(Cond::Nz, E8(e8)) if !self.reg_set.zero() => {
                self.reg_set.pc = self.reg_set.pc.wrapping_add_signed(e8.into());
            }
            Op::JrCondE8(Cond::C, E8(e8)) if self.reg_set.carry() => {
                self.reg_set.pc = self.reg_set.pc.wrapping_add_signed(e8.into());
            }
            Op::JrCondE8(Cond::Nc, E8(e8)) if !self.reg_set.carry() => {
                self.reg_set.pc = self.reg_set.pc.wrapping_add_signed(e8.into());
            }
            Op::JrCondE8(..) => return Ok(HandleOp::FalseCond),
            Op::Stop(_) => self.state = State::Stopped,
            Op::LdR8R8(r8_src, r8_dest) => self.write_r8(r8_dest, self.read_r8(r8_src)?)?,
            Op::Halt => self.state = State::Halted,
            Op::AddR8(r8) => {
                let operand = self.read_r8(r8)?;
                let (result, carry) = self.reg_set.a.overflowing_add(operand);
                self.reg_set.set_zero(result == 0x00);
                self.reg_set.set_sub(false);
                self.reg_set
                    .set_half_carry((self.reg_set.a & 0x0F) + (operand & 0x0F) > 0x0F);
                self.reg_set.set_carry(carry);
                self.reg_set.a = result;
            }
            Op::AdcR8(r8) => {
                let operand = self.read_r8(r8)?;
                let (result, carry) = self.reg_set.a.carrying_add(operand, self.reg_set.carry());
                self.reg_set.set_zero(result == 0x00);
                self.reg_set.set_sub(false);
                self.reg_set
                    .set_half_carry((self.reg_set.a & 0x0F) + (operand & 0x0F) > 0x0F);
                self.reg_set.set_carry(carry);
                self.reg_set.a = result;
            }
            Op::SubR8(r8) => {
                let operand = self.read_r8(r8)?;
                let (result, carry) = self.reg_set.a.overflowing_sub(operand);
                self.reg_set.set_zero(result == 0x00);
                self.reg_set.set_sub(true);
                self.reg_set
                    .set_half_carry((self.reg_set.a & 0x0F) < (operand & 0x0F));
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
                    .set_half_carry((self.reg_set.a & 0x0F) < (operand & 0x0F));
                self.reg_set.set_carry(carry);
            }
            Op::AddN8(N8(n8)) => {
                let (result, carry) = self.reg_set.a.overflowing_add(n8);
                self.reg_set.set_zero(result == 0x00);
                self.reg_set.set_sub(false);
                self.reg_set
                    .set_half_carry((self.reg_set.a & 0x0F) + (n8 & 0x0F) > 0x0F);
                self.reg_set.set_carry(carry);
                self.reg_set.a = result;
            }
            Op::AdcN8(N8(n8)) => {
                let (result, carry) = self.reg_set.a.carrying_add(n8, self.reg_set.carry());
                self.reg_set.set_zero(result == 0x00);
                self.reg_set.set_sub(false);
                self.reg_set
                    .set_half_carry((self.reg_set.a & 0x0F) + (n8 & 0x0F) > 0x0F);
                self.reg_set.set_carry(carry);
                self.reg_set.a = result;
            }
            Op::SubN8(N8(n8)) => {
                let (result, carry) = self.reg_set.a.overflowing_sub(n8);
                self.reg_set.set_zero(result == 0x00);
                self.reg_set.set_sub(true);
                self.reg_set
                    .set_half_carry((self.reg_set.a & 0x0F) < (n8 & 0x0F));
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
                    .set_half_carry((self.reg_set.a & 0x0F) < (n8 & 0x0F));
                self.reg_set.set_carry(carry);
            }
            Op::RetCond(Cond::Z) if self.reg_set.zero() => self.ret()?,
            Op::RetCond(Cond::Nz) if !self.reg_set.zero() => self.ret()?,
            Op::RetCond(Cond::C) if self.reg_set.carry() => self.ret()?,
            Op::RetCond(Cond::Nc) if !self.reg_set.carry() => self.ret()?,
            Op::RetCond(_) => return Ok(HandleOp::FalseCond),
            Op::Ret => self.ret()?,
            Op::Reti => {
                self.ret()?;
                self.ime = true;
            }
            Op::JpCondA16(Cond::Z, A16(a16)) if self.reg_set.zero() => self.reg_set.pc = a16,
            Op::JpCondA16(Cond::Nz, A16(a16)) if !self.reg_set.zero() => self.reg_set.pc = a16,
            Op::JpCondA16(Cond::C, A16(a16)) if self.reg_set.carry() => self.reg_set.pc = a16,
            Op::JpCondA16(Cond::Nc, A16(a16)) if !self.reg_set.carry() => self.reg_set.pc = a16,
            Op::JpCondA16(..) => return Ok(HandleOp::FalseCond),
            Op::JpA16(A16(a16)) => self.reg_set.pc = a16,
            Op::JpHl => self.reg_set.pc = self.reg_set.hl(),
            Op::CallCondA16(Cond::Z, a16) if self.reg_set.zero() => self.call(a16)?,
            Op::CallCondA16(Cond::Nz, a16) if !self.reg_set.zero() => self.call(a16)?,
            Op::CallCondA16(Cond::C, a16) if self.reg_set.carry() => self.call(a16)?,
            Op::CallCondA16(Cond::Nc, a16) if !self.reg_set.carry() => self.call(a16)?,
            Op::CallCondA16(..) => return Ok(HandleOp::FalseCond),
            Op::CallA16(a16) => self.call(a16)?,
            Op::Rst(Tgt3(tgt3)) => self.call(A16(u16::from_be_bytes([0x00, tgt3])))?,
            Op::Pop(r16_stk) => {
                let popped = u16::from_le_bytes([
                    self.memory.read(self.reg_set.sp)?,
                    self.memory.read(self.reg_set.sp + 1)?,
                ]);
                self.reg_set.sp += 2;
                match r16_stk {
                    R16Stk::Bc => self.reg_set.set_bc(popped),
                    R16Stk::De => self.reg_set.set_de(popped),
                    R16Stk::Hl => self.reg_set.set_hl(popped),
                    R16Stk::Af => self.reg_set.set_af(popped),
                }
            }
            Op::Push(r16_stk) => {
                let push = match r16_stk {
                    R16Stk::Bc => self.reg_set.bc(),
                    R16Stk::De => self.reg_set.de(),
                    R16Stk::Hl => self.reg_set.hl(),
                    R16Stk::Af => self.reg_set.af(),
                };
                let [upper, lower] = push.to_be_bytes();
                self.reg_set.sp -= 1;
                self.memory.write(self.reg_set.sp, upper)?;
                self.reg_set.sp -= 1;
                self.memory.write(self.reg_set.sp, lower)?;
            }
            Op::Prefix(prefixed, r8) => 'prefixed: {
                let value = self.read_r8(r8)?;
                self.reg_set.f = 0x00;
                let (result, carry) = match prefixed {
                    Prefixed::Rlc => (value.rotate_left(1), value & 0b10000000 != 0),
                    Prefixed::Rrc => (value.rotate_right(1), value % 2 == 1),
                    Prefixed::Rl => (
                        (value << 1) + self.reg_set.carry() as u8,
                        value & 0b10000000 != 0,
                    ),
                    Prefixed::Rr => (
                        ((self.reg_set.carry() as u8) << 7) + (value >> 1),
                        value % 2 == 1,
                    ),
                    Prefixed::Sla => (value << 1, value & 0b10000000 != 0),
                    Prefixed::Sra => ((value >> 1) | (value & 0b10000000), value % 2 == 1),
                    Prefixed::Swap => ((value << 4) | (value >> 4), false),
                    Prefixed::Srl => (value >> 1, value % 2 == 1),
                    Prefixed::Bit(B3(b3)) => {
                        self.reg_set.set_zero((value >> b3) % 2 == 0);
                        self.reg_set.set_half_carry(true);
                        break 'prefixed;
                    }
                    Prefixed::Res(B3(b3)) => {
                        self.write_r8(r8, value & !(1u8 << b3))?;
                        break 'prefixed;
                    }
                    Prefixed::Set(B3(b3)) => {
                        self.write_r8(r8, value | (1u8 << b3))?;
                        break 'prefixed;
                    }
                };
                self.write_r8(r8, result)?;
                self.reg_set.set_zero(result == 0);
                self.reg_set.set_carry(carry);
            }
            Op::LdhCA => self
                .memory
                .write(u16::from_be_bytes([0xFF, self.reg_set.c]), self.reg_set.a)?,
            Op::LdhA8A(A8(a8)) => self
                .memory
                .write(u16::from_be_bytes([0xFF, a8]), self.reg_set.a)?,
            Op::LdA16A(A16(a16)) => self.memory.write(a16, self.reg_set.a)?,
            Op::LdhAC => {
                self.reg_set.a = self
                    .memory
                    .read(u16::from_be_bytes([0xFF, self.reg_set.c]))?;
            }
            Op::LdhAA8(A8(a8)) => {
                self.reg_set.a = self.memory.read(u16::from_be_bytes([0xFF, a8]))?;
            }
            Op::LdAA16(A16(a16)) => self.reg_set.a = self.memory.read(a16)?,
            Op::AddSpE8(E8(e8)) => {
                let (result, carry) = self.reg_set.sp.overflowing_add_signed(e8.into());
                self.reg_set.set_sub(false);
                self.reg_set.set_half_carry(
                    (self.reg_set.sp & 0x000F).wrapping_add_signed(e8.into()) > 0x000F,
                );
                self.reg_set.set_carry(carry);
                self.reg_set.sp = result;
            }
            Op::LdHlSpPlusE8(E8(e8)) => {
                let (result, carry) = self.reg_set.sp.overflowing_add_signed(e8.into());
                self.reg_set.set_zero(false);
                self.reg_set.set_sub(false);
                self.reg_set.set_half_carry(
                    (self.reg_set.sp & 0x000F).wrapping_add_signed(e8.into()) > 0x000F,
                );
                self.reg_set.set_carry(carry);
                self.reg_set.set_hl(result);
            }
            Op::LdSpHl => self.reg_set.sp = self.reg_set.hl(),
            Op::Di => self.ime = false,
            Op::Ei => self.ime = true,
        }

        Ok(HandleOp::Handled)
    }
}
