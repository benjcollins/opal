use std::{collections::HashMap, rc::Rc};

use crate::{
    ast::{self, Block, Else, Expr, Fun, Ident, If, InfixOp, Lit, Stmt, VarDef},
    scope::Scope,
    typed_ast::{TypedBlock, TypedElse, TypedExpr, TypedFun, TypedIf, TypedStmt, TypedVar, VarId},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Int,
    Float,
    Bool,
    Unit,
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

pub fn resolve_type(ty: &ast::Type) -> Result<Type, ()> {
    Ok(match ty.0.0.as_str() {
        "Int" => Type::Int,
        "Float" => Type::Float,
        "Bool" => Type::Bool,
        "Unit" => Type::Unit,
        _ => return Err(()),
    })
}

pub struct FunSig {
    pub params: Vec<Type>,
    pub returns: Type,
}

impl FunSig {
    pub fn new(params: Vec<Type>, returns: Type) -> FunSig {
        FunSig { params, returns }
    }
}

pub fn infer_fun(fun: &Fun, env: &HashMap<Ident, FunSig>) -> Result<TypedFun, ()> {
    let returns = fun
        .returns
        .as_ref()
        .map(|returns| resolve_type(&returns))
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
    inferer.scope.exit_block();
    Ok(TypedFun {
        name: fun.name.clone(),
        params,
        returns,
        block,
    })
}

impl<'e> Inferer<'e> {
    fn infer_expr(&mut self, expr: &Expr) -> Result<(TypedExpr, Type), ()> {
        let (typed_expr, ty) = match expr {
            Expr::Lit(lit) => {
                let typed_expr = TypedExpr::Lit(*lit);
                let ty = infer_lit(lit);
                (typed_expr, ty)
            }
            Expr::Call(name, args) => {
                let fun_sig = self.env.get(name).ok_or(())?;
                let mut typed_args = vec![];
                let mut arg_tys = vec![];
                for arg in args {
                    let (typed_arg, arg_ty) = self.infer_expr(arg)?;
                    typed_args.push(typed_arg);
                    arg_tys.push(arg_ty);
                }
                if arg_tys != fun_sig.params {
                    return Err(());
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

                let left_ty = left_ty.as_numeric_type().ok_or(())?;
                let right_ty = right_ty.as_numeric_type().ok_or(())?;

                if left_ty != right_ty {
                    return Err(());
                }

                let typed_expr = TypedExpr::Infix {
                    left: Box::new(left_expr),
                    op: *op,
                    ty: left_ty,
                    right: Box::new(right_expr),
                };

                let ty = match op {
                    InfixOp::Arith(_) => left_ty.into(),
                    InfixOp::Comp(_) => Type::Bool,
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

    fn infer_stmt(&mut self, stmt: &Stmt) -> Result<TypedStmt, ()> {
        Ok(match stmt {
            Stmt::Let { var, expr } => {
                let (expr, ty) = self.infer_expr(expr)?;
                let var = self.insert_var(var, ty);
                TypedStmt::Let { var, expr }
            }
            Stmt::Assign { var, expr } => {
                let var = self.scope.get(var.ident()).unwrap().clone();
                let (expr, ty) = self.infer_expr(expr)?;
                if ty != var.ty {
                    return Err(());
                }
                TypedStmt::Assign { var, expr }
            }
            Stmt::AssignArith { var, op, expr } => {
                let var = self.scope.get(var.ident()).unwrap().clone();
                let (expr, expr_ty) = self.infer_expr(expr)?;
                let var_ty = var.ty.as_numeric_type().ok_or(())?;
                let expr_ty = expr_ty.as_numeric_type().ok_or(())?;
                if expr_ty != var_ty {
                    return Err(());
                }
                TypedStmt::AssignArith {
                    var,
                    expr,
                    ty: expr_ty,
                    op: *op,
                }
            }
            Stmt::Expr(expr) => {
                let (expr, ty) = self.infer_expr(expr)?;
                if ty != Type::Unit {
                    return Err(());
                }
                TypedStmt::Expr(expr)
            }
            Stmt::Return(Some(expr)) => {
                let (expr, ty) = self.infer_expr(expr)?;
                if ty != self.returns {
                    return Err(());
                }
                TypedStmt::Return(expr)
            }
            Stmt::Return(None) => {
                if Type::Unit != self.returns {
                    return Err(());
                }
                TypedStmt::Return(TypedExpr::Lit(Lit::Unit))
            }
            Stmt::If(if_) => TypedStmt::If(self.infer_if(if_)?),
            Stmt::While { cond, block } => {
                let (cond, ty) = self.infer_expr(cond)?;
                if ty != Type::Bool {
                    return Err(());
                }
                let block = self.infer_block(block)?;
                TypedStmt::While { cond, block }
            }
        })
    }

    fn infer_if(&mut self, if_: &If) -> Result<TypedIf, ()> {
        let (cond, ty) = self.infer_expr(&if_.cond)?;
        if ty != Type::Bool {
            return Err(());
        }
        let if_block = self.infer_block(&if_.if_block)?;
        let else_ = match &if_.else_ {
            Else::If(if_) => TypedElse::If(Box::new(self.infer_if(if_)?)),
            Else::Block(block) => TypedElse::Block(self.infer_block(block)?),
            Else::Nothing => TypedElse::Nothing,
        };
        Ok(TypedIf { cond, if_block, else_ })
    }

    fn infer_block(&mut self, block: &Block) -> Result<TypedBlock, ()> {
        let mut stmts = vec![];
        self.scope.enter_block();
        for stmt in &block.stmts {
            stmts.push(self.infer_stmt(stmt)?);
        }
        self.scope.exit_block();
        Ok(TypedBlock { stmts })
    }
}
