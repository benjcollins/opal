use std::ops::Neg;

use strum::EnumIs;

use crate::{
    heap::{Heap, stack::StackGuard},
    instr::{Instr, Opcode, Operand, StackOperand},
    value::Value,
};

pub struct VM<'h> {
    pub stack: StackGuard<'h>,
    pub heap: &'h Heap,
}

#[derive(Debug)]
pub struct RuntimeError;

#[derive(EnumIs)]
pub enum ControlFlow {
    Break,
    Continue,
}

pub fn value_eq(a: Value, b: Value) -> Result<bool, RuntimeError> {
    match (a, b) {
        (Value::Int(a), Value::Int(b)) => Ok(a == b),
        (Value::Float(a), Value::Float(b)) => Ok(a == b),
        (Value::Bool(a), Value::Bool(b)) => Ok(a == b),
        (Value::Unit, Value::Unit) => Ok(true),
        (Value::List(a), Value::List(b)) => {
            if a.len() != b.len() {
                return Ok(false);
            }
            for i in 0..a.len() {
                if !value_eq(a.get(i), b.get(i))? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        (
            Value::VMFun(_) | Value::HostFun(_) | Value::UnpatchedFun,
            Value::VMFun(_) | Value::HostFun(_) | Value::UnpatchedFun,
        ) => Err(RuntimeError),
        _ => panic!(),
    }
}

impl<'h> VM<'h> {
    fn read_operand(&self, val: Operand) -> Value<'h> {
        match val {
            Operand::Stack(index) => self.read_stack(index),
            Operand::Const(index) => self.stack.function().get_constant(index.0 as usize),
        }
    }
    fn read_stack(&self, reg: StackOperand) -> Value<'h> {
        self.stack.get_stack_value(self.stack.base_ptr + reg.0 as usize)
    }
    fn write_stack(&mut self, reg: StackOperand, value: Value<'h>) {
        self.stack.set_stack_value(self.stack.base_ptr + reg.0 as usize, value);
    }

    fn execute_arith_instr(
        &mut self,
        instr: Instr,
        op: impl Fn(Value<'h>, Value<'h>) -> Value<'h>,
    ) -> Result<ControlFlow, RuntimeError> {
        let src1 = self.read_operand(instr.src1());
        let src2 = self.read_operand(instr.src2());
        self.write_stack(instr.dst(), op(src1, src2));
        self.stack.instr_ptr += 1;
        Ok(ControlFlow::Continue)
    }

    fn execute_branch_instr(
        &mut self,
        instr: Instr,
        cmp: impl Fn(Value<'h>, Value<'h>) -> Result<bool, RuntimeError>,
    ) -> Result<ControlFlow, RuntimeError> {
        let src1 = self.read_operand(instr.src1());
        let src2 = self.read_operand(instr.src2());
        if cmp(src1, src2)? {
            if instr.branch_offset().is_positive() {
                self.stack.instr_ptr += instr.branch_offset() as usize
            } else {
                self.stack.instr_ptr -= instr.branch_offset().neg() as usize;
            }
        } else {
            self.stack.instr_ptr += 1;
        }
        Ok(ControlFlow::Continue)
    }

    fn execute_set_instr(
        &mut self,
        instr: Instr,
        cmp: impl Fn(Value<'h>, Value<'h>) -> Result<bool, RuntimeError>,
    ) -> Result<ControlFlow, RuntimeError> {
        let src1 = self.read_operand(instr.src1());
        let src2 = self.read_operand(instr.src2());
        self.write_stack(instr.dst(), Value::Bool(cmp(src1, src2)?));
        self.stack.instr_ptr += 1;
        Ok(ControlFlow::Continue)
    }

    pub fn execute_next_instr(&mut self) -> Result<ControlFlow, RuntimeError> {
        let instr = self.stack.function().bytecode[self.stack.instr_ptr];

        fn int_op<'h>(op: impl Fn(i64, i64) -> i64) -> impl Fn(Value, Value) -> Value<'h> {
            move |a, b| {
                let (Value::Int(a), Value::Int(b)) = (a, b) else {
                    panic!()
                };
                Value::Int(op(a, b))
            }
        }

        fn float_op<'h>(op: impl Fn(f64, f64) -> f64) -> impl Fn(Value, Value) -> Value<'h> {
            move |a, b| {
                let (Value::Float(a), Value::Float(b)) = (a, b) else {
                    panic!()
                };
                Value::Float(op(a, b))
            }
        }

        fn int_cmp(cmp: impl Fn(i64, i64) -> bool) -> impl Fn(Value, Value) -> Result<bool, RuntimeError> {
            move |a, b| {
                let (Value::Int(a), Value::Int(b)) = (a, b) else {
                    panic!("{:?}, {:?}", a, b)
                };
                Ok(cmp(a, b))
            }
        }

        fn float_cmp(cmp: impl Fn(f64, f64) -> bool) -> impl Fn(Value, Value) -> Result<bool, RuntimeError> {
            move |a, b| {
                let (Value::Float(a), Value::Float(b)) = (a, b) else {
                    panic!()
                };
                Ok(cmp(a, b))
            }
        }

        match instr.opcode() {
            Opcode::Mov => {
                let src1 = self.read_operand(instr.src1());
                self.write_stack(instr.dst(), src1);
                self.stack.instr_ptr += 1;
                Ok(ControlFlow::Continue)
            }

            Opcode::IAdd => self.execute_arith_instr(instr, int_op(|a, b| a + b)),
            Opcode::ISub => self.execute_arith_instr(instr, int_op(|a, b| a - b)),
            Opcode::IMul => self.execute_arith_instr(instr, int_op(|a, b| a * b)),
            Opcode::IDiv => self.execute_arith_instr(instr, int_op(|a, b| a / b)),
            Opcode::IMod => self.execute_arith_instr(instr, int_op(|a, b| a % b)),
            Opcode::And => self.execute_arith_instr(instr, int_op(|a, b| a & b)),
            Opcode::Or => self.execute_arith_instr(instr, int_op(|a, b| a | b)),
            Opcode::XOr => self.execute_arith_instr(instr, int_op(|a, b| a ^ b)),
            Opcode::ShiftLeft => self.execute_arith_instr(instr, int_op(|a, b| a << b)),
            Opcode::ShiftRight => self.execute_arith_instr(instr, int_op(|a, b| a >> b)),

            Opcode::FAdd => self.execute_arith_instr(instr, float_op(|a, b| a + b)),
            Opcode::FSub => self.execute_arith_instr(instr, float_op(|a, b| a - b)),
            Opcode::FMul => self.execute_arith_instr(instr, float_op(|a, b| a * b)),
            Opcode::FDiv => self.execute_arith_instr(instr, float_op(|a, b| a / b)),
            Opcode::FMod => self.execute_arith_instr(instr, float_op(|a, b| a % b)),

            Opcode::BEq => self.execute_branch_instr(instr, |a, b| value_eq(a, b)),
            Opcode::BNEq => self.execute_branch_instr(instr, |a, b| value_eq(a, b).map(|b| !b)),
            Opcode::IBLt => self.execute_branch_instr(instr, int_cmp(|a, b| a < b)),
            Opcode::IBLte => self.execute_branch_instr(instr, int_cmp(|a, b| a <= b)),
            Opcode::FBLt => self.execute_branch_instr(instr, float_cmp(|a, b| a < b)),
            Opcode::FBLte => self.execute_branch_instr(instr, float_cmp(|a, b| a <= b)),

            Opcode::SEq => self.execute_set_instr(instr, |a, b| value_eq(a, b)),
            Opcode::SNEq => self.execute_set_instr(instr, |a, b| value_eq(a, b).map(|b| !b)),
            Opcode::ISLt => self.execute_set_instr(instr, int_cmp(|a, b| a < b)),
            Opcode::ISLte => self.execute_set_instr(instr, int_cmp(|a, b| a <= b)),
            Opcode::FSLt => self.execute_set_instr(instr, float_cmp(|a, b| a < b)),
            Opcode::FSLte => self.execute_set_instr(instr, float_cmp(|a, b| a <= b)),

            Opcode::Jump => {
                if instr.jump_offset().is_positive() {
                    self.stack.instr_ptr += instr.jump_offset() as usize;
                } else {
                    self.stack.instr_ptr -= instr.jump_offset().neg() as usize;
                }
                Ok(ControlFlow::Continue)
            }
            Opcode::Call => {
                match self.read_operand(instr.src1()) {
                    Value::HostFun(fun) => {
                        self.stack.base_ptr += instr.args_start() as usize;
                        let result = fun(&self.stack)?;
                        self.stack.base_ptr -= instr.args_start() as usize;
                        self.write_stack(instr.dst(), result);
                        self.stack.instr_ptr += 1;
                    }
                    Value::VMFun(fun) => {
                        self.stack.push_call_frame(fun);
                        self.stack.base_ptr += instr.args_start() as usize;
                    }
                    _ => panic!(),
                }
                Ok(ControlFlow::Continue)
            }
            Opcode::Ret => {
                let value = self.read_operand(instr.src1());
                if !self.stack.pop_call_frame() {
                    return Ok(ControlFlow::Break);
                };
                self.stack.instr_ptr += 1;
                let call_instr = self.stack.function().bytecode[self.stack.instr_ptr - 1];
                self.stack.base_ptr -= call_instr.args_start() as usize;
                self.write_stack(call_instr.dst(), value);
                Ok(ControlFlow::Continue)
            }
            Opcode::ListElements => {
                let mut temp = vec![];
                // TODO: remove redundant allocation
                for i in instr.args_start()..instr.args_start() + instr.args_count() {
                    temp.push(self.read_operand(Operand::Stack(StackOperand(i))));
                }
                let list = self.heap.alloc_list_elements(&temp);
                self.write_stack(instr.dst(), Value::List(list));
                self.stack.instr_ptr += 1;
                Ok(ControlFlow::Continue)
            }
            Opcode::ListDefaultLength => {
                let default_ = self.read_operand(instr.src1());
                let Value::Int(size) = self.read_operand(instr.src2()) else {
                    panic!()
                };
                let list = self.heap.alloc_list_default_size(default_, size as usize);
                self.write_stack(instr.dst(), Value::List(list));
                self.stack.instr_ptr += 1;
                Ok(ControlFlow::Continue)
            }
            Opcode::ListGet => {
                let Value::List(array) = self.read_operand(instr.src1()) else {
                    panic!()
                };
                let Value::Int(index) = self.read_operand(instr.src2()) else {
                    panic!()
                };
                self.write_stack(instr.dst(), array.get(index as usize));
                self.stack.instr_ptr += 1;
                Ok(ControlFlow::Continue)
            }
            Opcode::ListSet => {
                let Value::List(list) = self.read_stack(instr.dst()) else {
                    panic!()
                };
                let Value::Int(index) = self.read_operand(instr.src2()) else {
                    panic!()
                };
                let value = self.read_operand(instr.src1());
                list.set(index as usize, value);
                self.stack.instr_ptr += 1;
                Ok(ControlFlow::Continue)
            }
        }
    }
}
