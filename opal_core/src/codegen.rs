use std::collections::HashMap;

use crate::{
    ast::InfixOp,
    bytecode::{Bytecode, Fun},
    instr::{ImmSlot, Operand, Reg},
    ir,
    ty::{NumericType, TypeContext},
    value::Value,
};

pub struct Codegen<'a, 'b> {
    bytecode: Bytecode<'b>,
    local_map: HashMap<ir::LocalId, Reg>,
    global_imm_slots: Vec<(String, ImmSlot)>,
    next_reg: u8,
    type_context: &'a TypeContext,
    scopes: Vec<u8>,
}

impl<'a, 'b> Codegen<'a, 'b> {
    pub fn new(type_context: &'a TypeContext) -> Codegen<'a, 'b> {
        Codegen {
            bytecode: Bytecode::new(),
            local_map: HashMap::new(),
            global_imm_slots: Vec::new(),
            scopes: Vec::new(),
            next_reg: 0,
            type_context,
        }
    }

    pub fn finish(self) -> Fun {
        self.bytecode.finish()
    }

    pub fn gen_expr_operand(&mut self, expr: &ir::Expr) -> Operand<'b> {
        match expr {
            ir::Expr::Bool(value) => Value::from_bool(*value).into(),
            ir::Expr::Int(value) => Value::from_int(*value).into(),
            ir::Expr::Float(value) => Value::from_float(*value).into(),
            ir::Expr::String(_) => todo!(),
            ir::Expr::Unit => Value::UNIT.into(),
            ir::Expr::Local(id) => (*self.local_map.get(id).unwrap()).into(),
            ir::Expr::Global(id) => {
                let slot = self.bytecode.alloc_imm_slot();
                self.global_imm_slots.push((id.clone(), slot));
                slot.into()
            }
            _ => {
                let reg = self.alloc_reg();
                self.gen_expr_acc(expr);
                self.bytecode.instr().store(reg);
                reg.into()
            }
        }
    }

    pub fn gen_expr_acc(&mut self, expr: &ir::Expr) {
        self.enter_scope();
        match expr {
            ir::Expr::Call(fun, args) => {
                let fun = self.gen_expr_operand(fun);
                let args_base = self.next_reg;
                for arg in args {
                    let reg = self.alloc_reg();
                    self.gen_expr_acc(arg);
                    self.bytecode.instr().store(reg);
                }
                self.bytecode.instr().load(fun);
                self.bytecode.instr().call(args_base);
            }
            ir::Expr::Infix {
                left,
                op,
                ty,
                right,
            } => {
                let ty = self.type_context.get_numeric_type(ty).unwrap();
                let left_operand = self.gen_expr_operand(left);
                self.gen_expr_acc(right);
                let instr = self.bytecode.instr();
                match (op, ty) {
                    (InfixOp::Add, NumericType::Int) => instr.iadd(left_operand),
                    (InfixOp::Add, NumericType::Float) => instr.fadd(left_operand),
                    (InfixOp::Sub, NumericType::Int) => instr.isub(left_operand),
                    (InfixOp::Sub, NumericType::Float) => instr.fsub(left_operand),
                    (InfixOp::Mul, NumericType::Int) => instr.imul(left_operand),
                    (InfixOp::Mul, NumericType::Float) => instr.fmul(left_operand),
                    (InfixOp::Div, NumericType::Int) => instr.idiv(left_operand),
                    (InfixOp::Div, NumericType::Float) => instr.fdiv(left_operand),
                }
            }
            _ => {
                let operand = self.gen_expr_operand(expr);
                self.bytecode.instr().load(operand);
            }
        }
        self.exit_scope();
    }

    pub fn gen_stmt(&mut self, stmt: &ir::Stmt) {
        match stmt {
            ir::Stmt::VarDecl { local, value } => {
                let reg = self.alloc_reg();
                self.gen_expr_acc(value);
                self.bytecode.instr().store(reg);
                self.local_map.insert(*local, reg);
            }
            ir::Stmt::Expr(expr) => {
                self.gen_expr_acc(expr);
            }
            ir::Stmt::Return(expr) => {
                self.gen_expr_acc(expr);
                self.bytecode.instr().ret();
            }
        }
    }

    pub fn gen_block(&mut self, block: &ir::Block) {
        for stmt in &block.stmts {
            self.gen_stmt(stmt);
        }
    }

    fn enter_scope(&mut self) {
        self.scopes.push(self.next_reg);
    }

    fn exit_scope(&mut self) {
        self.next_reg = self.scopes.pop().unwrap();
    }

    fn alloc_reg(&mut self) -> Reg {
        let reg = Reg(self.next_reg);
        self.next_reg += 1;
        reg
    }
}
