use std::{collections::HashMap, rc::Rc};

use crate::{
    ast::{self, Block, Else, Expr, Fun, Ident, If, InfixOp, Lit, Stmt, VarDef},
    scope::Scope,
    typed_ast::{TypedBlock, TypedElse, TypedExpr, TypedFun, TypedIf, TypedInfixOp, TypedStmt, TypedVar, VarId},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Int,
    Float,
    Bool,
    Unit,
    Void,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericType {
    Int,
    Float,
}

impl From<NumericType> for Type {
    fn from(value: NumericType) -> Self {
        match value {
            NumericType::Int => Type::Int,
            NumericType::Float => Type::Float,
        }
    }
}

impl Type {
    fn as_numeric_type(&self) -> Option<NumericType> {
        match self {
            Type::Int => Some(NumericType::Int),
            Type::Float => Some(NumericType::Float),
            _ => None,
        }
    }
}

pub struct Inferer<'e> {
    next_var_id: u32,
    scope: Scope<Ident, Rc<TypedVar>>,
    env: &'e HashMap<Ident, FunSig>,
    returns: Type,
}

fn infer_lit(lit: &Lit) -> Type {
    match lit {
        Lit::Int(_) => Type::Int,
        Lit::Float(_) => Type::Float,
        Lit::Bool(_) => Type::Bool,
        Lit::Unit => Type::Unit,
    }
}

pub fn resolve_type(ty: &ast::Type) -> Result<Type, TypeError> {
    Ok(match ty.0.0.as_str() {
        "Int" => Type::Int,
        "Float" => Type::Float,
        "Bool" => Type::Bool,
        "Unit" => Type::Unit,
        "Void" => Type::Void,
        _ => return Err(TypeError("invalid type name")),
    })
}

#[derive(Debug, Clone)]
pub struct FunSig {
    pub params: Vec<Type>,
    pub returns: Type,
}

impl FunSig {
    pub fn new(params: Vec<Type>, returns: Type) -> FunSig {
        FunSig { params, returns }
    }
}

#[derive(Debug, Clone)]
pub struct TypeError(&'static str);

pub fn infer_fun(fun: &Fun, env: &HashMap<Ident, FunSig>) -> Result<TypedFun, TypeError> {
    let returns = fun
        .returns
        .as_ref()
        .map(resolve_type)
        .transpose()?
        .unwrap_or(Type::Unit);

    let mut inferer = Inferer {
        next_var_id: 0,
        scope: Scope::new(),
        env,
        returns,
    };
    inferer.scope.enter_block();
    let mut params = vec![];
    for (var, ty) in &fun.params {
        let ty = resolve_type(ty)?;
        params.push(inferer.insert_var(var, ty));
    }
    let block = inferer.infer_block(&fun.block)?;
    if !block.diverges && returns != Type::Unit {
        return Err(TypeError("missing return"));
    }
    inferer.scope.exit_block();
    Ok(TypedFun {
        name: fun.name.clone(),
        params,
        returns,
        block,
    })
}

impl<'e> Inferer<'e> {
    fn infer_expr(&mut self, expr: &Expr) -> Result<(TypedExpr, Type), TypeError> {
        let (typed_expr, ty) = match expr {
            Expr::Lit(lit) => {
                let typed_expr = TypedExpr::Lit(*lit);
                let ty = infer_lit(lit);
                (typed_expr, ty)
            }
            Expr::Call(name, args) => {
                let fun_sig = self.env.get(name).ok_or(TypeError("undefined function"))?;
                let mut typed_args = vec![];
                let mut arg_tys = vec![];
                for arg in args {
                    let (typed_arg, arg_ty) = self.infer_expr(arg)?;
                    typed_args.push(typed_arg);
                    arg_tys.push(arg_ty);
                }
                if arg_tys != fun_sig.params {
                    return Err(TypeError("incorrect function arguments"));
                }
                let expr = TypedExpr::Call {
                    name: name.clone(),
                    args: typed_args,
                };
                (expr, fun_sig.returns)
            }
            Expr::Paren(node) => self.infer_expr(node)?,
            Expr::Var(var) => {
                let var = self.scope.get(var.ident()).unwrap();
                let typed_expr = TypedExpr::Var(var.clone());
                (typed_expr, var.ty)
            }
            Expr::Infix { left, op, right } => {
                let (left_expr, left_ty) = self.infer_expr(left)?;
                let (right_expr, right_ty) = self.infer_expr(right)?;

                let (op, ty) = match *op {
                    InfixOp::Arith(op) => {
                        let left_ty = left_ty
                            .as_numeric_type()
                            .ok_or(TypeError("arithmetic operand left operand is not number"))?;
                        let right_ty = right_ty
                            .as_numeric_type()
                            .ok_or(TypeError("arithmetic operand right operand is not number"))?;
                        if left_ty != right_ty {
                            return Err(TypeError("arithmetic operands type mismatch"));
                        }
                        (TypedInfixOp::Arith(op, left_ty), left_ty.into())
                    }
                    InfixOp::Comp(op) => {
                        let left_ty = left_ty
                            .as_numeric_type()
                            .ok_or(TypeError("comparison operand left operand is not number"))?;
                        let right_ty = right_ty
                            .as_numeric_type()
                            .ok_or(TypeError("comparison operand right operand is not number"))?;
                        if left_ty != right_ty {
                            return Err(TypeError("comparison operands type mismatch"));
                        }
                        (TypedInfixOp::Comp(op, left_ty), Type::Bool)
                    }
                    InfixOp::Logical(op) => {
                        if left_ty != Type::Bool || right_ty != Type::Bool {
                            return Err(TypeError("logical operands incorrect type"));
                        }
                        (TypedInfixOp::Logical(op), Type::Bool)
                    }
                    InfixOp::Equality(op) => {
                        if left_ty != right_ty {
                            return Err(TypeError("equality operands type mismatch"));
                        }
                        (TypedInfixOp::Equality(op), Type::Bool)
                    }
                };

                let typed_expr = TypedExpr::Infix {
                    left: Box::new(left_expr),
                    op,
                    right: Box::new(right_expr),
                };

                (typed_expr, ty)
            }
        };

        Ok((typed_expr, ty))
    }

    fn insert_var(&mut self, var: &VarDef, ty: Type) -> Rc<TypedVar> {
        let id = VarId(self.next_var_id);
        self.next_var_id += 1;
        let var = Rc::new(TypedVar {
            mutable: var.mutable,
            ident: var.ident.clone(),
            ty,
            id,
        });
        self.scope.insert(var.ident.clone(), var.clone());
        var
    }

    fn infer_stmt(&mut self, stmt: &Stmt) -> Result<(TypedStmt, bool), TypeError> {
        Ok(match stmt {
            Stmt::Let { var, ty, expr } => {
                let (expr, expr_ty) = self.infer_expr(expr)?;
                if let Some(ty) = ty {
                    if expr_ty != resolve_type(ty)? {
                        return Err(TypeError("wrong type annotation!"));
                    }
                }
                let var = self.insert_var(var, expr_ty);
                (TypedStmt::Let { var, expr }, false)
            }
            Stmt::Assign { var, expr } => {
                let var = self.scope.get(var.ident()).unwrap().clone();
                let (expr, ty) = self.infer_expr(expr)?;
                if ty != var.ty {
                    return Err(TypeError("assignment type mismatch"));
                }
                (TypedStmt::Assign { var, expr }, false)
            }
            Stmt::AssignArith { var, op, expr } => {
                let var = self.scope.get(var.ident()).unwrap().clone();
                let (expr, expr_ty) = self.infer_expr(expr)?;
                let var_ty = var
                    .ty
                    .as_numeric_type()
                    .ok_or(TypeError("assign arith type mismatch"))?;
                let expr_ty = expr_ty
                    .as_numeric_type()
                    .ok_or(TypeError("assign arith type mismatch"))?;
                if expr_ty != var_ty {
                    return Err(TypeError("assign arith type mismatch"));
                }
                let stmt = TypedStmt::AssignArith {
                    var,
                    ty: var_ty,
                    op: *op,
                    expr,
                };
                (stmt, false)
            }
            Stmt::Expr(expr) => {
                let (expr, ty) = self.infer_expr(expr)?;
                let diverges = match ty {
                    Type::Unit => false,
                    Type::Void => true,
                    _ => return Err(TypeError("invalid statement expression")),
                };
                (TypedStmt::Expr(expr), diverges)
            }
            Stmt::Return(Some(expr)) => {
                let (expr, ty) = self.infer_expr(expr)?;
                if ty != self.returns {
                    return Err(TypeError("incorrect return type"));
                }
                (TypedStmt::Return(expr), true)
            }
            Stmt::Return(None) => {
                if Type::Unit != self.returns {
                    return Err(TypeError("incorrect return type"));
                }
                (TypedStmt::Return(TypedExpr::Lit(Lit::Unit)), true)
            }
            Stmt::If(if_) => {
                let (if_, diverges) = self.infer_if(if_)?;
                (TypedStmt::If(if_), diverges)
            }
            Stmt::While { cond, block } => {
                let (cond, ty) = self.infer_expr(cond)?;
                if ty != Type::Bool {
                    return Err(TypeError("while condition must be bool"));
                }
                let block = self.infer_block(block)?;
                (TypedStmt::While { cond, block }, false)
            }
        })
    }

    fn infer_if(&mut self, if_: &If) -> Result<(TypedIf, bool), TypeError> {
        let (cond, ty) = self.infer_expr(&if_.cond)?;
        if ty != Type::Bool {
            return Err(TypeError("if condition must be bool"));
        }
        let if_block = self.infer_block(&if_.if_block)?;
        let (else_, else_diverges) = match &if_.else_ {
            Else::If(if_) => {
                let (if_, diverges) = self.infer_if(if_)?;
                (TypedElse::If(Box::new(if_)), diverges)
            }
            Else::Block(block) => {
                let block = self.infer_block(block)?;
                let diverges = block.diverges;
                (TypedElse::Block(block), diverges)
            }
            Else::Nothing => (TypedElse::Nothing, false),
        };
        let diverges = if_block.diverges && else_diverges;
        Ok((TypedIf { cond, if_block, else_ }, diverges))
    }

    fn infer_block(&mut self, block: &Block) -> Result<TypedBlock, TypeError> {
        let mut stmts = vec![];
        self.scope.enter_block();
        let mut diverges = false;
        for stmt in &block.stmts {
            let (stmt, stmt_diverges) = self.infer_stmt(stmt)?;
            stmts.push(stmt);
            if stmt_diverges {
                diverges = true;
                break;
            }
        }
        self.scope.exit_block();
        Ok(TypedBlock { stmts, diverges })
    }
}
