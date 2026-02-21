use std::{collections::HashMap, rc::Rc};

use crate::{
    ast::{self, Block, Else, Expr, Fun, Ident, If, InfixOp, Lit, Stmt, VarDef},
    scope::Scope,
    typed_ast::{
        LocalTypedVar, TypedBlock, TypedElse, TypedExpr, TypedFun, TypedIf, TypedInfixOp, TypedStmt, TypedVar, VarId,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Float,
    Bool,
    Unit,
    Void,
    Array(Box<Type>),
    Fun(FunSig),
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
    scope: Scope<Ident, Rc<LocalTypedVar>>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunSig {
    pub params: Vec<Type>,
    pub returns: Box<Type>,
}

impl FunSig {
    pub fn new(params: Vec<Type>, returns: Type) -> FunSig {
        FunSig {
            params,
            returns: Box::new(returns),
        }
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
    if !block.diverges && inferer.returns != Type::Unit {
        return Err(TypeError("missing return"));
    }
    inferer.scope.exit_block();
    Ok(TypedFun {
        name: fun.name.clone(),
        params,
        returns: inferer.returns,
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
            Expr::Array(elements) => {
                let mut typed_elements = vec![];
                let mut element_ty = None;
                for element in elements {
                    let (typed_element, ty) = self.infer_expr(element)?;
                    element_ty = match &mut element_ty {
                        Some(array_ty) if *array_ty != ty => Err(TypeError("array elements must have the same type"))?,
                        _ => Some(ty),
                    };
                    typed_elements.push(typed_element);
                }
                let typed_expr = TypedExpr::Array(typed_elements);
                (typed_expr, Type::Array(Box::new(element_ty.unwrap_or(Type::Void))))
            }
            Expr::Call(fun, args) => {
                let (typed_fun, fun_ty) = self.infer_expr(fun)?;
                let Type::Fun(sig) = fun_ty else {
                    return Err(TypeError("trying to call non function type"));
                };
                let mut typed_args = vec![];
                let mut arg_tys = vec![];
                for arg in args {
                    let (typed_arg, arg_ty) = self.infer_expr(arg)?;
                    typed_args.push(typed_arg);
                    arg_tys.push(arg_ty);
                }
                if arg_tys != sig.params {
                    return Err(TypeError("incorrect function arguments"));
                }
                let expr = TypedExpr::Call {
                    fun: Box::new(typed_fun),
                    args: typed_args,
                };
                (expr, sig.returns.as_ref().clone())
            }
            Expr::Paren(node) => self.infer_expr(node)?,
            Expr::Var(var) => {
                let (typed_var, ty) = if let Some(var) = self.scope.get(var.ident()) {
                    (TypedVar::Local(var.clone()), var.ty.clone())
                } else if let Some(sig) = self.env.get(var.ident()) {
                    (TypedVar::Env(var.ident().clone()), Type::Fun(sig.clone()))
                } else {
                    return Err(TypeError("undefined varaible"));
                };
                (TypedExpr::Var(typed_var), ty)
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
            Expr::Index(array, index) => {
                let (typed_array, array_ty) = self.infer_expr(array)?;
                let Type::Array(element_ty) = array_ty else {
                    return Err(TypeError("cannot index non array type"));
                };
                let (typed_index, index_ty) = self.infer_expr(index)?;
                if index_ty != Type::Int {
                    return Err(TypeError("index type must be an Int"));
                }
                let typed_expr = TypedExpr::Index(Box::new(typed_array), Box::new(typed_index));
                (typed_expr, *element_ty)
            }
        };

        Ok((typed_expr, ty))
    }

    fn insert_var(&mut self, var: &VarDef, ty: Type) -> Rc<LocalTypedVar> {
        let id = VarId(self.next_var_id);
        self.next_var_id += 1;
        let var = Rc::new(LocalTypedVar {
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
            Stmt::Assign { dst, op, src } => {
                let (dst, dst_ty) = self.infer_expr(dst)?;
                let (src, src_ty) = self.infer_expr(src)?;
                if dst_ty != src_ty {
                    return Err(TypeError("assignment type mismatch"));
                }
                let op = if let Some(op) = *op {
                    let ty = dst_ty
                        .as_numeric_type()
                        .ok_or(TypeError("assign arith type mismatch"))?;
                    src_ty
                        .as_numeric_type()
                        .ok_or(TypeError("assign arith type mismatch"))?;
                    Some((op, ty))
                } else {
                    None
                };
                (TypedStmt::Assign { dst, op, src }, false)
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
