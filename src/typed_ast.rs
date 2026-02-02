use std::rc::Rc;

use crate::{
    ast::{Ident, InfixOp, Lit},
    infer::{NumericType, Type},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VarId(pub u32);

#[derive(Debug, Clone)]
pub struct TypedVar {
    pub mutable: bool,
    pub ident: Ident,
    pub ty: Type,
    pub id: VarId,
}

#[derive(Debug, Clone)]
pub enum TypedExpr {
    Lit(Lit),
    Var(Rc<TypedVar>),
    Infix {
        left: Box<TypedExpr>,
        right: Box<TypedExpr>,
        op: InfixOp,
        ty: NumericType,
    },
}

#[derive(Debug, Clone)]
pub enum TypedStmt {
    Let { var: Rc<TypedVar>, expr: TypedExpr },
    Assign { var: Rc<TypedVar>, expr: TypedExpr },
    Expr(TypedExpr),
}

#[derive(Debug)]
pub struct TypedBlock {
    pub stmts: Vec<TypedStmt>,
}

#[derive(Debug)]
pub struct TypedFun {
    pub name: Ident,
    pub params: Vec<Rc<TypedVar>>,
    pub returns: Option<Type>,
    pub block: TypedBlock,
}
