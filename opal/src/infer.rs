use std::{
    collections::{HashMap, hash_map::Entry},
    rc::Rc,
};

use crate::{
    ast::{AssignOp, Block, Else, Expr, Fun, Ident, If, InfixOp, Lit, PrefixOp, Stmt, VarDef},
    scope::Scope,
    ty::{FunSig, NumericType, Type},
    typed_ast::{
        LocalTypedVar, TypedAssignOp, TypedBlock, TypedElse, TypedExpr, TypedFun, TypedIf, TypedInfixOp, TypedPrefixOp,
        TypedStmt, TypedVar, VarId,
    },
};

pub struct Inferer<'e> {
    next_var_id: u32,
    scope: Scope<Ident, Rc<LocalTypedVar>>,
    env: &'e HashMap<Ident, FunSig>,
    returns: Type,
}

fn infer_lit(lit: &Lit) -> Type {
    match lit {
        Lit::Int(_) => Type::Numeric(NumericType::Int),
        Lit::Float(_) => Type::Numeric(NumericType::Float),
        Lit::Bool(_) => Type::Bool,
        Lit::Unit => Type::Unit,
        Lit::Str(_) => Type::Str,
    }
}

#[derive(Debug, Clone)]
pub struct TypeError(pub &'static str);

pub fn infer_fun(fun: &Fun, env: &HashMap<Ident, FunSig>) -> Result<TypedFun, TypeError> {
    let returns = fun
        .returns
        .as_ref()
        .map(|ty| ty.try_into().map_err(|_| TypeError("could not convert type")))
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
        let ty = ty.try_into().map_err(|_| TypeError("could not convert type"))?;
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

fn instantiate(map: &mut HashMap<Ident, Type>, arg: &Type, param: &Type) -> Result<(), ()> {
    match (arg, param) {
        (Type::Bool, Type::Bool) | (Type::Unit, Type::Unit) | (Type::Void, Type::Void) | (Type::Str, Type::Str) => {
            Ok(())
        }
        (Type::Numeric(a), Type::Numeric(b)) if a == b => Ok(()),
        (Type::List(a), Type::List(b)) => instantiate(map, a, b),
        (Type::Fun(a), Type::Fun(b)) => {
            if !a.generics.is_empty() || !b.generics.is_empty() {
                return Err(());
            }
            if a.params.len() != b.params.len() {
                return Err(());
            }
            for (a, b) in a.params.iter().zip(&b.params) {
                instantiate(map, a, b)?;
            }
            instantiate(map, &a.returns, &b.returns)
        }
        (ty, Type::Generic(name)) => match map.entry(name.clone()) {
            Entry::Occupied(entry) if entry.get() == ty => Ok(()),
            Entry::Vacant(entry) => {
                entry.insert(ty.clone());
                Ok(())
            }
            _ => Err(()),
        },
        _ => Err(()),
    }
}

impl<'e> Inferer<'e> {
    fn infer_expr(&mut self, expr: &Expr) -> Result<(TypedExpr, Type), TypeError> {
        let (typed_expr, ty) = match expr {
            Expr::Lit(lit) => {
                let typed_expr = TypedExpr::Lit(lit.clone());
                let ty = infer_lit(lit);
                (typed_expr, ty)
            }
            Expr::ArrayElements(elements) => {
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
                let typed_expr = TypedExpr::ArrayElements(typed_elements);
                (typed_expr, Type::List(Box::new(element_ty.unwrap_or(Type::Void))))
            }
            Expr::ArrayDefaultLength(default, length) => {
                let (typed_default, default_ty) = self.infer_expr(default)?;
                let (typed_length, length_ty) = self.infer_expr(length)?;
                if length_ty != Type::Numeric(NumericType::Int) {
                    return Err(TypeError("length type must be of int"));
                }
                let typed_expr = TypedExpr::ArrayDefaultLength(Box::new(typed_default), Box::new(typed_length));
                let ty = Type::List(Box::new(default_ty));
                (typed_expr, ty)
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
                if arg_tys.len() != sig.params.len() {
                    return Err(TypeError("incorrect number of function arguments"));
                }
                let mut map = HashMap::new();
                for (arg, param) in arg_tys.iter().zip(&sig.params) {
                    instantiate(&mut map, arg, param).map_err(|_| TypeError("could not instantiate function"))?;
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
                    println!("undefined variable: {}", var.ident().0.as_str());
                    return Err(TypeError("undefined variable"));
                };
                (TypedExpr::Var(typed_var), ty)
            }
            Expr::Infix { left, op, right } => {
                let (left_expr, left_ty) = self.infer_expr(left)?;
                let (right_expr, right_ty) = self.infer_expr(right)?;

                let (op, ty) = match (*op, left_ty, right_ty) {
                    (InfixOp::Arith(op), Type::Numeric(left_ty), Type::Numeric(right_ty)) if left_ty == right_ty => {
                        (TypedInfixOp::Arith(op, left_ty), Type::Numeric(left_ty))
                    }
                    (InfixOp::Comp(op), Type::Numeric(left_ty), Type::Numeric(right_ty)) if left_ty == right_ty => {
                        (TypedInfixOp::Comp(op, left_ty), Type::Bool)
                    }
                    (InfixOp::Logical(op), Type::Bool, Type::Bool) => (TypedInfixOp::Logical(op), Type::Bool),
                    (InfixOp::Equality(op), left_ty, right_ty) if left_ty == right_ty => {
                        (TypedInfixOp::Equality(op), Type::Bool)
                    }
                    (InfixOp::Bitwise(op), Type::Numeric(NumericType::Int), Type::Numeric(NumericType::Int)) => {
                        (TypedInfixOp::Bitwise(op), Type::Numeric(NumericType::Int))
                    }
                    _ => return Err(TypeError("infix operator type error")),
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
                let Type::List(element_ty) = array_ty else {
                    return Err(TypeError("cannot index non array type"));
                };
                let (typed_index, index_ty) = self.infer_expr(index)?;
                if index_ty != Type::Numeric(NumericType::Int) {
                    return Err(TypeError("index type must be an Int"));
                }
                let typed_expr = TypedExpr::Index(Box::new(typed_array), Box::new(typed_index));
                (typed_expr, *element_ty)
            }
            Expr::Prefix(op, expr) => {
                let (typed_expr, expr_ty) = self.infer_expr(expr)?;
                let (typed_op, ty) = match (op, expr_ty) {
                    (PrefixOp::Negative, Type::Numeric(ty)) => (TypedPrefixOp::Negative(ty), Type::Numeric(ty)),
                    (PrefixOp::Positive, Type::Numeric(ty)) => (TypedPrefixOp::Positive(ty), Type::Numeric(ty)),
                    (PrefixOp::LogicalNot, Type::Bool) => (TypedPrefixOp::LogicalNot, Type::Bool),
                    (PrefixOp::BitwiseNot, Type::Numeric(NumericType::Int)) => {
                        (TypedPrefixOp::BitwiseNot, Type::Numeric(NumericType::Int))
                    }
                    _ => return Err(TypeError("invalid prefix operatortypes!")),
                };
                (TypedExpr::Prefix(typed_op, Box::new(typed_expr)), ty)
            }
            Expr::FunType(_, _) => panic!(),
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
                if let Some(ty) = ty
                    && expr_ty != ty.try_into().map_err(|_| TypeError("could not convert type"))?
                {
                    return Err(TypeError("wrong type annotation!"));
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
                let op = match *op {
                    Some(AssignOp::Arith(op)) => {
                        let ty = dst_ty
                            .as_numeric_type()
                            .ok_or(TypeError("assign arith type mismatch"))?;
                        Some(TypedAssignOp::Arith(op, ty))
                    }
                    Some(AssignOp::Bitwise(op)) => {
                        if dst_ty != Type::Numeric(NumericType::Int) {
                            return Err(TypeError("assign bitwise type mismatch"))?;
                        }
                        Some(TypedAssignOp::Bitwise(op))
                    }
                    None => None,
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
            Stmt::Break => (TypedStmt::Break, true),
            Stmt::Continue => (TypedStmt::Continue, true),
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
