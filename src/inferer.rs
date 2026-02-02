use crate::{
    ast::{self, Block, Expr, FunDef, Ident, Lit, Stmt},
    scope::Scope,
    typed_ast::{TypedBlock, TypedExpr, TypedFunDef, TypedStmt, TypedVar, VarId},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Numeric(NumericType),
    Bool,
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
            Type::Bool => None,
        }
    }
}

pub struct Inferer {
    next_var_id: u32,
    scope: Scope<Ident, (VarId, Type)>,
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
        _ => return Err(()),
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
            Expr::Paren(node) => self.infer_expr(node)?,
            Expr::Var(var) => {
                let &(id, ty) = self.scope.get(var.ident()).unwrap();
                let typed_expr = TypedExpr::Var(TypedVar {
                    ident: var.ident().clone(),
                    id,
                });
                (typed_expr, ty)
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

    fn insert_var(&mut self, ident: Ident, ty: Type) -> TypedVar {
        let id = VarId(self.next_var_id);
        self.next_var_id += 1;
        self.scope.insert(ident.clone(), (id, ty));
        TypedVar { ident, id }
    }

    fn infer_stmt(&mut self, stmt: &Stmt) -> Result<TypedStmt, ()> {
        Ok(match stmt {
            Stmt::VarDef { var, expr } => {
                let (typed_expr, expr_ty) = self.infer_expr(expr)?;
                let typed_var = self.insert_var(var.ident().clone(), expr_ty);
                TypedStmt::VarDecl {
                    var: typed_var,
                    expr: typed_expr,
                }
            }
            Stmt::Assign { var, expr } => {
                let &(id, var_ty) = self.scope.get(var.ident()).unwrap();
                let (typed_expr, expr_ty) = self.infer_expr(expr)?;
                if expr_ty != var_ty {
                    return Err(());
                }
                TypedStmt::Assign {
                    var: TypedVar {
                        ident: var.ident().clone(),
                        id,
                    },
                    expr: typed_expr,
                }
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

    pub fn infer_fun_def(&mut self, fun_def: &FunDef) -> Result<TypedFunDef, ()> {
        self.scope.enter_block();
        let mut params = vec![];
        for (name, ty) in &fun_def.params {
            let ty = resolve_type(ty)?;
            let typed_var = self.insert_var(name.ident().clone(), ty);
            params.push((typed_var, ty));
        }
        let block = self.infer_block(&fun_def.block)?;
        self.scope.exit_block();
        Ok(TypedFunDef {
            name: fun_def.name.clone(),
            params,
            returns: fun_def
                .returns
                .as_ref()
                .map(|returns| resolve_type(&returns))
                .transpose()?,
            block,
        })
    }
}
