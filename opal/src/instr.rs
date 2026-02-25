use std::fmt::{self, Formatter};

use strum::{EnumIs, FromRepr, IntoStaticStr};

#[repr(u8)]
#[derive(Debug, Clone, Copy, FromRepr, IntoStaticStr)]
pub enum Op {
    MOV,

    IADD,
    ISUB,
    IMUL,
    IDIV,
    IMOD,

    AND,
    OR,
    XOR,
    SHL,
    SHR,

    FADD,
    FSUB,
    FMUL,
    FDIV,
    FMOD,

    BEQ,
    BNE,
    IBLT,
    IBLE,
    FBLT,
    FBLE,

    SEQ,
    SNE,
    ISLT,
    ISLE,
    FSLT,
    FSLE,

    NEW_ARRAY,
    GET_ARRAY,
    SET_ARRAY,

    JMP,

    CALL,
    RET,
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

    pub fn src1(self) -> Val {
        let idx = (self.0 >> 8) as u8;
        if self.0 >> 25 & 1 == 0 {
            Val::Reg(Reg(idx))
        } else {
            Val::Cst(Cst(idx))
        }
    }
    pub fn src2(self) -> Val {
        let idx = self.0 as u8;
        if self.0 >> 24 & 1 == 0 {
            Val::Reg(Reg(idx))
        } else {
            Val::Cst(Cst(idx))
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
    pub fn set_src1(&mut self, src1: Val) {
        self.0 |= (src1.is_cst() as u32) << 25;
        self.0 |= (src1.idx() as u32) << 8;
    }
    pub fn set_src2(&mut self, src2: Val) {
        self.0 |= (src2.is_cst() as u32) << 24;
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
pub enum Val {
    Reg(Reg),
    Cst(Cst),
}

impl Val {
    pub fn idx(self) -> u8 {
        match self {
            Val::Reg(reg) => reg.0,
            Val::Cst(cst) => cst.0,
        }
    }
}

impl fmt::Display for Val {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Val::Reg(reg) => write!(f, "r({})", reg.0),
            Val::Cst(cst) => write!(f, "c({})", cst.0),
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
            Op::MOV => write!(f, " {}, {}", self.dst(), self.src1()),
            Op::IADD
            | Op::ISUB
            | Op::IMUL
            | Op::IDIV
            | Op::IMOD
            | Op::FADD
            | Op::FSUB
            | Op::FMUL
            | Op::FDIV
            | Op::FMOD
            | Op::AND
            | Op::OR
            | Op::XOR
            | Op::SHL
            | Op::SHR => write!(f, " {}, {}, {}", self.dst(), self.src1(), self.src2()),
            Op::BEQ | Op::BNE | Op::IBLT | Op::IBLE | Op::FBLT | Op::FBLE => {
                write!(f, " {}, {}, {}", self.src1(), self.src2(), self.branch_offset())
            }
            Op::SEQ | Op::SNE | Op::ISLT | Op::ISLE | Op::FSLT | Op::FSLE => {
                write!(f, " {}, {}, {}", self.dst(), self.src1(), self.src2())
            }
            Op::JMP => write!(f, " {}", self.jump_offset()),
            Op::CALL => write!(f, " {}, {}, {}", self.dst(), self.src1(), self.args_start()),
            Op::RET => write!(f, " {}", self.src1()),
            Op::NEW_ARRAY => write!(f, " {}, {}", self.dst(), self.src1()),
            Op::GET_ARRAY | Op::SET_ARRAY => write!(f, " {}, {}, {}", self.dst(), self.src1(), self.src2()),
        }?;
        write!(f, ";")
    }
}
