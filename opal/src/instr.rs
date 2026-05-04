use std::fmt::{self, Formatter};

use strum::{EnumIs, FromRepr, IntoStaticStr};

#[repr(u8)]
#[derive(Debug, Clone, Copy, FromRepr, IntoStaticStr)]
pub enum Opcode {
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

    ListDefaultLength,
    ListElements,
    ListSet,
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
            Operand::Stack(StackOperand(idx))
        } else {
            Operand::Const(ConstOperand(idx))
        }
    }
    pub fn src2(self) -> Operand {
        let idx = self.0 as u8;
        if self.0 >> 24 & 1 == 0 {
            Operand::Stack(StackOperand(idx))
        } else {
            Operand::Const(ConstOperand(idx))
        }
    }
    pub fn dst(self) -> StackOperand {
        StackOperand((self.0 >> 16) as u8)
    }
    pub fn args_start(self) -> u8 {
        self.0 as u8
    }
    pub fn args_count(self) -> u8 {
        (self.0 >> 8) as u8
    }
    pub fn branch_offset(self) -> i8 {
        (self.0 >> 16) as i8
    }
    pub fn jump_offset(self) -> i16 {
        self.0 as i16
    }
    pub fn opcode(self) -> Opcode {
        Opcode::from_repr((self.0 >> 26) as u8).expect("invalid operation!")
    }

    pub fn set_opcode(&mut self, op: Opcode) {
        self.0 |= (op as u32) << 26;
    }
    pub fn set_dst(&mut self, dst: StackOperand) {
        self.0 |= (dst.0 as u32) << 16;
    }
    pub fn set_src1(&mut self, src1: Operand) {
        self.0 |= (src1.is_const() as u32) << 25;
        self.0 |= (src1.index() as u32) << 8;
    }
    pub fn set_src2(&mut self, src2: Operand) {
        self.0 |= (src2.is_const() as u32) << 24;
        self.0 |= src2.index() as u32;
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
    pub fn set_args_count(&mut self, args_count: u8) {
        self.0 |= (args_count as u32) << 8;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StackOperand(pub u8);

#[derive(Debug, Clone, Copy)]
pub struct ConstOperand(pub u8);

#[derive(Debug, EnumIs, Clone, Copy)]
pub enum Operand {
    Stack(StackOperand),
    Const(ConstOperand),
}

impl Operand {
    pub fn index(self) -> u8 {
        match self {
            Operand::Stack(StackOperand(index)) | Operand::Const(ConstOperand(index)) => index,
        }
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Stack(operand) => write!(f, "{}", operand),
            Operand::Const(operand) => write!(f, "{}", operand),
        }
    }
}

impl fmt::Display for StackOperand {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "stack({})", self.0)
    }
}

impl fmt::Display for ConstOperand {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "const({})", self.0)
    }
}

impl fmt::Display for Instr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let op: &'static str = self.opcode().into();
        write!(f, "{}", op)?;
        match self.opcode() {
            Opcode::Mov => write!(f, " {}, {}", self.dst(), self.src1()),
            Opcode::IAdd
            | Opcode::ISub
            | Opcode::IMul
            | Opcode::IDiv
            | Opcode::IMod
            | Opcode::FAdd
            | Opcode::FSub
            | Opcode::FMul
            | Opcode::FDiv
            | Opcode::FMod
            | Opcode::And
            | Opcode::Or
            | Opcode::XOr
            | Opcode::ShiftLeft
            | Opcode::ShiftRight => write!(f, " {}, {}, {}", self.dst(), self.src1(), self.src2()),
            Opcode::BEq | Opcode::BNEq | Opcode::IBLt | Opcode::IBLte | Opcode::FBLt | Opcode::FBLte => {
                write!(f, " {}, {}, {}", self.src1(), self.src2(), self.branch_offset())
            }
            Opcode::SEq | Opcode::SNEq | Opcode::ISLt | Opcode::ISLte | Opcode::FSLt | Opcode::FSLte => {
                write!(f, " {}, {}, {}", self.dst(), self.src1(), self.src2())
            }
            Opcode::Jump => write!(f, " {}", self.jump_offset()),
            Opcode::Call => write!(f, " {}, {}, {}", self.dst(), self.src1(), self.args_start()),
            Opcode::Ret => write!(f, " {}", self.src1()),
            Opcode::ListElements => write!(f, " {}, {}", self.dst(), self.src1()),
            Opcode::ListDefaultLength => todo!(),
            Opcode::ListSet | Opcode::ListGet => write!(f, " {}, {}, {}", self.dst(), self.src1(), self.src2()),
        }?;
        write!(f, ";")
    }
}
