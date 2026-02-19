use std::{
    convert::Infallible,
    marker::PhantomData,
    mem::transmute,
    ops::{ControlFlow, Neg},
    ptr,
};

use crate::{
    infer::Type,
    instr::{Instr, Op, Reg, Val},
    lower::CompiledFun,
};

pub struct Call<'f> {
    fun: &'f CompiledFun<'f>,
    ip: usize,
}

pub struct VM<'f> {
    pub call_stack: Vec<Call<'f>>,
    pub value_stack: Vec<Value<'f>>,

    pub fun: &'f CompiledFun<'f>,
    pub value_stack_base: usize,
    pub ip: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Value<'f>(u64, PhantomData<&'f ()>);

pub trait ValueConv {
    const TYPE: Type;

    fn into_value<'h>(self) -> Value<'h>;
    fn from_value<'h>(value: Value<'h>) -> Self;
}

pub trait NativeFunResult {
    type Output: ValueConv;
    fn map<'h>(self) -> Result<Value<'h>, RuntimeError>;
}

impl<T: ValueConv> NativeFunResult for Result<T, RuntimeError> {
    type Output = T;
    fn map<'h>(self) -> Result<Value<'h>, RuntimeError> {
        self.map(|t| t.into_value())
    }
}

impl ValueConv for i64 {
    const TYPE: Type = Type::Int;
    fn into_value<'h>(self) -> Value<'h> {
        Value::from_int(self)
    }
    fn from_value<'h>(value: Value<'h>) -> Self {
        value.as_int()
    }
}

impl ValueConv for bool {
    const TYPE: Type = Type::Bool;
    fn into_value<'h>(self) -> Value<'h> {
        Value::from_bool(self)
    }
    fn from_value<'h>(value: Value<'h>) -> Self {
        value.as_bool()
    }
}

impl ValueConv for f64 {
    const TYPE: Type = Type::Float;
    fn into_value<'h>(self) -> Value<'h> {
        Value::from_float(self)
    }
    fn from_value<'h>(value: Value<'h>) -> Self {
        value.as_float()
    }
}

impl ValueConv for () {
    const TYPE: Type = Type::Unit;
    fn into_value<'h>(self) -> Value<'h> {
        Value::from_unit(())
    }
    fn from_value<'h>(_: Value<'h>) -> Self {}
}

impl ValueConv for Infallible {
    const TYPE: Type = Type::Void;

    fn into_value<'h>(self) -> Value<'h> {
        match self {}
    }

    fn from_value<'h>(_: Value<'h>) -> Self {
        unreachable!()
    }
}

impl<'f> Value<'f> {
    pub fn from_unit(_: ()) -> Value<'f> {
        Value(0, PhantomData)
    }
    pub fn from_int(value: i64) -> Value<'f> {
        Value(value as u64, PhantomData)
    }
    pub fn from_float(value: f64) -> Value<'f> {
        Value(value.to_bits(), PhantomData)
    }
    pub fn from_bool(value: bool) -> Value<'f> {
        Value(value as u64, PhantomData)
    }
    pub fn from_fun(fun: Fun) -> Value<'f> {
        match fun {
            Fun::Native(fun) => Value(fun as usize as u64 | 1 << 63, PhantomData),
            Fun::Compiled(fun) => Value(ptr::from_ref(fun) as u64, PhantomData),
        }
    }
    pub fn as_float(self) -> f64 {
        f64::from_bits(self.0)
    }
    pub fn as_int(self) -> i64 {
        self.0 as i64
    }
    pub fn as_bool(self) -> bool {
        self.0 != 0
    }
    pub unsafe fn as_fun(self) -> Fun<'f> {
        unsafe {
            if self.0 >> 63 == 1 {
                let ptr = self.0 & !(1 << 63);
                Fun::Native(transmute(ptr as *const ()))
            } else {
                Fun::Compiled((self.0 as *const CompiledFun).as_ref().unwrap())
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Fun<'f> {
    Native(fn(&[Value<'f>]) -> Result<Value<'f>, RuntimeError>),
    Compiled(&'f CompiledFun<'f>),
}

fn int_op<'f>(op: impl Fn(i64, i64) -> i64) -> impl Fn(Value<'f>, Value<'f>) -> Value<'f> {
    move |a, b| Value::from_int(op(a.as_int(), b.as_int()))
}

fn float_op<'f>(op: impl Fn(f64, f64) -> f64) -> impl Fn(Value<'f>, Value<'f>) -> Value<'f> {
    move |a, b| Value::from_float(op(a.as_float(), b.as_float()))
}

fn int_cmp<'f>(cmp: impl Fn(i64, i64) -> bool) -> impl Fn(Value<'f>, Value<'f>) -> bool {
    move |a, b| cmp(a.as_int(), b.as_int())
}

fn float_cmp<'f>(cmp: impl Fn(f64, f64) -> bool) -> impl Fn(Value<'f>, Value<'f>) -> bool {
    move |a, b| cmp(a.as_float(), b.as_float())
}

#[derive(Debug)]
pub struct RuntimeError;

impl<'f> VM<'f> {
    fn read_value(&self, val: Val) -> Value<'f> {
        match val {
            Val::Reg(reg) => self.value_stack[self.value_stack_base + reg.0 as usize],
            Val::Cst(cst) => self.fun.consts[cst.0 as usize].get(),
        }
    }

    fn write_reg(&mut self, reg: Reg, val: Value<'f>) {
        self.value_stack[self.value_stack_base + reg.0 as usize] = val;
    }

    fn execute_arith_instr(
        &mut self,
        instr: Instr,
        op: impl Fn(Value<'f>, Value<'f>) -> Value<'f>,
    ) -> Result<ControlFlow<(), ()>, RuntimeError> {
        let src1 = self.read_value(instr.src1());
        let src2 = self.read_value(instr.src2());
        self.write_reg(instr.dst(), op(src1, src2));
        self.ip += 1;
        Ok(ControlFlow::Continue(()))
    }

    fn execute_branch_instr(
        &mut self,
        instr: Instr,
        cmp: impl Fn(Value<'f>, Value<'f>) -> bool,
    ) -> Result<ControlFlow<(), ()>, RuntimeError> {
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
        Ok(ControlFlow::Continue(()))
    }

    fn execute_set_instr(
        &mut self,
        instr: Instr,
        cmp: impl Fn(Value<'f>, Value<'f>) -> bool,
    ) -> Result<ControlFlow<(), ()>, RuntimeError> {
        let src1 = self.read_value(instr.src1());
        let src2 = self.read_value(instr.src2());
        self.write_reg(instr.dst(), Value::from_bool(cmp(src1, src2)));
        self.ip += 1;
        Ok(ControlFlow::Continue(()))
    }

    fn ret(&mut self, value: Value<'f>) -> Result<ControlFlow<(), ()>, RuntimeError> {
        let Some(prev_call) = self.call_stack.pop() else {
            return Ok(ControlFlow::Break(()));
        };
        self.fun = prev_call.fun;
        self.ip = prev_call.ip + 1;
        let call_instr = self.fun.bytecode[prev_call.ip];
        self.value_stack_base -= call_instr.args_start() as usize;
        self.write_reg(call_instr.dst(), value);
        Ok(ControlFlow::Continue(()))
    }

    pub fn execute_next_instr(&mut self) -> Result<ControlFlow<(), ()>, RuntimeError> {
        let instr = self.fun.bytecode[self.ip];

        match instr.op() {
            Op::MOV => {
                let src1 = self.read_value(instr.src1());
                self.write_reg(instr.dst(), src1);
                self.ip += 1;
                Ok(ControlFlow::Continue(()))
            }

            Op::IADD => self.execute_arith_instr(instr, int_op(|a, b| a + b)),
            Op::ISUB => self.execute_arith_instr(instr, int_op(|a, b| a - b)),
            Op::IMUL => self.execute_arith_instr(instr, int_op(|a, b| a * b)),
            Op::IDIV => self.execute_arith_instr(instr, int_op(|a, b| a / b)),
            Op::IMOD => self.execute_arith_instr(instr, int_op(|a, b| a % b)),

            Op::FADD => self.execute_arith_instr(instr, float_op(|a, b| a + b)),
            Op::FSUB => self.execute_arith_instr(instr, float_op(|a, b| a - b)),
            Op::FMUL => self.execute_arith_instr(instr, float_op(|a, b| a * b)),
            Op::FDIV => self.execute_arith_instr(instr, float_op(|a, b| a / b)),
            Op::FMOD => self.execute_arith_instr(instr, float_op(|a, b| a % b)),

            Op::BEQ => self.execute_branch_instr(instr, |a, b| a == b),
            Op::BNE => self.execute_branch_instr(instr, |a, b| a != b),
            Op::IBLT => self.execute_branch_instr(instr, int_cmp(|a, b| a < b)),
            Op::IBLE => self.execute_branch_instr(instr, int_cmp(|a, b| a <= b)),
            Op::FBLT => self.execute_branch_instr(instr, float_cmp(|a, b| a < b)),
            Op::FBLE => self.execute_branch_instr(instr, float_cmp(|a, b| a <= b)),

            Op::SEQ => self.execute_set_instr(instr, |a, b| a == b),
            Op::SNE => self.execute_set_instr(instr, |a, b| a != b),
            Op::ISLT => self.execute_set_instr(instr, int_cmp(|a, b| a < b)),
            Op::ISLE => self.execute_set_instr(instr, int_cmp(|a, b| a <= b)),
            Op::FSLT => self.execute_set_instr(instr, float_cmp(|a, b| a < b)),
            Op::FSLE => self.execute_set_instr(instr, float_cmp(|a, b| a <= b)),

            Op::JMP => {
                if instr.jump_offset().is_positive() {
                    self.ip += instr.jump_offset() as usize;
                } else {
                    self.ip -= instr.jump_offset().neg() as usize;
                }
                Ok(ControlFlow::Continue(()))
            }
            Op::CALL => {
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
                Ok(ControlFlow::Continue(()))
            }
            Op::RET => {
                let src = self.read_value(instr.src1());
                self.ret(src)
            }
        }
    }
}
