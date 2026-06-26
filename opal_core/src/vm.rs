use crate::{
    bytecode::Fun,
    instr::Opcode,
    value::{Float, Int, Value},
};

pub struct VM<'a> {
    ip: usize,
    fun: &'a Fun<'a>,
    call_stack: Vec<(&'a Fun<'a>, usize)>,
    acc: Value<'a>,
    regs: Vec<Value<'a>>,
}

impl<'a> VM<'a> {
    fn int_op(&mut self, operand: Value, op: impl Fn(Int, Int) -> Int) {
        self.acc = Value::from_int(op(operand.as_int(), self.acc.as_int()))
    }

    fn float_op(&mut self, operand: Value, op: impl Fn(Float, Float) -> Float) {
        self.acc = Value::from_float(op(self.acc.as_float(), operand.as_float()))
    }

    fn execute_next_instr(&mut self) {
        let instr = self.fun.bytecode[self.ip];
        let [opcode, operand] = instr.to_be_bytes();
        let opcode = Opcode::from_repr(opcode).unwrap();
        let index = operand as usize;

        match opcode {
            Opcode::LoadReg => self.acc = self.regs[index],
            Opcode::LoadImm => self.acc = self.fun.immediates[index].get(),
            Opcode::StoreReg => self.regs[index] = self.acc,
            Opcode::AddIntReg => self.int_op(self.regs[index], |a, b| a + b),
            Opcode::AddIntImm => self.int_op(self.fun.immediates[index].get(), |a, b| a + b),
            Opcode::AddFloatReg => self.float_op(self.regs[index], |a, b| a + b),
            Opcode::AddFloatImm => self.float_op(self.fun.immediates[index].get(), |a, b| a + b),
            Opcode::SubIntReg => self.int_op(self.regs[index], |a, b| a - b),
            Opcode::SubIntImm => self.int_op(self.fun.immediates[index].get(), |a, b| a - b),
            Opcode::SubFloatReg => self.float_op(self.regs[index], |a, b| a - b),
            Opcode::SubFloatImm => self.float_op(self.fun.immediates[index].get(), |a, b| a - b),
            Opcode::MulIntReg => self.int_op(self.regs[index], |a, b| a * b),
            Opcode::MulIntImm => self.int_op(self.fun.immediates[index].get(), |a, b| a * b),
            Opcode::MulFloatReg => self.float_op(self.regs[index], |a, b| a * b),
            Opcode::MulFloatImm => self.float_op(self.fun.immediates[index].get(), |a, b| a * b),
            Opcode::DivIntReg => self.int_op(self.regs[index], |a, b| a * b),
            Opcode::DivIntImm => self.int_op(self.fun.immediates[index].get(), |a, b| a / b),
            Opcode::DivFloatReg => self.float_op(self.regs[index], |a, b| a / b),
            Opcode::DivFloatImm => self.float_op(self.fun.immediates[index].get(), |a, b| a / b),
            Opcode::Call => self.int_op(self.fun.immediates[index].get(), |a, b| a / b),
            Opcode::Ret => self.int_op(self.regs[index], |a, b| a / b),
        }
    }
}
