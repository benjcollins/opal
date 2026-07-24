use derive_more::From;
use strum::FromRepr;

use crate::value::Value;

#[derive(FromRepr)]
#[repr(u8)]
pub enum Opcode {
    LoadReg,
    LoadImm,
    StoreReg,

    AddIntReg,
    AddIntImm,
    AddFloatReg,
    AddFloatImm,

    SubIntReg,
    SubIntImm,
    SubFloatReg,
    SubFloatImm,

    MulIntReg,
    MulIntImm,
    MulFloatReg,
    MulFloatImm,

    DivIntReg,
    DivIntImm,
    DivFloatReg,
    DivFloatImm,

    Call,
    Ret,
}

#[derive(Debug, Clone, Copy)]
pub struct Reg(pub u8);

#[derive(Debug, Clone, Copy)]
pub struct ImmSlot(pub u8);

#[derive(Debug, Clone, Copy, From)]
pub enum Operand<'a> {
    Reg(Reg),
    ImmValue(Value<'a>),
    ImmSlot(ImmSlot),
}

pub enum Kind {
    Int,
    Float,
    Bool,
    String,
    Unit,
    Object,
}
