use std::{marker::PhantomData, ptr};

use crate::{
    bytecode::{Instr, Op, Val},
    lower::Fun,
};

pub struct VM<'f> {
    pub bytecode: &'f [Instr],
    pub ip: usize,
    pub regs: Vec<Value<'f>>,
    pub csts: Vec<Value<'f>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Value<'f>(u64, PhantomData<&'f ()>);

impl<'f> Value<'f> {
    pub fn null() -> Value<'f> {
        Value(0, PhantomData)
    }
    pub fn int(value: i64) -> Value<'f> {
        Value(value as u64, PhantomData)
    }
    pub fn float(value: f64) -> Value<'f> {
        Value(value.to_bits(), PhantomData)
    }
    pub fn fun_ptr(value: &'f Fun) -> Value<'f> {
        Value(ptr::from_ref(value) as u64, PhantomData)
    }
    fn as_float(self) -> f64 {
        f64::from_bits(self.0)
    }
    fn as_int(self) -> i64 {
        self.0 as i64
    }
    unsafe fn as_fun_ptr(self) -> &'f Fun<'f> {
        unsafe { (self.0 as *const Fun).as_ref().unwrap() }
    }
}

fn int_op<'f>(op: impl Fn(i64, i64) -> i64) -> impl Fn(Value<'f>, Value<'f>) -> Value<'f> {
    move |a, b| Value::int(op(a.as_int(), b.as_int()))
}

fn float_op<'f>(op: impl Fn(f64, f64) -> f64) -> impl Fn(Value<'f>, Value<'f>) -> Value<'f> {
    move |a, b| Value::float(op(a.as_float(), b.as_float()))
}

fn int_cmp<'f>(cmp: impl Fn(i64, i64) -> bool) -> impl Fn(Value<'f>, Value<'f>) -> bool {
    move |a, b| cmp(a.as_int(), b.as_int())
}

fn float_cmp<'f>(cmp: impl Fn(f64, f64) -> bool) -> impl Fn(Value<'f>, Value<'f>) -> bool {
    move |a, b| cmp(a.as_float(), b.as_float())
}

impl<'f> VM<'f> {
    fn read_value(&self, val: Val) -> Value<'f> {
        match val {
            Val::Reg(reg) => self.regs[reg.0 as usize],
            Val::Cst(cst) => self.csts[cst.0 as usize],
        }
    }

    fn execute_arith_instr(&mut self, instr: Instr, op: impl Fn(Value<'f>, Value<'f>) -> Value<'f>) {
        let src1 = self.read_value(instr.src1());
        let src2 = self.read_value(instr.src2());
        self.regs[instr.dst().0 as usize] = op(src1, src2);
        self.ip += 1;
    }

    fn execute_branch_instr(&mut self, instr: Instr, cmp: impl Fn(Value<'f>, Value<'f>) -> bool) {
        let src1 = self.read_value(instr.src1());
        let src2 = self.read_value(instr.src2());
        if cmp(src1, src2) {
            self.ip = ((self.ip as isize) + (instr.branch_offset() as isize)) as usize;
        } else {
            self.ip += 1;
        }
    }

    pub fn execute_next_instr(&mut self) {
        let instr = self.bytecode[self.ip];

        println!("{:?}", instr.op());

        match instr.op() {
            Op::MOV => {
                let src1 = self.read_value(instr.src1());
                self.regs[instr.dst().0 as usize] = src1;
                self.ip += 1;
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

            Op::JMP => {
                self.ip = ((self.ip as isize) + (instr.jump_offset() as isize)) as usize;
            }

            Op::CALL => todo!(),
            Op::RET => todo!(),
        }
    }
}
