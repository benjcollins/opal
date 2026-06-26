use std::collections::HashMap;

use crate::{
    ast::InfixOp,
    bytecode::Bytecode,
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
            global_imm_slots: vec![],
            next_reg: 0,
            type_context,
            scopes: vec![],
        }
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
                self.bytecode.store(reg);
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
                    self.bytecode.store(reg);
                }
                self.bytecode.load(fun);
                self.bytecode.call(args_base);
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
                match (op, ty) {
                    (InfixOp::Add, NumericType::Int) => self.bytecode.iadd(left_operand),
                    (InfixOp::Add, NumericType::Float) => self.bytecode.fadd(left_operand),
                    (InfixOp::Sub, NumericType::Int) => self.bytecode.isub(left_operand),
                    (InfixOp::Sub, NumericType::Float) => self.bytecode.fsub(left_operand),
                    (InfixOp::Mul, NumericType::Int) => self.bytecode.imul(left_operand),
                    (InfixOp::Mul, NumericType::Float) => self.bytecode.fmul(left_operand),
                    (InfixOp::Div, NumericType::Int) => self.bytecode.idiv(left_operand),
                    (InfixOp::Div, NumericType::Float) => self.bytecode.fdiv(left_operand),
                }
            }
            _ => {
                let operand = self.gen_expr_operand(expr);
                self.bytecode.load(operand);
            }
        }
        self.exit_scope();
    }

    pub fn gen_stmt(&mut self, stmt: &ir::Stmt) {
        match stmt {
            ir::Stmt::VarDecl { local, value } => {
                let reg = self.alloc_reg();
                self.gen_expr_acc(value);
                self.bytecode.store(reg);
                self.local_map.insert(*local, reg);
            }
            ir::Stmt::Expr(expr) => {
                self.gen_expr_acc(expr);
            }
            ir::Stmt::Return(expr) => {
                self.gen_expr_acc(expr);
                self.bytecode.ret();
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
