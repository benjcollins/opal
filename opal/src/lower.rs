use std::{cell::Cell, collections::HashMap};

use crate::{
    ast::{ArithOp, CompOp, Ident, Lit, LogicalOp},
    bytecode::BytecodeBuffer,
    infer::NumericType,
    instr::{Cst, Instr, Reg, Val},
    typed_ast::{TypedBlock, TypedElse, TypedExpr, TypedFun, TypedIf, TypedInfixOp, TypedStmt, VarId},
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

#[derive(Debug)]
pub struct CompiledFun<'f> {
    pub consts: Vec<Cell<Value<'f>>>,
    pub bytecode: Vec<Instr>,
}

pub fn lower_fun<'f>(fun: &TypedFun) -> (CompiledFun<'f>, Vec<(Ident, u8)>) {
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
        CompiledFun {
            consts: lowerer.consts.into_iter().map(Cell::new).collect(),
            bytecode: lowerer.bytecode.finish(),
        },
        lowerer.fun_ptrs,
    )
}

impl CompOp {
    fn invert(&self) -> CompOp {
        match self {
            CompOp::Equal => CompOp::NotEqual,
            CompOp::NotEqual => CompOp::Equal,
            CompOp::Greater => CompOp::LessEqual,
            CompOp::Less => CompOp::GreaterEqual,
            CompOp::LessEqual => CompOp::Greater,
            CompOp::GreaterEqual => CompOp::Less,
        }
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
    fn fresh_const(&mut self) -> Cst {
        let cst = Cst(self.consts.len() as u8);
        self.consts.push(Value::from_unit(()));
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
    fn new_label(&mut self) -> Label {
        let label = Label(self.next_label);
        self.next_label += 1;
        label
    }
    fn lower_expr_val(&mut self, expr: &TypedExpr) -> Val {
        match expr {
            TypedExpr::Lit(lit) => {
                let cst = match *lit {
                    Lit::Int(value) => self.get_const(Value::from_int(value)),
                    Lit::Float(value) => self.get_const(Value::from_float(value)),
                    Lit::Bool(value) => self.get_const(Value::from_bool(value)),
                    Lit::Unit => self.get_const(Value::from_unit(())),
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
    fn lower_expr_comp_branch(
        &mut self,
        left: &TypedExpr,
        right: &TypedExpr,
        op: CompOp,
        label: Label,
        ty: NumericType,
    ) {
        let src1 = self.lower_expr_val(left);
        let src2 = self.lower_expr_val(right);
        match (op, ty) {
            (CompOp::Equal, _) => self.bytecode.instr().beq(src1, src2, label),
            (CompOp::NotEqual, _) => self.bytecode.instr().bne(src1, src2, label),

            (CompOp::Less, NumericType::Int) => self.bytecode.instr().iblt(src1, src2, label),
            (CompOp::LessEqual, NumericType::Int) => self.bytecode.instr().ible(src1, src2, label),
            (CompOp::Greater, NumericType::Int) => self.bytecode.instr().ibgt(src1, src2, label),
            (CompOp::GreaterEqual, NumericType::Int) => self.bytecode.instr().ibge(src1, src2, label),

            (CompOp::Less, NumericType::Float) => self.bytecode.instr().fblt(src1, src2, label),
            (CompOp::LessEqual, NumericType::Float) => self.bytecode.instr().fble(src1, src2, label),
            (CompOp::Greater, NumericType::Float) => self.bytecode.instr().fbgt(src1, src2, label),
            (CompOp::GreaterEqual, NumericType::Float) => self.bytecode.instr().fbge(src1, src2, label),
        }
    }
    fn lower_expr_logical_branch(
        &mut self,
        left: &TypedExpr,
        right: &TypedExpr,
        op: LogicalOp,
        target: Label,
        branch_if: bool,
    ) {
        match (op, branch_if) {
            (LogicalOp::And, branch_if @ true) | (LogicalOp::Or, branch_if @ false) => {
                let skip = self.new_label();
                self.lower_expr_branch(left, skip, !branch_if);
                self.lower_expr_branch(right, target, branch_if);
                self.bytecode.label(skip);
            }
            (LogicalOp::And, branch_if @ false) | (LogicalOp::Or, branch_if @ true) => {
                self.lower_expr_branch(left, target, branch_if);
                self.lower_expr_branch(right, target, branch_if);
            }
        }
    }
    fn lower_expr_branch(&mut self, expr: &TypedExpr, label: Label, branch_if: bool) {
        match expr {
            TypedExpr::Infix {
                left,
                right,
                op: TypedInfixOp::Comp(op, ty),
            } => {
                let op = if branch_if { *op } else { op.invert() };
                self.lower_expr_comp_branch(left, right, op, label, *ty)
            }
            TypedExpr::Infix {
                left,
                right,
                op: TypedInfixOp::Logical(op),
            } => {
                self.lower_expr_logical_branch(left, right, *op, label, branch_if);
            }
            _ => {
                let val = self.lower_expr_val(expr);
                let cst = self.get_const(Value::from_bool(branch_if));
                self.bytecode.instr().beq(val, Val::Cst(cst), label);
            }
        }
    }
    fn lower_infix_arith(&mut self, op: ArithOp, ty: NumericType, dst: Reg, src1: Val, src2: Val) {
        match (op, ty) {
            (ArithOp::Add, NumericType::Int) => self.bytecode.instr().iadd(dst, src1, src2),
            (ArithOp::Subtract, NumericType::Int) => self.bytecode.instr().isub(dst, src1, src2),
            (ArithOp::Multiply, NumericType::Int) => self.bytecode.instr().imul(dst, src1, src2),
            (ArithOp::Divide, NumericType::Int) => self.bytecode.instr().idiv(dst, src1, src2),
            (ArithOp::Modulus, NumericType::Int) => self.bytecode.instr().imod(dst, src1, src2),

            (ArithOp::Add, NumericType::Float) => self.bytecode.instr().fadd(dst, src1, src2),
            (ArithOp::Subtract, NumericType::Float) => self.bytecode.instr().fsub(dst, src1, src2),
            (ArithOp::Multiply, NumericType::Float) => self.bytecode.instr().fmul(dst, src1, src2),
            (ArithOp::Divide, NumericType::Float) => self.bytecode.instr().fdiv(dst, src1, src2),
            (ArithOp::Modulus, NumericType::Float) => self.bytecode.instr().fmod(dst, src1, src2),
        }
    }
    fn lower_infix_comp(&mut self, op: CompOp, ty: NumericType, dst: Reg, src1: Val, src2: Val) {
        match (op, ty) {
            (CompOp::Equal, _) => self.bytecode.instr().seq(dst, src1, src2),
            (CompOp::NotEqual, _) => self.bytecode.instr().sne(dst, src1, src2),

            (CompOp::Less, NumericType::Int) => self.bytecode.instr().islt(dst, src1, src2),
            (CompOp::LessEqual, NumericType::Int) => self.bytecode.instr().isle(dst, src1, src2),
            (CompOp::Greater, NumericType::Int) => self.bytecode.instr().isgt(dst, src1, src2),
            (CompOp::GreaterEqual, NumericType::Int) => self.bytecode.instr().isge(dst, src1, src2),

            (CompOp::Less, NumericType::Float) => self.bytecode.instr().fslt(dst, src1, src2),
            (CompOp::LessEqual, NumericType::Float) => self.bytecode.instr().fsle(dst, src1, src2),
            (CompOp::Greater, NumericType::Float) => self.bytecode.instr().fsgt(dst, src1, src2),
            (CompOp::GreaterEqual, NumericType::Float) => self.bytecode.instr().fsge(dst, src1, src2),
        }
    }
    fn lower_expr_dst(&mut self, expr: &TypedExpr, dst: Reg) {
        match expr {
            TypedExpr::Infix { left, right, op } => {
                let src1 = self.lower_expr_val(left);
                let src2 = self.lower_expr_val(right);
                match *op {
                    TypedInfixOp::Arith(op, ty) => self.lower_infix_arith(op, ty, dst, src1, src2),
                    TypedInfixOp::Comp(op, ty) => self.lower_infix_comp(op, ty, dst, src1, src2),
                    TypedInfixOp::Logical(op) => {
                        let if_true = self.new_label();
                        let if_false = self.new_label();

                        let false_val = self.get_const(Value::from_bool(false));
                        let true_val = self.get_const(Value::from_bool(true));

                        self.lower_expr_logical_branch(left, right, op, if_true, true);
                        self.bytecode.instr().mov(dst, Val::Cst(false_val));
                        self.bytecode.instr().jmp(if_false);
                        self.bytecode.label(if_true);
                        self.bytecode.instr().mov(dst, Val::Cst(true_val));
                        self.bytecode.label(if_false);
                    }
                }
            }
            TypedExpr::Call { name, args } => {
                let fun = self.fresh_const();
                self.fun_ptrs.push((name.clone(), fun.0));
                let arg_start = self.stack_top;
                self.enter_stack_frame();
                for arg in args {
                    let arg_reg = self.alloc_reg();
                    self.lower_expr_dst(arg, arg_reg);
                }
                self.bytecode.instr().call(dst, Val::Cst(fun), arg_start);
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
            TypedStmt::AssignArith { var, ty, op, expr } => {
                let var_reg = *self.vars.get(&var.id).unwrap();
                let expr_val = self.lower_expr_val(expr);
                self.lower_infix_arith(*op, *ty, var_reg, Val::Reg(var_reg), expr_val);
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
            TypedStmt::If(if_) => self.lower_if(if_),
            TypedStmt::While { cond, block } => {
                let loop_start = self.new_label();
                let loop_exit = self.new_label();

                self.bytecode.label(loop_start);
                self.lower_expr_branch(cond, loop_exit, false);
                self.lower_block(block);
                self.bytecode.instr().jmp(loop_start);
                self.bytecode.label(loop_exit);
            }
        }
    }
    pub fn lower_if(&mut self, if_: &TypedIf) {
        let if_true = self.new_label();
        let if_false = self.new_label();

        self.lower_expr_branch(&if_.cond, if_true, true);
        match &if_.else_ {
            TypedElse::If(if_) => self.lower_if(if_),
            TypedElse::Block(block) => self.lower_block(block),
            TypedElse::Nothing => (),
        }
        self.bytecode.instr().jmp(if_false);

        self.bytecode.label(if_true);
        self.lower_block(&if_.if_block);
        self.bytecode.label(if_false);
    }
    pub fn lower_block(&mut self, block: &TypedBlock) {
        self.enter_stack_frame();
        for stmt in &block.stmts {
            self.lower_stmt(stmt);
        }
        self.exit_stack_frame();
    }
}
