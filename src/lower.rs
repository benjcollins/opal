use std::collections::HashMap;

use crate::{
    ast::{InfixOp, Lit},
    bytecode::{BytecodeBuffer, Cst, Instr, Reg, Val},
    infer::NumericType,
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
}

pub struct Fun<'f> {
    pub consts: Vec<Value<'f>>,
    pub bytecode: Vec<Instr>,
}

pub fn lower_fun<'f>(fun: &TypedFun) -> Fun<'f> {
    let mut lowerer = Lowerer {
        bytecode: BytecodeBuffer::new(),
        consts: Vec::new(),
        consts_index: HashMap::new(),
        next_label: 0,
        stack_top: fun.params.len() as u8,
        stack_frames: Vec::new(),
        vars: HashMap::new(),
    };
    for param in &fun.params {
        let reg = lowerer.new_reg();
        lowerer.vars.insert(param.id, reg);
    }
    lowerer.lower_block(&fun.block);
    Fun {
        consts: lowerer.consts,
        bytecode: lowerer.bytecode.finish(),
    }
}

impl<'f> Lowerer<'f> {
    fn get_const(&mut self, value: Value<'f>) -> Cst {
        *self.consts_index.entry(value).or_insert_with(|| {
            let cst = Cst(self.consts.len() as u8);
            self.consts.push(value);
            cst
        })
    }
    fn new_reg(&mut self) -> Reg {
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
                };
                Val::Cst(cst)
            }
            TypedExpr::Var(var) => Val::Reg(*self.vars.get(&var.id).unwrap()),
            _ => {
                let dst = self.new_reg();
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
                let var_reg = self.new_reg();
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
