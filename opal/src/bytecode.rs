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

impl Default for BytecodeBuffer<()> {
    fn default() -> Self {
        BytecodeBuffer::new()
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
            self.bytecode_buffer
                .branches
                .push((self.bytecode_buffer.buffer.len(), label));
            self.instr.set_op(Op::$op);
            self.instr.set_src1(src1);
            self.instr.set_src2(src2);
        }
    };
}

macro_rules! define_branch_instr_rev {
    ($fn_name:ident, $op:ident) => {
        pub fn $fn_name(mut self, src1: Val, src2: Val, label: L) {
            self.bytecode_buffer
                .branches
                .push((self.bytecode_buffer.buffer.len(), label));
            self.instr.set_op(Op::$op);
            self.instr.set_src1(src2);
            self.instr.set_src2(src1);
        }
    };
}

macro_rules! define_set_instr {
    ($fn_name:ident, $op:ident) => {
        pub fn $fn_name(mut self, dst: Reg, src1: Val, src2: Val) {
            self.instr.set_op(Op::$op);
            self.instr.set_dst(dst);
            self.instr.set_src1(src1);
            self.instr.set_src2(src2);
        }
    };
}

macro_rules! define_set_instr_rev {
    ($fn_name:ident, $op:ident) => {
        pub fn $fn_name(mut self, dst: Reg, src1: Val, src2: Val) {
            self.instr.set_op(Op::$op);
            self.instr.set_dst(dst);
            self.instr.set_src1(src2);
            self.instr.set_src2(src1);
        }
    };
}

impl<'b, L> InstrBuilder<'b, L> {
    pub fn jmp(mut self, label: L) {
        self.bytecode_buffer
            .jumps
            .push((self.bytecode_buffer.buffer.len(), label));
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

    pub fn ret(mut self, src: Val) {
        self.instr.set_op(Op::RET);
        self.instr.set_src1(src);
    }

    pub fn new_array(mut self, dst: Reg, length: Val) {
        self.instr.set_op(Op::NEW_ARRAY);
        self.instr.set_dst(dst);
        self.instr.set_src1(length);
    }

    pub fn get_array(mut self, dst: Reg, array: Val, index: Val) {
        self.instr.set_op(Op::GET_ARRAY);
        self.instr.set_dst(dst);
        self.instr.set_src1(array);
        self.instr.set_src2(index);
    }

    pub fn set_array(mut self, array: Reg, value: Val, index: Val) {
        self.instr.set_op(Op::SET_ARRAY);
        self.instr.set_dst(array);
        self.instr.set_src1(value);
        self.instr.set_src2(index);
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

    define_arith_instr!(and, AND);
    define_arith_instr!(or, OR);
    define_arith_instr!(xor, XOR);
    define_arith_instr!(shl, SHL);
    define_arith_instr!(shr, SHR);

    define_branch_instr!(beq, BEQ);
    define_branch_instr!(bne, BNE);
    define_branch_instr!(iblt, IBLT);
    define_branch_instr!(ible, IBLE);
    define_branch_instr!(fblt, FBLT);
    define_branch_instr!(fble, FBLE);
    define_branch_instr_rev!(ibgt, IBLT);
    define_branch_instr_rev!(ibge, IBLE);
    define_branch_instr_rev!(fbgt, FBLT);
    define_branch_instr_rev!(fbge, FBLE);

    define_set_instr!(seq, SEQ);
    define_set_instr!(sne, SNE);
    define_set_instr!(islt, ISLT);
    define_set_instr!(isle, ISLE);
    define_set_instr!(fslt, FSLT);
    define_set_instr!(fsle, FSLE);
    define_set_instr_rev!(isgt, ISLT);
    define_set_instr_rev!(isge, ISLE);
    define_set_instr_rev!(fsgt, FSLT);
    define_set_instr_rev!(fsge, FSLE);
}
