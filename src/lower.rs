use std::{cell::Cell, collections::HashMap};

use crate::{
    ast::{Ident, InfixOp, Lit},
    bytecode::BytecodeBuffer,
    infer::NumericType,
    instr::{Cst, Instr, Reg, Val},
    typed_ast::{TypedBlock, TypedExpr, TypedFun, TypedStmt, VarId},
    vm::Value,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Label(u32);

pub struct Lowerer<'f> {
    pub bytecode: BytecodeBuffer<Label>,
    pub consts: Vec<Value<'f>>,
    pub consts_index: HashMap<Value<'f>, Cst>,
    pub next_label: u32,
    pub stack_top: u8,
    pub stack_frames: Vec<u8>,
    pub vars: HashMap<VarId, Reg>,
    pub fun_ptrs: Vec<(Ident, u8)>,
}

pub struct Fun<'f> {
    pub consts: Vec<Cell<Value<'f>>>,
    pub bytecode: Vec<Instr>,
}

pub fn lower_fun<'f>(fun: &TypedFun) -> (Fun<'f>, Vec<(Ident, u8)>) {
    let mut lowerer = Lowerer {
        bytecode: BytecodeBuffer::new(),
        consts: Vec::new(),
        consts_index: HashMap::new(),
        next_label: 0,
        stack_top: 0,
        stack_frames: Vec::new(),
        vars: HashMap::new(),
        fun_ptrs: Vec::new(),
    };
    for param in &fun.params {
        let reg = lowerer.alloc_reg();
        lowerer.vars.insert(param.id, reg);
    }
    lowerer.lower_block(&fun.block);
    (
        Fun {
            consts: lowerer.consts.into_iter().map(Cell::new).collect(),
            bytecode: lowerer.bytecode.finish(),
        },
        lowerer.fun_ptrs,
    )
}

impl<'f> Lowerer<'f> {
    fn get_const(&mut self, value: Value<'f>) -> Cst {
        *self.consts_index.entry(value).or_insert_with(|| {
            let cst = Cst(self.consts.len() as u8);
            self.consts.push(value);
            cst
        })
    }
    fn fresh_const(&mut self) -> Cst {
        let cst = Cst(self.consts.len() as u8);
        self.consts.push(Value::unit());
        cst
    }
    fn alloc_reg(&mut self) -> Reg {
        let reg = Reg(self.stack_top);
        self.stack_top += 1;
        reg
    }
    fn enter_stack_frame(&mut self) {
        self.stack_frames.push(self.stack_top);
    }
    fn exit_stack_frame(&mut self) {
        self.stack_top = self.stack_frames.pop().unwrap();
    }
    fn lower_expr_val(&mut self, expr: &TypedExpr) -> Val {
        match expr {
            TypedExpr::Lit(lit) => {
                let cst = match lit {
                    &Lit::Int(value) => self.get_const(Value::int(value)),
                    &Lit::Float(value) => self.get_const(Value::float(value)),
                    &Lit::Bool(_) => todo!(),
                    &Lit::Unit => self.get_const(Value::unit()),
                };
                Val::Cst(cst)
            }
            TypedExpr::Var(var) => Val::Reg(*self.vars.get(&var.id).unwrap()),
            _ => {
                let dst = self.alloc_reg();
                self.lower_expr_dst(expr, dst);
                Val::Reg(dst)
            }
        }
    }
    fn lower_expr_dst(&mut self, expr: &TypedExpr, dst: Reg) {
        match expr {
            TypedExpr::Infix { left, right, op, ty } => {
                let src1 = self.lower_expr_val(left);
                let src2 = self.lower_expr_val(right);
                match (op, ty) {
                    (InfixOp::Add, NumericType::Int) => self.bytecode.instr().iadd(dst, src1, src2),
                    (InfixOp::Subtract, NumericType::Int) => self.bytecode.instr().isub(dst, src1, src2),
                    (InfixOp::Multiply, NumericType::Int) => self.bytecode.instr().imul(dst, src1, src2),
                    (InfixOp::Divide, NumericType::Int) => self.bytecode.instr().idiv(dst, src1, src2),
                    (InfixOp::Mod, NumericType::Int) => self.bytecode.instr().imod(dst, src1, src2),

                    (InfixOp::Add, NumericType::Float) => self.bytecode.instr().fadd(dst, src1, src2),
                    (InfixOp::Subtract, NumericType::Float) => self.bytecode.instr().fsub(dst, src1, src2),
                    (InfixOp::Multiply, NumericType::Float) => self.bytecode.instr().fmul(dst, src1, src2),
                    (InfixOp::Divide, NumericType::Float) => self.bytecode.instr().fdiv(dst, src1, src2),
                    (InfixOp::Mod, NumericType::Float) => self.bytecode.instr().fmod(dst, src1, src2),
                };
            }
            TypedExpr::Call { name, args, native } => {
                let fun = self.fresh_const();
                self.fun_ptrs.push((name.clone(), fun.0));
                let arg_start = self.stack_top;
                self.enter_stack_frame();
                for arg in args {
                    let arg_reg = self.alloc_reg();
                    self.lower_expr_dst(arg, arg_reg);
                }
                if *native {
                    self.bytecode.instr().calln(dst, Val::Cst(fun), arg_start);
                } else {
                    self.bytecode.instr().call(dst, Val::Cst(fun), arg_start);
                }
                self.exit_stack_frame();
            }
            _ => {
                self.enter_stack_frame();
                let src = self.lower_expr_val(expr);
                self.bytecode.instr().mov(dst, src);
                self.exit_stack_frame();
            }
        }
    }
    fn lower_stmt(&mut self, stmt: &TypedStmt) {
        match stmt {
            TypedStmt::Let { var, expr } => {
                let var_reg = self.alloc_reg();
                self.vars.insert(var.id, var_reg);
                self.lower_expr_dst(expr, var_reg);
            }
            TypedStmt::Assign { var, expr } => {
                let var_reg = self.vars.get(&var.id).unwrap();
                self.lower_expr_dst(expr, *var_reg);
            }
            TypedStmt::Expr(expr) => {
                self.enter_stack_frame();
                self.lower_expr_val(expr);
                self.exit_stack_frame();
            }
            TypedStmt::Return(expr) => {
                let val = self.lower_expr_val(expr);
                self.bytecode.instr().ret(val);
            }
        }
    }
    pub fn lower_block(&mut self, block: &TypedBlock) {
        self.enter_stack_frame();
        for stmt in &block.stmts {
            self.lower_stmt(stmt);
        }
        self.exit_stack_frame();
    }
}
