use std::ops::Neg;

use strum::EnumIs;

use crate::{
    heap::{
        mutator::Mutator,
        object::{Array, Object},
        stack::Stack,
    },
    instr::{Instr, Op, Reg, Val},
    lower::CompiledFun,
    value::{Value, ValueTag},
};

pub struct VM<'m, 's, 'h> {
    pub call_stack: Vec<(usize, &'s CompiledFun<'s>)>,
    pub value_stack: Stack<'h, 's>,
    pub fun: &'s CompiledFun<'s>,
    pub value_stack_frame: usize,
    pub ip: usize,
    pub mutator: &'m Mutator<'h>,
}

#[derive(Debug)]
pub struct RuntimeError;

#[derive(EnumIs)]
pub enum ControlFlow {
    Break,
    Continue,
}

impl<'m, 's: 'm, 'h> VM<'m, 's, 'h> {
    fn read_value(&self, val: Val) -> Value<'m, 's> {
        match val {
            Val::Reg(reg) => self.read_reg(reg),
            Val::Cst(cst) => self.fun.consts[cst.0 as usize].get().into(),
        }
    }
    fn read_reg(&self, reg: Reg) -> Value<'m, 's> {
        self.value_stack
            .get(self.value_stack_frame + reg.0 as usize, self.mutator)
    }
    fn write_reg(&mut self, reg: Reg, val: Value<'m, 's>) {
        self.value_stack
            .set(self.value_stack_frame + reg.0 as usize, val, self.mutator);
    }

    fn execute_arith_instr(
        &mut self,
        instr: Instr,
        op: impl Fn(Value<'m, 's>, Value<'m, 's>) -> Value<'m, 's>,
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
        cmp: impl Fn(Value<'m, 's>, Value<'m, 's>) -> bool,
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
        cmp: impl Fn(Value<'m, 's>, Value<'m, 's>) -> bool,
    ) -> Result<ControlFlow, RuntimeError> {
        let src1 = self.read_value(instr.src1());
        let src2 = self.read_value(instr.src2());
        self.write_reg(instr.dst(), Value::bool(cmp(src1, src2)));
        self.ip += 1;
        Ok(ControlFlow::Continue)
    }

    pub fn execute_next_instr(&mut self) -> Result<ControlFlow, RuntimeError> {
        let instr = self.fun.bytecode[self.ip];

        fn int_op<'m, 's>(op: impl Fn(i64, i64) -> i64) -> impl Fn(Value, Value) -> Value<'m, 's> {
            move |a, b| Value::int(op(a.as_int(), b.as_int()))
        }

        fn float_op<'m, 's>(op: impl Fn(f64, f64) -> f64) -> impl Fn(Value, Value) -> Value<'m, 's> {
            move |a, b| Value::float(op(a.as_float(), b.as_float()))
        }

        fn int_cmp(cmp: impl Fn(i64, i64) -> bool) -> impl Fn(Value, Value) -> bool {
            move |a, b| cmp(a.as_int(), b.as_int())
        }

        fn float_cmp(cmp: impl Fn(f64, f64) -> bool) -> impl Fn(Value, Value) -> bool {
            move |a, b| cmp(a.as_float(), b.as_float())
        }

        // for i in 0..8 {
        //     let value = self.read_reg(Reg(i));
        //     print!("r{} = {}", i, value);
        //     if i != 7 { print!(", ") } else { println!() }
        // }
        // println!("{}", instr);

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
                let value: Value<'m, 's> = self.read_value(instr.src1());
                match value.tag() {
                    ValueTag::HostFun => {
                        let fun = value.as_host_fun();
                        let args_start = instr.args_start() as usize;
                        self.write_reg(
                            instr.dst(),
                            fun(&self.value_stack, self.mutator, self.value_stack_frame + args_start)?,
                        );
                        self.ip += 1;
                    }
                    ValueTag::Fun => {
                        let fun = value.as_fun();
                        self.call_stack.push((self.ip, self.fun));
                        self.value_stack_frame += instr.args_start() as usize;
                        self.fun = fun;
                        self.ip = 0;
                    }
                    _ => panic!(),
                }
                Ok(ControlFlow::Continue)
            }
            Op::Ret => {
                let value: Value<'m, 's> = self.read_value(instr.src1());
                let Some((prev_ip, prev_fun)) = self.call_stack.pop() else {
                    return Ok(ControlFlow::Break);
                };
                self.ip = prev_ip + 1;
                self.fun = prev_fun;
                let call_instr = self.fun.bytecode[prev_ip];
                self.value_stack_frame -= call_instr.args_start() as usize;
                self.write_reg(call_instr.dst(), value);
                Ok(ControlFlow::Continue)
            }
            Op::ArrayInit => {
                let length = self.read_value(instr.src1()).as_int();
                let array = self.mutator.alloc_array(length as usize);
                self.write_reg(instr.dst(), Value::array(array));
                self.ip += 1;
                Ok(ControlFlow::Continue)
            }
            Op::ArrayGet => {
                let v: Value<'m, 's> = self.read_value(instr.src1());
                let array: Object<'m, Array<'s>> = v.as_array();
                let index = self.read_value(instr.src2()).as_int();
                self.write_reg(instr.dst(), array.get(index as usize));
                self.ip += 1;
                Ok(ControlFlow::Continue)
            }
            Op::ArraySet => {
                let array = self.read_reg(instr.dst()).as_array();
                let value = self.read_value(instr.src1());
                let index = self.read_value(instr.src2()).as_int();
                array.set(index as usize, value);
                self.ip += 1;
                Ok(ControlFlow::Continue)
            }
        }
    }
}
