use std::collections::HashMap;

use crate::{
    ast,
    ir::{self, Expr},
    lexer::Span,
    scoped_map::ScopedMap,
    ty::{CanonType, MetaType, Type, TypeContext},
};

pub struct Analyzer<'a, 'b> {
    pub var_map: ScopedMap<String, ir::LocalId>,
    pub var_types: HashMap<ir::LocalId, MetaType>,
    pub globals: &'a HashMap<String, CanonType>,
    pub return_ty: CanonType,
    pub type_context: &'b mut TypeContext,
    pub next_local_id: u32,
}

#[derive(Debug, Clone)]
pub struct TypeError {
    message: &'static str,
    span: Span,
}

pub enum Fallthrough {
    True,
    False,
    AllOf(Vec<Fallthrough>),
    AnyOf(Vec<Fallthrough>),
    NotVoid(MetaType),
}

impl<'a, 'b> Analyzer<'a, 'b> {
    pub fn new(
        globals: &'a HashMap<String, CanonType>,
        return_ty: CanonType,
        type_context: &'b mut TypeContext,
    ) -> Analyzer<'a, 'b> {
        Analyzer {
            var_map: ScopedMap::new(),
            var_types: HashMap::new(),
            globals,
            return_ty,
            type_context,
            next_local_id: 0,
        }
    }

    pub fn analyze_block(
        &mut self,
        block: &ast::Block,
    ) -> Result<(ir::Block, Fallthrough), TypeError> {
        let mut stmts = Vec::new();
        let mut fallthroughs = Vec::new();
        for stmt in &block.stmts {
            let (stmt, fallthrough) = self.analyze_stmt(stmt)?;
            stmts.push(stmt);
            fallthroughs.push(fallthrough);
        }
        Ok((ir::Block { stmts }, Fallthrough::AllOf(fallthroughs)))
    }

    pub fn analyze_stmt(&mut self, stmt: &ast::Stmt) -> Result<(ir::Stmt, Fallthrough), TypeError> {
        Ok(match stmt {
            ast::Stmt::VarDecl { name, value, .. } => {
                let (value_ty, value) = self.analyze_expr(value)?;
                let local = self.decl_local_var(name.clone(), value_ty);
                let var_decl = ir::Stmt::VarDecl { local, value };
                (var_decl, Fallthrough::True)
            }
            ast::Stmt::Expr { expr, .. } => {
                let (expr_ty, expr) = self.analyze_expr(expr)?;
                (ir::Stmt::Expr(expr), Fallthrough::NotVoid(expr_ty))
            }
            ast::Stmt::Return { expr, return_, .. } => {
                let expr = if let Some(expr) = expr {
                    let (expr_ty, expr) = self.analyze_expr(expr)?;
                    self.type_context
                        .unify_with_canon(&expr_ty, &self.return_ty)
                        .map_err(|_| TypeError {
                            message: "incorrect return type",
                            span: *return_,
                        })?;
                    expr
                } else {
                    Expr::Unit
                };
                (ir::Stmt::Return(expr), Fallthrough::False)
            }
        })
    }

    pub fn analyze_expr(&mut self, expr: &ast::Expr) -> Result<(MetaType, ir::Expr), TypeError> {
        Ok(match expr {
            ast::Expr::Bool { value, .. } => (Type::Bool.into(), ir::Expr::Bool(*value)),
            ast::Expr::Int { value, .. } => (Type::Int.into(), ir::Expr::Int(*value)),
            ast::Expr::Float { value, .. } => (Type::Float.into(), ir::Expr::Float(*value)),
            ast::Expr::String { value, .. } => {
                (Type::String.into(), ir::Expr::String(value.clone()))
            }
            ast::Expr::Unit { .. } => (Type::Unit.into(), ir::Expr::Unit),
            ast::Expr::Var { name, span } => {
                if let Some(var_id) = self.var_map.get(name) {
                    let ty = self.var_types.get(var_id).unwrap().clone();
                    (ty, ir::Expr::Local(*var_id))
                } else if let Some(ty) = self.globals.get(name) {
                    (ty.into(), ir::Expr::Global(name.clone()))
                } else {
                    return Err(TypeError {
                        span: *span,
                        message: "undefined variable",
                    });
                }
            }
            ast::Expr::Call {
                fun,
                args,
                open_paren,
                close_paren,
            } => {
                let (fun_ty, fun_ir) = self.analyze_expr(fun)?;
                let mut arg_tys = Vec::new();
                let mut arg_irs = Vec::new();
                for (arg_expr, _) in args {
                    let (arg_ty, arg_ir) = self.analyze_expr(arg_expr)?;
                    arg_tys.push(arg_ty);
                    arg_irs.push(arg_ir);
                }
                let return_ty = self.type_context.fresh_meta();
                let fun_expected_ty = Type::Fun(arg_tys, Box::new(return_ty.into())).into();
                let call_ir = ir::Expr::Call(Box::new(fun_ir), arg_irs);
                self.type_context
                    .unify(&fun_ty, &fun_expected_ty)
                    .map_err(|_| TypeError {
                        message: "incorrect function signature",
                        span: open_paren.to(close_paren),
                    })?;
                (return_ty.into(), call_ir)
            }
            ast::Expr::Parens { expr, .. } => self.analyze_expr(expr)?,
            ast::Expr::Infix {
                left,
                op,
                right,
                op_span,
                ..
            } => {
                let (left_ty, left_ir) = self.analyze_expr(left)?;
                let (right_ty, right_ir) = self.analyze_expr(right)?;
                let ty = self.type_context.fresh_numeric_meta();
                self.type_context
                    .unify(&ty.into(), &left_ty)
                    .map_err(|_| TypeError {
                        message: "incorrect numeric type",
                        span: *op_span,
                    })?;
                self.type_context
                    .unify(&ty.into(), &right_ty)
                    .map_err(|_| TypeError {
                        message: "incorrect numeric type",
                        span: *op_span,
                    })?;
                let infix_ir = ir::Expr::Infix {
                    left: Box::new(left_ir),
                    op: *op,
                    ty: ty.into(),
                    right: Box::new(right_ir),
                };
                (ty.into(), infix_ir)
            }
        })
    }

    pub fn decl_local_var(&mut self, name: String, ty: MetaType) -> ir::LocalId {
        let local = ir::LocalId(self.next_local_id);
        self.next_local_id += 1;
        self.var_map.insert(name.clone(), local);
        self.var_types.insert(local, ty);
        local
    }

    pub fn is_fallthrough(&self, fallthrough: &Fallthrough) -> bool {
        match fallthrough {
            Fallthrough::True => true,
            Fallthrough::False => false,
            Fallthrough::AllOf(fallthroughs) => fallthroughs.iter().all(|f| self.is_fallthrough(f)),
            Fallthrough::AnyOf(fallthroughs) => fallthroughs.iter().any(|f| self.is_fallthrough(f)),
            Fallthrough::NotVoid(ty) => match &*self.type_context.get_type(ty).unwrap() {
                Type::Void => false,
                _ => false,
            },
        }
    }
}
