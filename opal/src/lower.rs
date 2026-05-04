use std::collections::HashMap;

use crate::{
    ast::{ArithOp, BitwiseOp, CompOp, EqualityOp, Ident, Lit, LogicalOp},
    bytecode::BytecodeBuffer,
    heap::{Heap, function::Function, handle::Handle},
    instr::{ConstOperand, Operand, StackOperand},
    ty::NumericType,
    typed_ast::{
        TypedAssignOp, TypedBlock, TypedElse, TypedExpr, TypedFun, TypedIf, TypedInfixOp, TypedPrefixOp, TypedStmt,
        TypedVar, VarId,
    },
    value::Value,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Label(u32);

pub struct Lowerer<'h> {
    pub bytecode: BytecodeBuffer<Label>,
    pub consts: Vec<Value<'h>>,
    pub next_label: u32,
    pub stack_top: u8,
    pub stack_frames: Vec<u8>,
    pub vars: HashMap<VarId, StackOperand>,
    pub fun_ptrs: Vec<(Ident, u8)>,
    pub loop_stack: Vec<LoopLabels>,
}

#[derive(Debug, Clone, Copy)]
pub struct LoopLabels {
    break_: Label,
    continue_: Label,
}

pub fn lower_fun<'h>(fun: &TypedFun, heap: &Heap) -> (Handle<Function>, Vec<(Ident, u8)>) {
    let mut lowerer = Lowerer {
        bytecode: BytecodeBuffer::new(),
        consts: Vec::new(),
        next_label: 0,
        stack_top: 0,
        stack_frames: Vec::new(),
        vars: HashMap::new(),
        fun_ptrs: Vec::new(),
        loop_stack: Vec::new(),
    };
    for param in &fun.params {
        let reg = lowerer.alloc_stack_slot();
        lowerer.vars.insert(param.id, reg);
    }
    lowerer.lower_block(&fun.block);
    if !fun.block.diverges {
        let unit = lowerer.add_const(Value::Unit);
        lowerer.bytecode.instr().ret(Operand::Const(unit));
    }
    let bytecode = lowerer.bytecode.finish();
    let fun = heap.alloc_function(&bytecode, &lowerer.consts, 0).to_handle();
    (fun, lowerer.fun_ptrs)
}

impl<'h> Lowerer<'h> {
    fn add_const(&mut self, value: Value<'h>) -> ConstOperand {
        let cst = ConstOperand(self.consts.len() as u8);
        self.consts.push(value);
        cst
    }
    fn alloc_stack_slot(&mut self) -> StackOperand {
        let reg = StackOperand(self.stack_top);
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
    fn lower_expr_lit_val(&mut self, lit: &Lit) -> Operand {
        let cst = match *lit {
            Lit::Int(value) => self.add_const(Value::Int(value)),
            Lit::Float(value) => self.add_const(Value::Float(value)),
            Lit::Bool(value) => self.add_const(Value::Bool(value)),
            Lit::Unit => self.add_const(Value::Unit),
        };
        Operand::Const(cst)
    }
    fn lower_expr_var_val(&mut self, var: &TypedVar) -> Operand {
        match var {
            TypedVar::Local(var) => Operand::Stack(*self.vars.get(&var.id).unwrap()),
            TypedVar::Env(name) => {
                let fun = self.add_const(Value::UnpatchedFun);
                self.fun_ptrs.push((name.clone(), fun.0));
                Operand::Const(fun)
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
        branch_if: bool,
    ) {
        let src1 = self.lower_expr_val(left);
        let src2 = self.lower_expr_val(right);
        match (op, ty, branch_if) {
            (CompOp::Less, NumericType::Int, true) | (CompOp::GreaterEqual, NumericType::Int, false) => {
                self.bytecode.instr().iblt(src1, src2, label)
            }
            (CompOp::LessEqual, NumericType::Int, true) | (CompOp::Greater, NumericType::Int, false) => {
                self.bytecode.instr().iblte(src1, src2, label)
            }
            (CompOp::Greater, NumericType::Int, true) | (CompOp::LessEqual, NumericType::Int, false) => {
                self.bytecode.instr().ibgt(src1, src2, label)
            }
            (CompOp::GreaterEqual, NumericType::Int, true) | (CompOp::Less, NumericType::Int, false) => {
                self.bytecode.instr().ibgte(src1, src2, label)
            }
            (CompOp::Less, NumericType::Float, true) | (CompOp::GreaterEqual, NumericType::Float, false) => {
                self.bytecode.instr().fblt(src1, src2, label)
            }
            (CompOp::LessEqual, NumericType::Float, true) | (CompOp::Greater, NumericType::Float, false) => {
                self.bytecode.instr().fblte(src1, src2, label)
            }
            (CompOp::Greater, NumericType::Float, true) | (CompOp::LessEqual, NumericType::Float, false) => {
                self.bytecode.instr().fbgt(src1, src2, label)
            }
            (CompOp::GreaterEqual, NumericType::Float, true) | (CompOp::Less, NumericType::Float, false) => {
                self.bytecode.instr().fbgte(src1, src2, label)
            }
        }
    }
    fn lower_expr_equality_branch(
        &mut self,
        left: &TypedExpr,
        right: &TypedExpr,
        op: EqualityOp,
        label: Label,
        branch_if: bool,
    ) {
        let src1 = self.lower_expr_val(left);
        let src2 = self.lower_expr_val(right);
        match (op, branch_if) {
            (EqualityOp::Equal, true) | (EqualityOp::NotEqual, false) => self.bytecode.instr().beq(src1, src2, label),
            (EqualityOp::Equal, false) | (EqualityOp::NotEqual, true) => self.bytecode.instr().bneq(src1, src2, label),
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
            } => self.lower_expr_comp_branch(left, right, *op, label, *ty, branch_if),
            TypedExpr::Infix {
                left,
                right,
                op: TypedInfixOp::Equality(op),
            } => self.lower_expr_equality_branch(left, right, *op, label, branch_if),
            TypedExpr::Infix {
                left,
                right,
                op: TypedInfixOp::Logical(op),
            } => {
                self.lower_expr_logical_branch(left, right, *op, label, branch_if);
            }
            _ => {
                let val = self.lower_expr_val(expr);
                let cst = self.add_const(Value::Bool(branch_if));
                self.bytecode.instr().beq(val, Operand::Const(cst), label);
            }
        }
    }
    fn lower_infix_arith(&mut self, op: ArithOp, ty: NumericType, dst: StackOperand, src1: Operand, src2: Operand) {
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
    fn lower_infix_comp(&mut self, op: CompOp, ty: NumericType, dst: StackOperand, src1: Operand, src2: Operand) {
        match (op, ty) {
            (CompOp::Less, NumericType::Int) => self.bytecode.instr().islt(dst, src1, src2),
            (CompOp::LessEqual, NumericType::Int) => self.bytecode.instr().islte(dst, src1, src2),
            (CompOp::Greater, NumericType::Int) => self.bytecode.instr().isgt(dst, src1, src2),
            (CompOp::GreaterEqual, NumericType::Int) => self.bytecode.instr().isgte(dst, src1, src2),

            (CompOp::Less, NumericType::Float) => self.bytecode.instr().fslt(dst, src1, src2),
            (CompOp::LessEqual, NumericType::Float) => self.bytecode.instr().fslte(dst, src1, src2),
            (CompOp::Greater, NumericType::Float) => self.bytecode.instr().fsgt(dst, src1, src2),
            (CompOp::GreaterEqual, NumericType::Float) => self.bytecode.instr().fsgte(dst, src1, src2),
        }
    }
    fn lower_infix_equality(&mut self, op: EqualityOp, dst: StackOperand, src1: Operand, src2: Operand) {
        match op {
            EqualityOp::Equal => self.bytecode.instr().seq(dst, src1, src2),
            EqualityOp::NotEqual => self.bytecode.instr().sneq(dst, src1, src2),
        }
    }
    fn lower_infix_logical(&mut self, op: LogicalOp, dst: StackOperand, left: &TypedExpr, right: &TypedExpr) {
        let if_true = self.new_label();
        let if_false = self.new_label();

        let false_val = self.add_const(Value::Bool(false));
        let true_val = self.add_const(Value::Bool(true));

        self.lower_expr_logical_branch(left, right, op, if_true, true);
        self.bytecode.instr().mov(dst, Operand::Const(false_val));
        self.bytecode.instr().jmp(if_false);
        self.bytecode.label(if_true);
        self.bytecode.instr().mov(dst, Operand::Const(true_val));
        self.bytecode.label(if_false);
    }
    fn lower_infix_bitwise(&mut self, op: BitwiseOp, dst: StackOperand, src1: Operand, src2: Operand) {
        match op {
            BitwiseOp::And => self.bytecode.instr().and(dst, src1, src2),
            BitwiseOp::Or => self.bytecode.instr().or(dst, src1, src2),
            BitwiseOp::XOr => self.bytecode.instr().xor(dst, src1, src2),
            BitwiseOp::ShiftLeft => self.bytecode.instr().shl(dst, src1, src2),
            BitwiseOp::ShiftRight => self.bytecode.instr().shr(dst, src1, src2),
        }
    }
    fn lower_expr_array_elements_dst(&mut self, elements: &[TypedExpr], dst: StackOperand) {
        self.enter_stack_frame();
        let start = self.stack_top;
        for element in elements {
            let dst = self.alloc_stack_slot();
            self.lower_expr_dst(element, dst);
        }
        self.exit_stack_frame();
        self.bytecode.instr().list_elements(dst, start, elements.len() as u8);
    }
    fn lower_expr_array_default_length_dst(&mut self, default: &TypedExpr, length: &TypedExpr, dst: StackOperand) {
        let length = self.lower_expr_val(length);
        let default = self.lower_expr_val(default);
        self.bytecode.instr().list_default_length(dst, default, length);
    }
    fn lower_expr_index_dst(&mut self, array: &TypedExpr, index: &TypedExpr, dst: StackOperand) {
        let array = self.lower_expr_val(array);
        let index = self.lower_expr_val(index);
        self.bytecode.instr().list_get(dst, array, index);
    }
    fn lower_expr_infix_dst(&mut self, left: &TypedExpr, op: TypedInfixOp, right: &TypedExpr, dst: StackOperand) {
        match op {
            TypedInfixOp::Arith(op, ty) => {
                let src1 = self.lower_expr_val(left);
                let src2 = self.lower_expr_val(right);
                self.lower_infix_arith(op, ty, dst, src1, src2)
            }
            TypedInfixOp::Comp(op, ty) => {
                let src1 = self.lower_expr_val(left);
                let src2 = self.lower_expr_val(right);
                self.lower_infix_comp(op, ty, dst, src1, src2)
            }
            TypedInfixOp::Equality(op) => {
                let src1 = self.lower_expr_val(left);
                let src2 = self.lower_expr_val(right);
                self.lower_infix_equality(op, dst, src1, src2);
            }
            TypedInfixOp::Logical(op) => self.lower_infix_logical(op, dst, left, right),
            TypedInfixOp::Bitwise(op) => {
                let src1 = self.lower_expr_val(left);
                let src2 = self.lower_expr_val(right);
                self.lower_infix_bitwise(op, dst, src1, src2);
            }
        }
    }
    fn lower_expr_prefix_dst(&mut self, op: TypedPrefixOp, expr: &TypedExpr, dst: StackOperand) {
        match op {
            TypedPrefixOp::Negative(NumericType::Int) => {
                let val = self.lower_expr_val(expr);
                let zero = self.add_const(Value::Int(0));
                self.bytecode.instr().isub(dst, Operand::Const(zero), val);
            }
            TypedPrefixOp::Negative(NumericType::Float) => {
                let val = self.lower_expr_val(expr);
                let zero = self.add_const(Value::Float(0.0));
                self.bytecode.instr().fsub(dst, Operand::Const(zero), val);
            }
            TypedPrefixOp::Positive(_) => self.lower_expr_dst(expr, dst),
            TypedPrefixOp::BitwiseNot => {
                let val = self.lower_expr_val(expr);
                let all_ones = self.add_const(Value::Int(!0));
                self.bytecode.instr().xor(dst, Operand::Const(all_ones), val);
            }
            TypedPrefixOp::LogicalNot => {
                let val = self.lower_expr_val(expr);
                let false_ = self.add_const(Value::Bool(false));
                self.bytecode.instr().seq(dst, val, Operand::Const(false_));
            }
        }
    }
    fn lower_expr_call_dst(&mut self, fun: &TypedExpr, args: &[TypedExpr], dst: StackOperand) {
        let fun = self.lower_expr_val(fun);
        let arg_start = self.stack_top;
        self.enter_stack_frame();
        for arg in args {
            let arg_reg = self.alloc_stack_slot();
            self.lower_expr_dst(arg, arg_reg);
        }
        self.bytecode.instr().call(dst, fun, arg_start);
        self.exit_stack_frame();
    }
    fn dst_to_val(&mut self, f: impl Fn(&mut Lowerer, StackOperand)) -> Operand {
        let dst = self.alloc_stack_slot();
        f(self, dst);
        Operand::Stack(dst)
    }
    fn val_to_dst(&mut self, f: impl Fn(&mut Lowerer) -> Operand, dst: StackOperand) {
        let src = f(self);
        self.bytecode.instr().mov(dst, src);
    }
    fn lower_expr_val(&mut self, expr: &TypedExpr) -> Operand {
        match expr {
            TypedExpr::Lit(lit) => self.lower_expr_lit_val(lit),
            TypedExpr::Var(var) => self.lower_expr_var_val(var),
            TypedExpr::Call { fun, args } => self.dst_to_val(|self_, dst| self_.lower_expr_call_dst(fun, args, dst)),
            TypedExpr::ArrayElements(elements) => {
                self.dst_to_val(|self_, dst| self_.lower_expr_array_elements_dst(elements, dst))
            }
            TypedExpr::ArrayDefaultLength(default, length) => {
                self.dst_to_val(|self_, dst| self_.lower_expr_array_default_length_dst(default, length, dst))
            }
            TypedExpr::Index(array, index) => {
                self.dst_to_val(|self_, dst| self_.lower_expr_index_dst(array, index, dst))
            }
            TypedExpr::Prefix(op, expr) => self.dst_to_val(|self_, dst| self_.lower_expr_prefix_dst(*op, expr, dst)),
            TypedExpr::Infix { left, right, op } => {
                self.dst_to_val(|self_, dst| self_.lower_expr_infix_dst(left, *op, right, dst))
            }
        }
    }
    fn lower_expr_dst(&mut self, expr: &TypedExpr, dst: StackOperand) {
        self.enter_stack_frame();
        match expr {
            TypedExpr::ArrayElements(elements) => self.lower_expr_array_elements_dst(elements, dst),
            TypedExpr::ArrayDefaultLength(default, length) => {
                self.lower_expr_array_default_length_dst(default, length, dst)
            }
            TypedExpr::Index(array, index) => self.lower_expr_index_dst(array, index, dst),
            TypedExpr::Infix { left, right, op } => self.lower_expr_infix_dst(left, *op, right, dst),
            TypedExpr::Prefix(op, expr) => self.lower_expr_prefix_dst(*op, expr, dst),
            TypedExpr::Call { fun, args } => self.lower_expr_call_dst(fun, args, dst),
            TypedExpr::Lit(lit) => self.val_to_dst(|self_| self_.lower_expr_lit_val(lit), dst),
            TypedExpr::Var(var) => self.val_to_dst(|self_| self_.lower_expr_var_val(var), dst),
        }
        self.exit_stack_frame();
    }
    fn lower_assign(&mut self, dst: &TypedExpr, op: Option<TypedAssignOp>, src: &TypedExpr) {
        match dst {
            TypedExpr::Var(var) => {
                let TypedVar::Local(var) = var else { panic!() };
                let dst = *self.vars.get(&var.id).unwrap();
                match op {
                    Some(op) => {
                        let src = self.lower_expr_val(src);
                        match op {
                            TypedAssignOp::Arith(op, ty) => {
                                self.lower_infix_arith(op, ty, dst, Operand::Stack(dst), src)
                            }
                            TypedAssignOp::Bitwise(op) => self.lower_infix_bitwise(op, dst, Operand::Stack(dst), src),
                        }
                    }
                    None => self.lower_expr_dst(src, dst),
                }
            }
            TypedExpr::Index(array, index) => {
                self.enter_stack_frame();
                let array = match self.lower_expr_val(array) {
                    Operand::Stack(reg) => reg,
                    Operand::Const(src) => {
                        let dst = self.alloc_stack_slot();
                        self.bytecode.instr().mov(dst, Operand::Const(src));
                        dst
                    }
                };
                let index = self.lower_expr_val(index);
                let value = self.lower_expr_val(src);

                let value = op.map_or(value, |op| {
                    let reg = self.alloc_stack_slot();
                    self.bytecode.instr().list_get(reg, Operand::Stack(array), index);
                    match op {
                        TypedAssignOp::Arith(op, ty) => self.lower_infix_arith(op, ty, reg, Operand::Stack(reg), value),
                        TypedAssignOp::Bitwise(op) => self.lower_infix_bitwise(op, reg, Operand::Stack(reg), value),
                    };
                    Operand::Stack(reg)
                });

                self.bytecode.instr().list_set(array, value, index);

                self.exit_stack_frame();
            }
            _ => panic!(),
        }
    }
    fn lower_stmt(&mut self, stmt: &TypedStmt) {
        match stmt {
            TypedStmt::Let { var, expr } => {
                let var_reg = self.alloc_stack_slot();
                self.vars.insert(var.id, var_reg);
                self.lower_expr_dst(expr, var_reg);
            }
            TypedStmt::Assign { dst, op, src } => {
                self.lower_assign(dst, *op, src);
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
                self.loop_stack.push(LoopLabels {
                    break_: loop_exit,
                    continue_: loop_start,
                });
                self.lower_block(block);
                self.loop_stack.pop();
                self.bytecode.instr().jmp(loop_start);
                self.bytecode.label(loop_exit);
            }
            TypedStmt::Break => {
                let labels = self.loop_stack[self.loop_stack.len() - 1];
                self.bytecode.instr().jmp(labels.break_);
            }
            TypedStmt::Continue => {
                let labels = self.loop_stack[self.loop_stack.len() - 1];
                self.bytecode.instr().jmp(labels.continue_);
            }
        }
    }
    pub fn lower_else(&mut self, else_: &TypedElse) {
        match else_ {
            TypedElse::If(if_) => self.lower_if(if_),
            TypedElse::Block(block) => self.lower_block(block),
            TypedElse::Nothing => (),
        }
    }
    pub fn lower_if(&mut self, if_: &TypedIf) {
        if if_.if_block.diverges {
            let else_label = self.new_label();
            self.lower_expr_branch(&if_.cond, else_label, false);
            self.lower_block(&if_.if_block);
            self.bytecode.label(else_label);
            self.lower_else(&if_.else_);
        } else if else_diverges(&if_.else_) {
            let if_label = self.new_label();
            self.lower_expr_branch(&if_.cond, if_label, true);
            self.lower_else(&if_.else_);
            self.bytecode.label(if_label);
            self.lower_block(&if_.if_block);
        } else {
            let if_label = self.new_label();
            let exit_label = self.new_label();

            self.lower_expr_branch(&if_.cond, if_label, true);
            self.lower_else(&if_.else_);
            self.bytecode.instr().jmp(exit_label);

            self.bytecode.label(if_label);
            self.lower_block(&if_.if_block);
            self.bytecode.label(exit_label);
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

pub fn else_diverges(else_: &TypedElse) -> bool {
    match else_ {
        TypedElse::If(if_) => if_.if_block.diverges && else_diverges(&if_.else_),
        TypedElse::Block(block) => block.diverges,
        TypedElse::Nothing => false,
    }
}
