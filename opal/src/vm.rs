use std::ops::Neg;

use strum::EnumIs;

use crate::{
    heap::ObjectPtr,
    instr::{Instr, Op, Reg, Val},
    lower::CompiledFun,
    runtime::NativeFunSig,
    value::Value,
};

pub struct Call {
    fun: ObjectPtr,
    ip: usize,
}

pub struct VM<'f> {
    pub call_stack: Vec<Call>,
    pub value_stack: Vec<Value>,

    pub fun: &'f CompiledFun<'f>,
    pub value_stack_base: usize,
    pub ip: usize,
    pub heap: &'f ObjectHeap,
}

#[derive(Debug, Clone, Copy)]
pub enum Fun {
    Native(NativeFunSig),
    Compiled(ObjectPtr),
}

#[derive(Debug)]
pub struct RuntimeError;

#[derive(EnumIs)]
pub enum ControlFlow {
    Break,
    Continue,
}

impl<'f> VM<'f> {
    fn read_value(&self, val: Val) -> Value {
        match val {
            Val::Reg(reg) => self.read_reg(reg),
            Val::Cst(cst) => self.fun.consts[cst.0 as usize].get(),
        }
    }
    fn read_reg(&self, reg: Reg) -> Value {
        self.value_stack[self.value_stack_base + reg.0 as usize]
    }
    fn write_reg(&mut self, reg: Reg, val: Value) {
        self.value_stack[self.value_stack_base + reg.0 as usize] = val;
    }

    fn execute_arith_instr(
        &mut self,
        instr: Instr,
        op: impl Fn(Value, Value) -> Value,
    ) -> Result<ControlFlow, RuntimeError> {
        let src1 = self.read_value(instr.src1());
        let src2 = self.read_value(instr.src2());
        self.write_reg(instr.dst(), op(src1, src2));
        self.ip += 1;
        Ok(ControlFlow::Continue)
    }

    fn execute_branch_instr(
        &mut self,
        instr: Instr,
        cmp: impl Fn(Value, Value) -> bool,
    ) -> Result<ControlFlow, RuntimeError> {
        let src1 = self.read_value(instr.src1());
        let src2 = self.read_value(instr.src2());
        if cmp(src1, src2) {
            if instr.branch_offset().is_positive() {
                self.ip += instr.branch_offset() as usize
            } else {
                self.ip -= instr.branch_offset().neg() as usize;
            }
        } else {
            self.ip += 1;
        }
        Ok(ControlFlow::Continue)
    }

    fn execute_set_instr(
        &mut self,
        instr: Instr,
        cmp: impl Fn(Value, Value) -> bool,
    ) -> Result<ControlFlow, RuntimeError> {
        let src1 = self.read_value(instr.src1());
        let src2 = self.read_value(instr.src2());
        self.write_reg(instr.dst(), Value::from_bool(cmp(src1, src2)));
        self.ip += 1;
        Ok(ControlFlow::Continue)
    }

    fn ret(&mut self, value: Value) -> Result<ControlFlow, RuntimeError> {
        let Some(prev_call) = self.call_stack.pop() else {
            return Ok(ControlFlow::Break);
        };
        self.fun = prev_call.fun;
        self.ip = prev_call.ip + 1;
        let call_instr = self.fun.bytecode[prev_call.ip];
        self.value_stack_base -= call_instr.args_start() as usize;
        self.write_reg(call_instr.dst(), value);
        Ok(ControlFlow::Continue)
    }

    pub fn execute_next_instr(&mut self) -> Result<ControlFlow, RuntimeError> {
        let instr = self.fun.bytecode[self.ip];

        fn int_op<'f>(op: impl Fn(i64, i64) -> i64) -> impl Fn(Value, Value) -> Value {
            move |a, b| Value::from_int(op(a.as_int(), b.as_int()))
        }

        fn float_op<'f>(op: impl Fn(f64, f64) -> f64) -> impl Fn(Value, Value) -> Value {
            move |a, b| Value::from_float(op(a.as_float(), b.as_float()))
        }

        fn int_cmp<'f>(cmp: impl Fn(i64, i64) -> bool) -> impl Fn(Value, Value) -> bool {
            move |a, b| cmp(a.as_int(), b.as_int())
        }

        fn float_cmp<'f>(cmp: impl Fn(f64, f64) -> bool) -> impl Fn(Value, Value) -> bool {
            move |a, b| cmp(a.as_float(), b.as_float())
        }

        match instr.op() {
            Op::Mov => {
                let src1 = self.read_value(instr.src1());
                self.write_reg(instr.dst(), src1);
                self.ip += 1;
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
                    self.ip += instr.jump_offset() as usize;
                } else {
                    self.ip -= instr.jump_offset().neg() as usize;
                }
                Ok(ControlFlow::Continue)
            }
            Op::Call => {
                let fun = unsafe { self.read_value(instr.src1()).as_fun() };
                match fun {
                    Fun::Native(fun) => {
                        let args_start = instr.args_start() as usize;
                        let args = &self.value_stack[self.value_stack_base + args_start..];
                        self.write_reg(instr.dst(), fun(args)?);
                        self.ip += 1;
                    }
                    Fun::Compiled(fun) => {
                        self.call_stack.push(Call {
                            fun: self.fun,
                            ip: self.ip,
                        });
                        self.value_stack_base += instr.args_start() as usize;
                        self.fun = fun;
                        self.ip = 0;
                    }
                }
                Ok(ControlFlow::Continue)
            }
            Op::Ret => {
                let src = self.read_value(instr.src1());
                self.ret(src)
            }
            Op::ArrayInit => {
                let length = self.read_value(instr.src1());
                let array_object = self.heap.alloc_array(length.as_int() as u64);
                self.write_reg(instr.dst(), Value::from_object(array_object.heap_object()));
                self.ip += 1;
                Ok(ControlFlow::Continue)
            }
            Op::ArrayGet => {
                let array = self.read_value(instr.src1());
                let array_object = unsafe { array.as_object() }.as_array().unwrap();
                let index = self.read_value(instr.src2());
                self.write_reg(instr.dst(), array_object.get(index.as_int() as u64));
                self.ip += 1;
                Ok(ControlFlow::Continue)
            }
            Op::ArraySet => {
                let array = self.read_reg(instr.dst());
                let array_object = unsafe { array.as_object() }.as_array().unwrap();
                let value = self.read_value(instr.src1());
                let index = self.read_value(instr.src2());
                array_object.set(index.as_int() as u64, value);
                self.ip += 1;
                Ok(ControlFlow::Continue)
            }
        }
    }
}
