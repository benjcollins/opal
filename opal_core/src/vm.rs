use std::ops::Add;

use crate::{
    instr::Opcode,
    value::{Float, Int, Value},
};

pub struct VM<'a> {
    ip: usize,
    immediates: &'a [Value],
    bytecode: &'a [u16],
    acc: Value,
    regs: [Value; 256],
}

impl<'a> VM<'a> {
    fn int_op(&mut self, operand: Value, op: impl Fn(Int, Int) -> Int) {
        self.acc = Value::from_int(op(operand.as_int(), self.acc.as_int()))
    }

    fn float_op(&mut self, operand: Value, op: impl Fn(Float, Float) -> Float) {
        self.acc = Value::from_float(op(self.acc.as_float(), operand.as_float()))
    }

    fn execute_next_instr(&mut self) {
        let instr = self.bytecode[self.ip];
        let [opcode, operand] = instr.to_be_bytes();
        let opcode = Opcode::from_repr(opcode).unwrap();
        let index = operand as usize;

        match opcode {
            Opcode::LoadReg => self.acc = self.regs[index],
            Opcode::LoadImm => self.acc = self.immediates[index],
            Opcode::StoreReg => self.regs[index] = self.acc,
            Opcode::AddIntReg => self.int_op(self.regs[index], |a, b| a + b),
            Opcode::AddIntImm => self.int_op(self.immediates[index], |a, b| a + b),
            Opcode::AddFloatReg => self.float_op(self.regs[index], |a, b| a + b),
            Opcode::AddFloatImm => self.float_op(self.immediates[index], |a, b| a + b),
            Opcode::SubIntReg => todo!(),
            Opcode::SubIntImm => todo!(),
            Opcode::SubFloatReg => todo!(),
            Opcode::SubFloatImm => todo!(),
            Opcode::MulIntReg => todo!(),
            Opcode::MulIntImm => todo!(),
            Opcode::MulFloatReg => todo!(),
            Opcode::MulFloatImm => todo!(),
            Opcode::DivIntReg => todo!(),
            Opcode::DivIntImm => todo!(),
            Opcode::DivFloatReg => todo!(),
            Opcode::DivFloatImm => todo!(),
            Opcode::PushKind => todo!(),
            Opcode::Call => todo!(),
            Opcode::Ret => todo!(),
        }
    }
}
