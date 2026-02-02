use std::rc::Rc;

use crate::{
    ast::{self, Block, Expr, Fun, Ident, Lit, Stmt, VarDef},
    scope::Scope,
    typed_ast::{TypedBlock, TypedExpr, TypedFun, TypedStmt, TypedVar, VarId},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Numeric(NumericType),
    Bool,
    Unit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericType {
    Int,
    Float,
}

impl Type {
    fn as_numeric_type(&self) -> Option<NumericType> {
        match self {
            Type::Numeric(numeric_type) => Some(*numeric_type),
            _ => None,
        }
    }
}

pub struct Inferer {
    next_var_id: u32,
    scope: Scope<Ident, Rc<TypedVar>>,
}

fn infer_lit(lit: &Lit) -> Type {
    match lit {
        Lit::Int(_) => Type::Numeric(NumericType::Int),
        Lit::Float(_) => Type::Numeric(NumericType::Float),
        Lit::Bool(_) => Type::Bool,
    }
}

fn resolve_type(ty: &ast::Type) -> Result<Type, ()> {
    Ok(match ty.0.0.as_str() {
        "Int" => Type::Numeric(NumericType::Int),
        "Float" => Type::Numeric(NumericType::Float),
        "Bool" => Type::Bool,
        "Unit" => Type::Unit,
        _ => return Err(()),
    })
}

pub fn infer_fun(fun: &Fun) -> Result<TypedFun, ()> {
    let mut inferer = Inferer {
        next_var_id: 0,
        scope: Scope::new(),
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
        returns: fun.returns.as_ref().map(|returns| resolve_type(&returns)).transpose()?,
        block,
    })
}

impl Inferer {
    pub fn new() -> Inferer {
        Inferer {
            next_var_id: 0,
            scope: Scope::new(),
        }
    }

    fn infer_expr(&mut self, expr: &Expr) -> Result<(TypedExpr, Type), ()> {
        let (typed_expr, ty) = match expr {
            Expr::Lit(lit) => {
                let typed_expr = TypedExpr::Lit(*lit);
                let ty = infer_lit(lit);
                (typed_expr, ty)
            }
            Expr::Call(name, args) => {
                todo!()
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

                let ty = Type::Numeric(left_ty);

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
            Stmt::Expr(expr) => {
                let (expr, ty) = self.infer_expr(expr)?;
                if ty != Type::Unit {
                    return Err(());
                }
                TypedStmt::Expr(expr)
            }
        })
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
