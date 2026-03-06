use std::ops::Neg;

use strum::EnumIs;

use crate::{
    heap::{Bytecode, HeapLock, Object, Values},
    instr::{Instr, Op, Reg, Val},
    runtime::NativeFun,
    value::Value,
};

pub struct VM<'h, 'l> {
    pub call_stack: Object<'l, Values>,
    pub value_stack: Object<'l, Values>,

    pub bytecode: Object<'l, Bytecode>,
    pub consts: Object<'l, Values>,

    pub value_stack_frame: usize,
    pub call_stack_top: usize,

    pub ip: usize,
    pub heap: &'l HeapLock<'h>,
}

#[derive(Debug, Clone, Copy)]
pub enum Fun<'l> {
    Native(NativeFun),
    Compiled(Object<'l, Values>),
}

#[derive(Debug)]
pub struct RuntimeError;

#[derive(EnumIs)]
pub enum ControlFlow {
    Break,
    Continue,
}

impl<'h: 'l, 'l> VM<'h, 'l> {
    fn read_value(&self, val: Val) -> Value<'l> {
        match val {
            Val::Reg(reg) => self.read_reg(reg),
            Val::Cst(cst) => self.consts.get(cst.0 as usize),
        }
    }
    fn read_reg(&self, reg: Reg) -> Value<'l> {
        self.value_stack.get(self.value_stack_frame + reg.0 as usize)
    }
    fn write_reg(&mut self, reg: Reg, val: Value<'l>) {
        self.value_stack.set(self.value_stack_frame + reg.0 as usize, val);
    }

    fn execute_arith_instr(
        &mut self,
        instr: Instr,
        op: impl Fn(Value<'l>, Value<'l>) -> Value<'l>,
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
        cmp: impl Fn(Value<'l>, Value<'l>) -> bool,
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
        cmp: impl Fn(Value<'l>, Value<'l>) -> bool,
    ) -> Result<ControlFlow, RuntimeError> {
        let src1 = self.read_value(instr.src1());
        let src2 = self.read_value(instr.src2());
        self.write_reg(instr.dst(), Value::from_bool(cmp(src1, src2)));
        self.ip += 1;
        Ok(ControlFlow::Continue)
    }

    fn call_stack_push(&mut self) {
        self.call_stack
            .set(self.call_stack_top, Value::from_int(self.ip as i64));
        self.call_stack
            .set(self.call_stack_top + 1, Value::from_object(self.bytecode));
        self.call_stack
            .set(self.call_stack_top + 2, Value::from_object(self.consts));
        self.call_stack_top += 3;
    }

    fn call_stack_pop(&mut self) -> Option<(usize, Object<'l, Bytecode>, Object<'l, Values>)> {
        if self.call_stack_top == 0 {
            return None;
        }
        self.call_stack_top -= 3;
        let ip = self.call_stack.get(self.call_stack_top).as_int() as usize;
        let bytecode = unsafe { self.call_stack.get(self.call_stack_top + 1).as_object() };
        let consts = unsafe { self.call_stack.get(self.call_stack_top + 2).as_object() };
        Some((ip, bytecode, consts))
    }

    fn ret(&mut self, value: Value<'l>) -> Result<ControlFlow, RuntimeError> {
        let Some((prev_ip, prev_bytecode, prev_consts)) = self.call_stack_pop() else {
            return Ok(ControlFlow::Break);
        };
        self.ip = prev_ip + 1;
        self.bytecode = prev_bytecode;
        self.consts = prev_consts;
        let call_instr = self.bytecode.get(prev_ip);
        self.value_stack_frame -= call_instr.args_start() as usize;
        self.write_reg(call_instr.dst(), value);
        Ok(ControlFlow::Continue)
    }

    pub fn execute_next_instr(&mut self) -> Result<ControlFlow, RuntimeError> {
        let instr = self.bytecode.get(self.ip);

        fn int_op<'v>(op: impl Fn(i64, i64) -> i64) -> impl Fn(Value<'v>, Value<'v>) -> Value<'v> {
            move |a, b| Value::from_int(op(a.as_int(), b.as_int()))
        }

        fn float_op<'v>(op: impl Fn(f64, f64) -> f64) -> impl Fn(Value<'v>, Value<'v>) -> Value<'v> {
            move |a, b| Value::from_float(op(a.as_float(), b.as_float()))
        }

        fn int_cmp<'v>(cmp: impl Fn(i64, i64) -> bool) -> impl Fn(Value<'v>, Value<'v>) -> bool {
            move |a, b| cmp(a.as_int(), b.as_int())
        }

        fn float_cmp<'v>(cmp: impl Fn(f64, f64) -> bool) -> impl Fn(Value<'v>, Value<'v>) -> bool {
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
                let value = self.read_value(instr.src1());
                if value.is_object() {
                    let object: Object<'_, Values> = unsafe { value.as_object() };
                    self.call_stack_push();
                    self.value_stack_frame += instr.args_start() as usize;
                    self.bytecode = unsafe { object.get(0).as_object() };
                    self.consts = unsafe { object.get(1).as_object() };
                    self.ip = 0;
                } else {
                    let args_start = instr.args_start() as usize;
                    let fun = unsafe { value.as_native_fun() };
                    self.write_reg(instr.dst(), fun(self.value_stack, self.value_stack_frame + args_start)?);
                    self.ip += 1;
                }
                Ok(ControlFlow::Continue)
            }
            Op::Ret => {
                let src = self.read_value(instr.src1());
                self.ret(src)
            }
            Op::ArrayInit => {
                let length = self.read_value(instr.src1());
                let object: Object<'l, Values> = self.heap.alloc(length.as_int() as usize);
                self.write_reg(instr.dst(), Value::from_object(object));
                self.ip += 1;
                Ok(ControlFlow::Continue)
            }
            Op::ArrayGet => {
                let object: Object<'l, Values> = unsafe { self.read_value(instr.src1()).as_object() };
                let index = self.read_value(instr.src2());
                self.write_reg(instr.dst(), object.get(index.as_int() as usize));
                self.ip += 1;
                Ok(ControlFlow::Continue)
            }
            Op::ArraySet => {
                let object: Object<'l, Values> = unsafe { self.read_reg(instr.dst()).as_object() };
                let value = self.read_value(instr.src1());
                let index = self.read_value(instr.src2());
                object.set(index.as_int() as usize, value);
                self.ip += 1;
                Ok(ControlFlow::Continue)
            }
        }
    }
}
