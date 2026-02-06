use enumset::__internal::EnumSetTypeRepr;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("exhausted all instructions")]
    Exhausted,
    #[error("invalid instruction encountered")]
    Invalid,
}

#[derive(Debug)]
#[repr(u8)]
pub enum R8 {
    B = 0b000,
    C = 0b001,
    D = 0b010,
    E = 0b011,
    H = 0b100,
    L = 0b101,
    HlDeref = 0b110,
    A = 0b111,
}

impl R8 {
    fn from_210(opcode: u8) -> Self {
        match opcode & 0b00000111 {
            0b000 => Self::B,
            0b001 => Self::C,
            0b010 => Self::D,
            0b011 => Self::E,
            0b100 => Self::H,
            0b101 => Self::L,
            0b110 => Self::HlDeref,
            0b111 => Self::A,
            _ => unreachable!(),
        }
    }

    fn from_543(opcode: u8) -> Self {
        Self::from_210(opcode >> 3)
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum R16 {
    Bc = 0b00,
    De = 0b01,
    Hl = 0b10,
    Sp = 0b11,
}

impl R16 {
    fn from_54(opcode: u8) -> Self {
        match (opcode >> 4) & 0b00000011 {
            0b00 => Self::Bc,
            0b01 => Self::De,
            0b10 => Self::Hl,
            0b11 => Self::Sp,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum R16Stk {
    Bc = 0b00,
    De = 0b01,
    Hl = 0b10,
    Af = 0b11,
}

impl R16Stk {
    fn from_54(opcode: u8) -> Self {
        match (opcode >> 4) & 0b00000011 {
            0b00 => Self::Bc,
            0b01 => Self::De,
            0b10 => Self::Hl,
            0b11 => Self::Af,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum R16Mem {
    Bc = 0b00,
    De = 0b01,
    Hli = 0b10,
    Hld = 0b11,
}

impl R16Mem {
    fn from_54(opcode: u8) -> Self {
        match (opcode >> 4) & 0b00000011 {
            0b00 => Self::Bc,
            0b01 => Self::De,
            0b10 => Self::Hli,
            0b11 => Self::Hld,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum Cond {
    Nz = 0b00,
    Z = 0b01,
    Nc = 0b10,
    C = 0b11,
}

impl Cond {
    fn from_43(opcode: u8) -> Self {
        match (opcode >> 3) & 0b00000011 {
            0b00 => Self::Nz,
            0b01 => Self::Z,
            0b10 => Self::Nc,
            0b11 => Self::C,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct N8(u8);

impl N8 {
    fn read(instructions: &[u8]) -> Result<(Self, &[u8]), Error> {
        if let [first, rest @ ..] = instructions {
            Ok((Self(*first), rest))
        } else {
            Err(Error::Exhausted)
        }
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct N16(u16);

impl N16 {
    fn read(instructions: &[u8]) -> Result<(Self, &[u8]), Error> {
        if let [first, second, rest @ ..] = instructions {
            Ok((Self(u16::from_le_bytes([*first, *second])), rest))
        } else {
            Err(Error::Exhausted)
        }
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct A8(u8);

impl A8 {
    fn read(instructions: &[u8]) -> Result<(Self, &[u8]), Error> {
        if let [first, rest @ ..] = instructions {
            Ok((Self(*first), rest))
        } else {
            Err(Error::Exhausted)
        }
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct A16(u16);

impl A16 {
    fn read(instructions: &[u8]) -> Result<(Self, &[u8]), Error> {
        if let [first, second, rest @ ..] = instructions {
            Ok((Self(u16::from_le_bytes([*first, *second])), rest))
        } else {
            Err(Error::Exhausted)
        }
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct E8(i8);

impl E8 {
    fn read(instructions: &[u8]) -> Result<(Self, &[u8]), Error> {
        if let [first, rest @ ..] = instructions {
            Ok((Self(*first as _), rest))
        } else {
            Err(Error::Exhausted)
        }
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct B3(u8);

impl B3 {
    fn from_543(opcode: u8) -> Self {
        Self((opcode >> 3) & 0b00000111)
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Tgt3(u8);

impl Tgt3 {
    fn from_543(opcode: u8) -> Self {
        Self((opcode >> 3) & 0b00000111)
    }
}

#[derive(Debug)]
pub enum Op {
    Nop,
    LdR16N16(R16, N16),
    LdR16MemA(R16Mem),
    LdAR16Mem(R16Mem),
    LdA16Sp(A16),
    IncR16(R16),
    DecR16(R16),
    AddHlR16(R16),
    IncR8(R8),
    DecR8(R8),
    LdR8N8(R8, N8),
    Rlca,
    Rrca,
    Rla,
    Rra,
    Daa,
    Cpl,
    Scf,
    Ccf,
    JrE8(E8),
    JrCondE8(Cond, E8),
    Stop(u8),
    LdR8R8(R8, R8),
    Halt,
    AddR8(R8),
    AdcR8(R8),
    SubR8(R8),
    SbcR8(R8),
    AndR8(R8),
    XorR8(R8),
    OrR8(R8),
    CpR8(R8),
    AddN8(N8),
    AdcN8(N8),
    SubN8(N8),
    SbcN8(N8),
    AndN8(N8),
    XorN8(N8),
    OrN8(N8),
    CpN8(N8),
    RetCond(Cond),
    Ret,
    Reti,
    JpCondA16(Cond, A16),
    JpA16(A16),
    JpHl,
    CallCondA16(Cond, A16),
    CallA16(A16),
    Rst(Tgt3),
    Pop(R16Stk),
    Push(R16Stk),
    Prefix(Prefixed, R8),
    LdhCA,
    LdhA8A(A8),
    LdA16A(A16),
    LdhAC,
    LdhAA8(A8),
    LdAA16(A16),
    AddSpE8(E8),
    LdHlSpPlusE8(E8),
    LdSpHl,
    Di,
    Ei,
}

#[derive(Debug)]
pub enum Prefixed {
    Rlc,
    Rrc,
    Rl,
    Rr,
    Sla,
    Sra,
    Swap,
    Srl,
    Bit(B3),
    Res(B3),
    Set(B3),
}

#[derive(Debug)]
pub enum FlagMode {
    Op,
    Set,
    Reset,
    Ignore,
}

#[derive(Debug)]
pub struct Properties {
    duration: u8,
    zero: FlagMode,
    subtract: FlagMode,
    half_carry: FlagMode,
    carry: FlagMode,
}

impl Op {
    pub fn read(instructions: &[u8]) -> Result<(Self, &[u8]), Error> {
        let &[opcode, ref rest @ ..] = instructions else {
            return Err(Error::Exhausted);
        };
        match opcode >> 6 {
            0b00 => match opcode {
                0b00000000 => Ok((Self::Nop, rest)),
                0b00010000 => {
                    if let &[second, ref rest @ ..] = rest {
                        Ok((Self::Stop(second), rest))
                    } else {
                        Err(Error::Exhausted)
                    }
                }
                0b00011000 => {
                    let (e8, rest) = E8::read(rest)?;
                    Ok((Self::JrE8(e8), rest))
                }
                0b00000111 => Ok((Self::Rlca, rest)),
                0b00001111 => Ok((Self::Rrca, rest)),
                0b00001000 => {
                    let (a16, rest) = A16::read(rest)?;
                    Ok((Self::LdA16Sp(a16), rest))
                }
                0b00010111 => Ok((Self::Rla, rest)),
                0b00011111 => Ok((Self::Rra, rest)),
                0b00100111 => Ok((Self::Daa, rest)),
                0b00101111 => Ok((Self::Cpl, rest)),
                0b00110111 => Ok((Self::Scf, rest)),
                0b00111111 => Ok((Self::Ccf, rest)),
                _ => match opcode & 0b00001111 {
                    0b0001 => {
                        let (n16, rest) = N16::read(rest)?;
                        Ok((Self::LdR16N16(R16::from_54(opcode), n16), rest))
                    }
                    0b0010 => Ok((Self::LdR16MemA(R16Mem::from_54(opcode)), rest)),
                    0b1010 => Ok((Self::LdAR16Mem(R16Mem::from_54(opcode)), rest)),
                    0b0011 => Ok((Self::IncR16(R16::from_54(opcode)), rest)),
                    0b1011 => Ok((Self::DecR16(R16::from_54(opcode)), rest)),
                    0b1001 => Ok((Self::AddHlR16(R16::from_54(opcode)), rest)),
                    _ => match opcode & 0b00000111 {
                        0b100 => Ok((Self::IncR8(R8::from_543(opcode)), rest)),
                        0b101 => Ok((Self::DecR8(R8::from_543(opcode)), rest)),
                        0b110 => {
                            let (n8, rest) = N8::read(rest)?;
                            Ok((Self::LdR8N8(R8::from_543(opcode), n8), rest))
                        }
                        _ => {
                            if opcode & 0b00100000 != 0 {
                                let (e8, rest) = E8::read(rest)?;
                                Ok((Self::JrCondE8(Cond::from_43(opcode), e8), rest))
                            } else {
                                Err(Error::Invalid)
                            }
                        }
                    },
                },
            },
            0b01 => {
                if opcode == 0b01110110 {
                    Ok((Self::Halt, rest))
                } else {
                    Ok((
                        Self::LdR8R8(R8::from_543(opcode), R8::from_210(opcode)),
                        rest,
                    ))
                }
            }
            0b10 => {
                let r8 = R8::from_210(opcode);
                match (opcode >> 3) & 0b00111 {
                    0b000 => Ok((Self::AddR8(r8), rest)),
                    0b001 => Ok((Self::AdcR8(r8), rest)),
                    0b010 => Ok((Self::SubR8(r8), rest)),
                    0b011 => Ok((Self::SbcR8(r8), rest)),
                    0b100 => Ok((Self::AndR8(r8), rest)),
                    0b101 => Ok((Self::XorR8(r8), rest)),
                    0b110 => Ok((Self::OrR8(r8), rest)),
                    0b111 => Ok((Self::CpR8(r8), rest)),
                    _ => unreachable!(),
                }
            }
            0b11 => {
                if opcode & 0b00000111 == 0b110 {
                    let (n8, rest) = N8::read(rest)?;
                    match (opcode >> 3) & 0b00111 {
                        0b000 => Ok((Self::AddN8(n8), rest)),
                        0b001 => Ok((Self::AdcN8(n8), rest)),
                        0b010 => Ok((Self::SubN8(n8), rest)),
                        0b011 => Ok((Self::SbcN8(n8), rest)),
                        0b100 => Ok((Self::AndN8(n8), rest)),
                        0b101 => Ok((Self::XorN8(n8), rest)),
                        0b110 => Ok((Self::OrN8(n8), rest)),
                        0b111 => Ok((Self::CpN8(n8), rest)),
                        _ => unreachable!(),
                    }
                } else {
                    match opcode {
                        0b11001001 => Ok((Self::Ret, rest)),
                        0b11011001 => Ok((Self::Reti, rest)),
                        0b11000011 => {
                            let (a16, rest) = A16::read(rest)?;
                            Ok((Self::JpA16(a16), rest))
                        }
                        0b11101001 => Ok((Self::JpHl, rest)),
                        0b11001101 => {
                            let (a16, rest) = A16::read(rest)?;
                            Ok((Self::CallA16(a16), rest))
                        }
                        0b11001011 => {
                            if let &[prefixed, ref rest @ ..] = rest {
                                let r8 = R8::from_210(prefixed);
                                match prefixed >> 3 {
                                    0b00000 => Ok((Self::Prefix(Prefixed::Rlc, r8), rest)),
                                    0b00001 => Ok((Self::Prefix(Prefixed::Rrc, r8), rest)),
                                    0b00010 => Ok((Self::Prefix(Prefixed::Rl, r8), rest)),
                                    0b00011 => Ok((Self::Prefix(Prefixed::Rr, r8), rest)),
                                    0b00100 => Ok((Self::Prefix(Prefixed::Sla, r8), rest)),
                                    0b00101 => Ok((Self::Prefix(Prefixed::Sra, r8), rest)),
                                    0b00110 => Ok((Self::Prefix(Prefixed::Swap, r8), rest)),
                                    0b00111 => Ok((Self::Prefix(Prefixed::Srl, r8), rest)),
                                    _ => {
                                        let b3 = B3::from_543(prefixed);
                                        match prefixed >> 6 {
                                            0b01 => Ok((Self::Prefix(Prefixed::Bit(b3), r8), rest)),
                                            0b10 => Ok((Self::Prefix(Prefixed::Res(b3), r8), rest)),
                                            0b11 => Ok((Self::Prefix(Prefixed::Set(b3), r8), rest)),
                                            _ => unreachable!(),
                                        }
                                    }
                                }
                            } else {
                                Err(Error::Exhausted)
                            }
                        }
                        0b11100010 => Ok((Self::LdhCA, rest)),
                        0b11100000 => {
                            let (a8, rest) = A8::read(rest)?;
                            Ok((Self::LdhA8A(a8), rest))
                        }
                        0b11101010 => {
                            let (a16, rest) = A16::read(rest)?;
                            Ok((Self::LdA16A(a16), rest))
                        }
                        0b11110010 => Ok((Self::LdhAC, rest)),
                        0b11110000 => {
                            let (a8, rest) = A8::read(rest)?;
                            Ok((Self::LdhAA8(a8), rest))
                        }
                        0b11111010 => {
                            let (a16, rest) = A16::read(rest)?;
                            Ok((Self::LdAA16(a16), rest))
                        }
                        0b11101000 => {
                            let (e8, rest) = E8::read(rest)?;
                            Ok((Self::AddSpE8(e8), rest))
                        }
                        0b11111000 => {
                            let (e8, rest) = E8::read(rest)?;
                            Ok((Self::LdHlSpPlusE8(e8), rest))
                        }
                        0b11111001 => Ok((Self::LdSpHl, rest)),
                        0b11110011 => Ok((Self::Di, rest)),
                        0b11111011 => Ok((Self::Ei, rest)),
                        _ => match opcode & 0b00000111 {
                            0b000 => Ok((Self::RetCond(Cond::from_43(opcode)), rest)),
                            0b001 => Ok((Self::Pop(R16Stk::from_54(opcode)), rest)),
                            0b010 => {
                                let (a16, rest) = A16::read(rest)?;
                                Ok((Self::JpCondA16(Cond::from_43(opcode), a16), rest))
                            }
                            0b100 => {
                                let (a16, rest) = A16::read(rest)?;
                                Ok((Self::CallCondA16(Cond::from_43(opcode), a16), rest))
                            }
                            0b101 => Ok((Self::Push(R16Stk::from_54(opcode)), rest)),
                            0b111 => Ok((Self::Rst(Tgt3::from_543(opcode)), rest)),
                            _ => Err(Error::Invalid),
                        },
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn properties(&self) -> Properties {
        todo!()
    }
}
