use derive_more::From;

use crate::value::Value;

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
pub enum Operand {
    Reg(Reg),
    ImmValue(Value),
    ImmSlot(ImmSlot),
}
