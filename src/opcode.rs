use crate::register::{DblReg, Reg};

#[derive(Debug)]
pub enum Opcode {
    Nop,
    Stop,
    JrNz(E8),
    JrNc(E8),
    LdDblN16(DblReg, N16),
    LdDerefA(DblReg),
    LdHliA,
    LdHldA,
    IncDbl(DblReg),
    IncReg(Reg),
    IncHlDeref,
    DecReg(Reg),
    DecHlDeref,
    LdRegN8(Reg, N8),
    LdHlDerefN8(N8),
    Rlca,
    Rla,
    Daa,
    SCF,
    LdA16DerefSp(A16),
    Jr(E8),
    JrZ(E8),
    JrC(E8),
    AddHlDbl(DblReg),
    LdADeref(DblReg),
    LdAHli,
    LdAHld,
    DecDbl(DblReg),
    Rrca,
    Rra,
    Cpl,
    Ccf,
    LdRegReg(Reg, Reg),
    LdHlDerefReg(Reg),
    LdRegHlDeref(Reg),
    Halt,
    AddAReg(Reg),
    AddAHlDeref,
    AdcAReg(Reg),
    AdcAHlDeref,
    SubAReg(Reg),
    SubAHlDeref,
    SbcAReg(Reg),
    SbcAHlDeref,
    AndAReg(Reg),
    AndAHlDeref,
    XorAReg(Reg),
    XorAHlDeref,
    OrAReg(Reg),
    OrAHlDeref,
    CpAReg(Reg),
    CpAHlDeref,
    RetNz,
    RetNc,
    LdhA8DerefA(A8),
    LdhAA8Deref(A8),
    PopDbl(DblReg),
    JpNz(A16),
    JpNc(A16),
    LdhCDerefA,
    LdhACDeref,
    Jp(A16),
    Di,
    CallNz(A16),
    CallNc(A16),
    PushDbl(DblReg),
    AddAN8(N8),
    SubAN8(N8),
    AndAN8(N8),
    OrAN8(N8),
    Rst00,
    Rst10,
    Rst20,
    Rst30,
    RetZ,
    RetC,
    AddSpE8(E8),
    LdHhlSpPlusE8(E8),
    Ret,
    Reti,
    JpHl,
    LdSpHl,
    JpZ(A16),
    JpC(A16),
    LdA16DerefA(A16),
    LdAA16Deref(A16),
    Prefix,
    Ei,
    CallZ(A16),
    CallC(A16),
    Call(A16),
    AdcAN8(N8),
    SbcAN8(N8),
    XorAN8(N8),
    CpAN8(N8),
    Rst08,
    Rst18,
    Rst28,
    Rst38,
}

#[derive(Debug)]
pub struct N8(pub u8);

#[derive(Debug)]
pub struct N16(pub u16);

#[derive(Debug)]
pub struct A8(pub u8);

#[derive(Debug)]
pub struct A16(pub u16);

#[derive(Debug)]
pub struct E8(pub i8);

#[derive(Debug)]
pub enum Flag {
    Zero,
    Subtract,
    HalfCarry,
    Carry,
}

#[derive(Debug)]
pub enum FlagMode {
    Op,
    Set,
    Reset,
    Ignore,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {}

pub fn pop(instructions: &[u8]) -> Result<(Opcode, &[u8]), Error> {
    todo!()
}

impl Opcode {
    pub fn flag_mode(&self, flag: Flag) -> FlagMode {
        todo!()
    }
}
