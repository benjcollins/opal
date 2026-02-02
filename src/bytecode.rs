use std::{collections::HashMap, hash::Hash};

use strum::{EnumIs, FromRepr};

#[repr(u8)]
#[derive(Debug, Clone, Copy, FromRepr)]
pub enum Op {
    MOV,

    IADD,
    ISUB,
    IMUL,
    IDIV,
    IMOD,

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

    JMP,

    CALL,
    RET,
}

#[derive(Debug, Clone, Copy)]
pub struct Instr(u32);

impl Instr {
    pub fn src1(self) -> Val {
        let idx = (self.0 >> 8) as u8;
        if self.0 >> 25 & 1 == 0 {
            Val::reg(idx)
        } else {
            Val::cst(idx)
        }
    }

    pub fn src2(self) -> Val {
        let idx = self.0 as u8;
        if self.0 >> 24 & 1 == 0 {
            Val::reg(idx)
        } else {
            Val::cst(idx)
        }
    }

    pub fn dst(self) -> Reg {
        Reg((self.0 >> 16) as u8)
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
}

pub struct BytecodeBuffer<L> {
    buffer: Vec<Instr>,
    jumps: Vec<(usize, L)>,
    branches: Vec<(usize, L)>,
    labels: HashMap<L, usize>,
}

pub struct InstrBuilder<'b, L> {
    bytecode_buffer: &'b mut BytecodeBuffer<L>,
}

impl<L: Hash + Eq> BytecodeBuffer<L> {
    pub fn new() -> BytecodeBuffer<L> {
        BytecodeBuffer {
            buffer: Vec::new(),
            jumps: Vec::new(),
            branches: Vec::new(),
            labels: HashMap::new(),
        }
    }
    pub fn instr<'b>(&'b mut self) -> InstrBuilder<'b, L> {
        InstrBuilder {
            bytecode_buffer: self,
        }
    }
    pub fn label(&mut self, label: L) {
        self.labels.insert(label, self.buffer.len());
    }
    pub fn finish(mut self) -> Vec<Instr> {
        for (jump_index, label) in self.jumps {
            let target_index = *self.labels.get(&label).expect("label not defined!");
            self.buffer[jump_index].0 |= (target_index as i16 - jump_index as i16) as u16 as u32;
        }
        for (branch_index, label) in self.branches {
            let target_index = *self.labels.get(&label).expect("label not defined!");
            self.buffer[branch_index].0 |= ((target_index as i8 - branch_index as i8) as u8 as u32) << 16;
        }
        self.buffer
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reg(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cst(pub u8);

impl Val {
    pub fn cst(idx: u8) -> Val {
        Val::Cst(Cst(idx))
    }
    pub fn reg(idx: u8) -> Val {
        Val::Reg(Reg(idx))
    }
}

#[derive(Debug, EnumIs)]
pub enum Val {
    Reg(Reg),
    Cst(Cst),
}

impl Val {
    fn idx(self) -> u8 {
        match self {
            Val::Reg(reg) => reg.0,
            Val::Cst(cst) => cst.0,
        }
    }
}

macro_rules! define_arith_instr {
    ($fn_name:ident, $op:ident) => {
        pub fn $fn_name(self, dst: Reg, src1: Val, src2: Val) {
            self.arith_instr(Op::$op, dst, src1, src2);
        }
    };
}

macro_rules! define_branch_instr {
    ($fn_name:ident, $op:ident) => {
        pub fn $fn_name(self, src1: Val, src2: Val, label: L) {
            self.branch_instr(Op::$op, src1, src2, label);
        }
    };
}

impl<'b, L> InstrBuilder<'b, L> {
    fn arith_instr(self, op: Op, dst: Reg, src1: Val, src2: Val) {
        self.bytecode_buffer.buffer.push(Instr(
            (op as u32) << 26
                | (src1.is_cst() as u32) << 25
                | (src2.is_cst() as u32) << 24
                | (dst.0 as u32) << 16
                | (src1.idx() as u32) << 8
                | (src2.idx() as u32),
        ));
    }

    pub fn jmp(self, label: L) {
        self.bytecode_buffer
            .jumps
            .push((self.bytecode_buffer.buffer.len(), label));
        self.bytecode_buffer.buffer.push(Instr((Op::JMP as u32) << 26));
    }

    fn branch_instr(self, op: Op, src1: Val, src2: Val, label: L) {
        self.bytecode_buffer
            .branches
            .push((self.bytecode_buffer.buffer.len(), label));
        self.bytecode_buffer.buffer.push(Instr(
            (op as u32) << 26
                | (src1.is_cst() as u32) << 25
                | (src2.is_cst() as u32) << 24
                | (src1.idx() as u32) << 8
                | (src2.idx() as u32),
        ));
    }

    pub fn mov(self, dst: Reg, src: Val) {
        self.arith_instr(Op::MOV, dst, src, Val::reg(0));
    }

    define_arith_instr!(iadd, IADD);
    define_arith_instr!(isub, ISUB);
    define_arith_instr!(imul, IMUL);
    define_arith_instr!(idiv, IDIV);
    define_arith_instr!(imod, IMOD);

    define_arith_instr!(fadd, FADD);
    define_arith_instr!(fsub, FSUB);
    define_arith_instr!(fmul, FMUL);
    define_arith_instr!(fdiv, FDIV);
    define_arith_instr!(fmod, FMOD);

    define_branch_instr!(beq, BEQ);
    define_branch_instr!(bne, BNE);
    define_branch_instr!(iblt, IBLT);
    define_branch_instr!(ible, IBLE);
    define_branch_instr!(fblt, FBLT);
    define_branch_instr!(fble, FBLE);
}
