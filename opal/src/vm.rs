use std::ops::Neg;

use strum::EnumIs;

use crate::{
    heap2::{Heap, StackGuard},
    instr::{Instr, Op, Operand, Reg},
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

impl<'h> VM<'h> {
    fn read_operand(&self, val: Operand) -> Value<'h> {
        match val {
            Operand::Stack(index) => self.read_stack(index),
            Operand::Const(index) => self.stack.get_function().get_constant(index.0 as usize),
        }
    }
    fn read_stack(&self, reg: Reg) -> Value<'h> {
        self.stack.get_stack_value(self.stack.base_ptr + reg.0 as usize)
    }
    fn write_stack(&mut self, reg: Reg, value: Value<'h>) {
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
        cmp: impl Fn(Value<'h>, Value<'h>) -> bool,
    ) -> Result<ControlFlow, RuntimeError> {
        let src1 = self.read_operand(instr.src1());
        let src2 = self.read_operand(instr.src2());
        if cmp(src1, src2) {
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
        cmp: impl Fn(Value<'h>, Value<'h>) -> bool,
    ) -> Result<ControlFlow, RuntimeError> {
        let src1 = self.read_operand(instr.src1());
        let src2 = self.read_operand(instr.src2());
        self.write_stack(instr.dst(), Value::Bool(cmp(src1, src2)));
        self.stack.instr_ptr += 1;
        Ok(ControlFlow::Continue)
    }

    pub fn execute_next_instr(&mut self) -> Result<ControlFlow, RuntimeError> {
        let instr = self.stack.get_function().bytecode[self.stack.instr_ptr];

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

        fn int_cmp(cmp: impl Fn(i64, i64) -> bool) -> impl Fn(Value, Value) -> bool {
            move |a, b| {
                let (Value::Int(a), Value::Int(b)) = (a, b) else {
                    panic!()
                };
                cmp(a, b)
            }
        }

        fn float_cmp(cmp: impl Fn(f64, f64) -> bool) -> impl Fn(Value, Value) -> bool {
            move |a, b| {
                let (Value::Float(a), Value::Float(b)) = (a, b) else {
                    panic!()
                };
                cmp(a, b)
            }
        }

        match instr.op() {
            Op::Mov => {
                let src1 = self.read_operand(instr.src1());
                self.write_stack(instr.dst(), src1);
                self.stack.instr_ptr += 1;
                Ok(ControlFlow::Continue)
            }

            Op::IAdd => self.execute_arith_instr(instr, int_op(|a, b| a + b)),
            Op::ISub => self.execute_arith_instr(instr, int_op(|a, b| a - b)),
            Op::IMul => self.execute_arith_instr(instr, int_op(|a, b| a * b)),
            Op::IDiv => self.execute_arith_instr(instr, int_op(|a, b| a / b)),
            Op::IMod => self.execute_arith_instr(instr, int_op(|a, b| a % b)),
            Op::And => self.execute_arith_instr(instr, int_op(|a, b| a & b)),
            Op::Or => self.execute_arith_instr(instr, int_op(|a, b| a | b)),
            Op::XOr => self.execute_arith_instr(instr, int_op(|a, b| a ^ b)),
            Op::ShiftLeft => self.execute_arith_instr(instr, int_op(|a, b| a << b)),
            Op::ShiftRight => self.execute_arith_instr(instr, int_op(|a, b| a >> b)),

            Op::FAdd => self.execute_arith_instr(instr, float_op(|a, b| a + b)),
            Op::FSub => self.execute_arith_instr(instr, float_op(|a, b| a - b)),
            Op::FMul => self.execute_arith_instr(instr, float_op(|a, b| a * b)),
            Op::FDiv => self.execute_arith_instr(instr, float_op(|a, b| a / b)),
            Op::FMod => self.execute_arith_instr(instr, float_op(|a, b| a % b)),

            Op::BEq => self.execute_branch_instr(instr, |a, b| a == b),
            Op::BNEq => self.execute_branch_instr(instr, |a, b| a != b),
            Op::IBLt => self.execute_branch_instr(instr, int_cmp(|a, b| a < b)),
            Op::IBLte => self.execute_branch_instr(instr, int_cmp(|a, b| a <= b)),
            Op::FBLt => self.execute_branch_instr(instr, float_cmp(|a, b| a < b)),
            Op::FBLte => self.execute_branch_instr(instr, float_cmp(|a, b| a <= b)),

            Op::SEq => self.execute_set_instr(instr, |a, b| a == b),
            Op::SNEq => self.execute_set_instr(instr, |a, b| a != b),
            Op::ISLt => self.execute_set_instr(instr, int_cmp(|a, b| a < b)),
            Op::ISLte => self.execute_set_instr(instr, int_cmp(|a, b| a <= b)),
            Op::FSLt => self.execute_set_instr(instr, float_cmp(|a, b| a < b)),
            Op::FSLte => self.execute_set_instr(instr, float_cmp(|a, b| a <= b)),

            Op::Jump => {
                if instr.jump_offset().is_positive() {
                    self.stack.instr_ptr += instr.jump_offset() as usize;
                } else {
                    self.stack.instr_ptr -= instr.jump_offset().neg() as usize;
                }
                Ok(ControlFlow::Continue)
            }
            Op::Call => {
                let value = self.read_operand(instr.src1());
                match self.read_operand(instr.src1()) {
                    Value::HostFun(fun) => {
                        self.stack.base_ptr += instr.args_start() as usize;
                        self.write_stack(instr.dst(), fun(&self.stack)?);
                        self.stack.base_ptr -= instr.args_start() as usize;
                        self.stack.instr_ptr += 1;
                    }
                    Value::Fun(fun) => {
                        self.stack.push_call_frame(fun);
                        self.stack.base_ptr += instr.args_start() as usize;
                    }
                    _ => panic!(),
                }
                Ok(ControlFlow::Continue)
            }
            Op::Ret => {
                let value = self.read_operand(instr.src1());
                if !self.stack.pop_call_frame() {
                    return Ok(ControlFlow::Break);
                };
                self.stack.instr_ptr += 1;
                let call_instr = self.stack.get_function().bytecode[self.stack.instr_ptr - 1];
                self.stack.base_ptr -= call_instr.args_start() as usize;
                self.write_stack(call_instr.dst(), value);
                Ok(ControlFlow::Continue)
            }
            Op::ArrayInit => {
                // let length = self.read_operand(instr.src1()).as_int();
                // let array = self.heap.alloc_array(length as usize);
                // self.write_stack(instr.dst(), Value::array(array));
                // self.ip += 1;
                // Ok(ControlFlow::Continue)
                todo!()
            }
            Op::ListGet => {
                let Value::List(array) = self.read_operand(instr.src1()) else {
                    panic!()
                };
                let Value::Int(index) = self.read_operand(instr.src2()) else {
                    panic!()
                };
                self.write_stack(instr.dst(), array.get_element(index as usize));
                self.stack.instr_ptr += 1;
                Ok(ControlFlow::Continue)
            }
            Op::ArraySet => {
                let Value::List(list) = self.read_stack(instr.dst()) else {
                    panic!()
                };
                let Value::Int(index) = self.read_operand(instr.src2()) else {
                    panic!()
                };
                let value = self.read_operand(instr.src1());
                list.set_element(index as usize, value);
                self.stack.instr_ptr += 1;
                Ok(ControlFlow::Continue)
            }
        }
    }
}
