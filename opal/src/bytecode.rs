use std::{collections::HashMap, hash::Hash};

use crate::instr::{Instr, Op, Operand, Reg};

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
    pub fn finish(mut self) -> Box<[Instr]> {
        for (jump_index, label) in self.jumps {
            let target_index = *self.labels.get(&label).expect("label not defined!");
            self.buffer[jump_index].set_jump_offset((target_index as isize - jump_index as isize) as i16);
        }
        for (branch_index, label) in self.branches {
            let target_index = *self.labels.get(&label).expect("label not defined!");
            self.buffer[branch_index].set_branch_offset((target_index as isize - branch_index as isize) as i8);
        }
        self.buffer.into_boxed_slice()
    }
}

macro_rules! define_arith_instr {
    ($fn_name:ident, $op:ident) => {
        pub fn $fn_name(mut self, dst: Reg, src1: Operand, src2: Operand) {
            self.instr.set_op(Op::$op);
            self.instr.set_dst(dst);
            self.instr.set_src1(src1);
            self.instr.set_src2(src2);
        }
    };
}

macro_rules! define_branch_instr {
    ($fn_name:ident, $op:ident) => {
        pub fn $fn_name(mut self, src1: Operand, src2: Operand, label: L) {
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
        pub fn $fn_name(mut self, src1: Operand, src2: Operand, label: L) {
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
        pub fn $fn_name(mut self, dst: Reg, src1: Operand, src2: Operand) {
            self.instr.set_op(Op::$op);
            self.instr.set_dst(dst);
            self.instr.set_src1(src1);
            self.instr.set_src2(src2);
        }
    };
}

macro_rules! define_set_instr_rev {
    ($fn_name:ident, $op:ident) => {
        pub fn $fn_name(mut self, dst: Reg, src1: Operand, src2: Operand) {
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
        self.instr.set_op(Op::Jump);
    }

    pub fn mov(mut self, dst: Reg, src: Operand) {
        self.instr.set_op(Op::Mov);
        self.instr.set_dst(dst);
        self.instr.set_src1(src);
    }

    pub fn call(mut self, dst: Reg, fun: Operand, args_start: u8) {
        self.instr.set_op(Op::Call);
        self.instr.set_dst(dst);
        self.instr.set_src1(fun);
        self.instr.set_args_start(args_start);
    }

    pub fn ret(mut self, src: Operand) {
        self.instr.set_op(Op::Ret);
        self.instr.set_src1(src);
    }

    pub fn array_new(mut self, dst: Reg, length: Operand) {
        self.instr.set_op(Op::ArrayInit);
        self.instr.set_dst(dst);
        self.instr.set_src1(length);
    }

    pub fn array_get(mut self, dst: Reg, array: Operand, index: Operand) {
        self.instr.set_op(Op::ListGet);
        self.instr.set_dst(dst);
        self.instr.set_src1(array);
        self.instr.set_src2(index);
    }

    pub fn array_set(mut self, array: Reg, value: Operand, index: Operand) {
        self.instr.set_op(Op::ArraySet);
        self.instr.set_dst(array);
        self.instr.set_src1(value);
        self.instr.set_src2(index);
    }

    define_arith_instr!(iadd, IAdd);
    define_arith_instr!(isub, ISub);
    define_arith_instr!(imul, IMul);
    define_arith_instr!(idiv, IDiv);
    define_arith_instr!(imod, IMod);

    define_arith_instr!(fadd, FAdd);
    define_arith_instr!(fsub, FSub);
    define_arith_instr!(fmul, FMul);
    define_arith_instr!(fdiv, FDiv);
    define_arith_instr!(fmod, FMod);

    define_arith_instr!(and, And);
    define_arith_instr!(or, Or);
    define_arith_instr!(xor, XOr);
    define_arith_instr!(shl, ShiftLeft);
    define_arith_instr!(shr, ShiftRight);

    define_branch_instr!(beq, BEq);
    define_branch_instr!(bneq, BNEq);
    define_branch_instr!(iblt, IBLt);
    define_branch_instr!(iblte, IBLte);
    define_branch_instr!(fblt, FBLt);
    define_branch_instr!(fblte, FBLte);
    define_branch_instr_rev!(ibgt, IBLt);
    define_branch_instr_rev!(ibgte, IBLte);
    define_branch_instr_rev!(fbgt, FBLt);
    define_branch_instr_rev!(fbgte, FBLte);

    define_set_instr!(seq, SEq);
    define_set_instr!(sneq, SNEq);
    define_set_instr!(islt, ISLt);
    define_set_instr!(islte, ISLte);
    define_set_instr!(fslt, FSLt);
    define_set_instr!(fslte, FSLte);
    define_set_instr_rev!(isgt, ISLt);
    define_set_instr_rev!(isgte, ISLte);
    define_set_instr_rev!(fsgt, FSLt);
    define_set_instr_rev!(fsgte, FSLte);
}
