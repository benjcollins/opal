use std::collections::HashMap;

use crate::{
    ast::InfixOp,
    bytecode::Bytecode,
    instr::{ImmSlot, Operand, Reg},
    ir,
    ty::NumericType,
    value::Value,
};

pub struct Codegen {
    bytecode: Bytecode,
    local_map: HashMap<ir::LocalId, Reg>,
    global_imm_slots: Vec<(String, ImmSlot)>,
    next_reg: u8,
}

impl Codegen {
    pub fn gen_expr_operand(&mut self, expr: &ir::Expr) -> Operand {
        match expr {
            ir::Expr::Bool(value) => Operand::ImmValue(Value::bool(*value)),
            ir::Expr::Int(value) => Operand::ImmValue(Value::int(*value)),
            ir::Expr::Float(value) => Operand::ImmValue(Value::float(*value)),
            ir::Expr::String(_) => todo!(),
            ir::Expr::Unit => Operand::ImmValue(Value::UNIT),
            ir::Expr::Local(id) => Operand::Reg(*self.local_map.get(id).unwrap()),
            ir::Expr::Global(id) => {
                let slot = self.bytecode.alloc_imm_slot();
                self.global_imm_slots.push((id.clone(), slot));
                Operand::ImmSlot(slot)
            }
            _ => {
                let reg = self.alloc_reg();
                self.gen_expr_acc(expr);
                self.bytecode.st(reg);
                Operand::Reg(reg)
            }
        }
    }

    pub fn gen_expr_acc(&mut self, expr: &ir::Expr) {
        match expr {
            ir::Expr::Call(fun, args) => {
                let fun = self.gen_expr_operand(fun);
                let args_base = self.next_reg;
                for arg in args {
                    let reg = self.alloc_reg();
                    self.gen_expr_acc(arg);
                    self.bytecode.st(reg);
                }
                self.bytecode.ld(fun);
                self.bytecode.call(args_base);
            }
            ir::Expr::Infix { left, op, ty, right } => {
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
                    (_, NumericType::Meta(_)) => panic!(),
                }
            }
            _ => {
                let operand = self.gen_expr_operand(expr);
                self.bytecode.ld(operand);
            }
        }
    }

    pub fn gen_stmt(&mut self, stmt: &ir::Stmt) {
        match stmt {
            ir::Stmt::VarDecl { var, value } => todo!(),
            ir::Stmt::Expr(expr) => todo!(),
            ir::Stmt::Return(expr) => todo!(),
        }
    }

    fn alloc_reg(&mut self) -> Reg {
        todo!()
    }
}
