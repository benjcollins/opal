use std::{collections::HashMap, hash::Hash};

use crate::instr::{Instr, Op, Reg, Val};

pub struct BytecodeBuffer<L> {
    buffer: Vec<Instr>,
    jumps: Vec<(usize, L)>,
    branches: Vec<(usize, L)>,
    labels: HashMap<L, usize>,
}

pub struct InstrBuilder<'b, L> {
    bytecode_buffer: &'b mut BytecodeBuffer<L>,
    instr: Instr,
}

impl<'b, L> Drop for InstrBuilder<'b, L> {
    fn drop(&mut self) {
        self.bytecode_buffer.buffer.push(self.instr);
    }
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
            instr: Instr::new(),
        }
    }
    pub fn label(&mut self, label: L) {
        self.labels.insert(label, self.buffer.len());
    }
    pub fn finish(mut self) -> Vec<Instr> {
        for (jump_index, label) in self.jumps {
            let target_index = *self.labels.get(&label).expect("label not defined!");
            self.buffer[jump_index].set_jump_offset((target_index as isize - jump_index as isize) as i16);
        }
        for (branch_index, label) in self.branches {
            let target_index = *self.labels.get(&label).expect("label not defined!");
            self.buffer[branch_index].set_branch_offset((target_index as isize - branch_index as isize) as i8);
        }
        self.buffer
    }
}

macro_rules! define_arith_instr {
    ($fn_name:ident, $op:ident) => {
        pub fn $fn_name(mut self, dst: Reg, src1: Val, src2: Val) {
            self.instr.set_op(Op::$op);
            self.instr.set_dst(dst);
            self.instr.set_src1(src1);
            self.instr.set_src2(src2);
        }
    };
}

macro_rules! define_branch_instr {
    ($fn_name:ident, $op:ident) => {
        pub fn $fn_name(mut self, src1: Val, src2: Val, label: L) {
            self.bytecode_buffer.branches.push((self.bytecode_buffer.buffer.len(), label));
            self.instr.set_op(Op::$op);
            self.instr.set_src1(src1);
            self.instr.set_src2(src2);
        }
    };
}

impl<'b, L> InstrBuilder<'b, L> {
    pub fn jmp(mut self, label: L) {
        self.bytecode_buffer.jumps.push((self.bytecode_buffer.buffer.len(), label));
        self.instr.set_op(Op::JMP);
    }

    pub fn mov(mut self, dst: Reg, src: Val) {
        self.instr.set_op(Op::MOV);
        self.instr.set_dst(dst);
        self.instr.set_src1(src);
    }

    pub fn call(mut self, dst: Reg, fun: Val, args_start: u8) {
        self.instr.set_op(Op::CALL);
        self.instr.set_dst(dst);
        self.instr.set_src1(fun);
        self.instr.set_args_start(args_start);
    }

    pub fn calln(mut self, dst: Reg, fun: Val, args_start: u8) {
        self.instr.set_op(Op::CALLN);
        self.instr.set_dst(dst);
        self.instr.set_src1(fun);
        self.instr.set_args_start(args_start);
    }

    pub fn ret(mut self, src: Val) {
        self.instr.set_op(Op::RET);
        self.instr.set_src1(src);
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
