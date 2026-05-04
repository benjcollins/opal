use std::fmt::{self, Formatter};

use strum::{EnumIs, FromRepr, IntoStaticStr};

#[repr(u8)]
#[derive(Debug, Clone, Copy, FromRepr, IntoStaticStr)]
pub enum Op {
    Mov,

    IAdd,
    ISub,
    IMul,
    IDiv,
    IMod,

    And,
    Or,
    XOr,
    ShiftLeft,
    ShiftRight,

    FAdd,
    FSub,
    FMul,
    FDiv,
    FMod,

    BEq,
    BNEq,
    IBLt,
    IBLte,
    FBLt,
    FBLte,

    SEq,
    SNEq,
    ISLt,
    ISLte,
    FSLt,
    FSLte,

    ArrayInit,
    ArraySet,
    ListGet,

    Jump,

    Call,
    Ret,
}

#[derive(Debug, Clone, Copy)]
pub struct Instr(u32);

impl Default for Instr {
    fn default() -> Self {
        Instr::new()
    }
}

impl Instr {
    pub fn new() -> Instr {
        Instr(0)
    }

    pub fn src1(self) -> Operand {
        let idx = (self.0 >> 8) as u8;
        if self.0 >> 25 & 1 == 0 {
            Operand::Stack(Reg(idx))
        } else {
            Operand::Const(Cst(idx))
        }
    }
    pub fn src2(self) -> Operand {
        let idx = self.0 as u8;
        if self.0 >> 24 & 1 == 0 {
            Operand::Stack(Reg(idx))
        } else {
            Operand::Const(Cst(idx))
        }
    }
    pub fn dst(self) -> Reg {
        Reg((self.0 >> 16) as u8)
    }
    pub fn args_start(self) -> u8 {
        self.0 as u8
    }
    pub fn branch_offset(self) -> i8 {
        (self.0 >> 16) as i8
    }
    pub fn jump_offset(self) -> i16 {
        self.0 as i16
    }
    pub fn op(self) -> Op {
        Op::from_repr((self.0 >> 26) as u8).expect("invalid operation!")
    }

    pub fn set_op(&mut self, op: Op) {
        self.0 |= (op as u32) << 26;
    }
    pub fn set_dst(&mut self, dst: Reg) {
        self.0 |= (dst.0 as u32) << 16;
    }
    pub fn set_src1(&mut self, src1: Operand) {
        self.0 |= (src1.is_const() as u32) << 25;
        self.0 |= (src1.idx() as u32) << 8;
    }
    pub fn set_src2(&mut self, src2: Operand) {
        self.0 |= (src2.is_const() as u32) << 24;
        self.0 |= src2.idx() as u32;
    }
    pub fn set_branch_offset(&mut self, offset: i8) {
        self.0 |= (offset as u8 as u32) << 16;
    }
    pub fn set_jump_offset(&mut self, offset: i16) {
        self.0 |= offset as u16 as u32;
    }
    pub fn set_args_start(&mut self, args_start: u8) {
        self.0 |= args_start as u32;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reg(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cst(pub u8);

#[derive(Debug, EnumIs, Clone, Copy)]
pub enum Operand {
    Stack(Reg),
    Const(Cst),
}

impl Operand {
    pub fn idx(self) -> u8 {
        match self {
            Operand::Stack(reg) => reg.0,
            Operand::Const(cst) => cst.0,
        }
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Stack(reg) => write!(f, "r({})", reg.0),
            Operand::Const(cst) => write!(f, "c({})", cst.0),
        }
    }
}

impl fmt::Display for Reg {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "r({})", self.0)
    }
}

impl fmt::Display for Instr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let op: &'static str = self.op().into();
        write!(f, "{}", op)?;
        match self.op() {
            Op::Mov => write!(f, " {}, {}", self.dst(), self.src1()),
            Op::IAdd
            | Op::ISub
            | Op::IMul
            | Op::IDiv
            | Op::IMod
            | Op::FAdd
            | Op::FSub
            | Op::FMul
            | Op::FDiv
            | Op::FMod
            | Op::And
            | Op::Or
            | Op::XOr
            | Op::ShiftLeft
            | Op::ShiftRight => write!(f, " {}, {}, {}", self.dst(), self.src1(), self.src2()),
            Op::BEq | Op::BNEq | Op::IBLt | Op::IBLte | Op::FBLt | Op::FBLte => {
                write!(f, " {}, {}, {}", self.src1(), self.src2(), self.branch_offset())
            }
            Op::SEq | Op::SNEq | Op::ISLt | Op::ISLte | Op::FSLt | Op::FSLte => {
                write!(f, " {}, {}, {}", self.dst(), self.src1(), self.src2())
            }
            Op::Jump => write!(f, " {}", self.jump_offset()),
            Op::Call => write!(f, " {}, {}, {}", self.dst(), self.src1(), self.args_start()),
            Op::Ret => write!(f, " {}", self.src1()),
            Op::ArrayInit => write!(f, " {}, {}", self.dst(), self.src1()),
            Op::ArraySet | Op::ListGet => write!(f, " {}, {}, {}", self.dst(), self.src1(), self.src2()),
        }?;
        write!(f, ";")
    }
}
