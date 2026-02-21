use std::rc::Rc;

use crate::{
    ast::{ArithOp, CompOp, EqualityOp, Ident, Lit, LogicalOp},
    infer::{NumericType, Type},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VarId(pub u32);

#[derive(Debug, Clone)]
pub struct LocalTypedVar {
    pub mutable: bool,
    pub ident: Ident,
    pub ty: Type,
    pub id: VarId,
}

#[derive(Debug, Clone)]
pub enum TypedVar {
    Local(Rc<LocalTypedVar>),
    Env(Ident),
}

#[derive(Debug, Clone)]
pub enum TypedInfixOp {
    Arith(ArithOp, NumericType),
    Comp(CompOp, NumericType),
    Equality(EqualityOp),
    Logical(LogicalOp),
}

#[derive(Debug, Clone)]
pub enum TypedExpr {
    Lit(Lit),
    Call {
        fun: Box<TypedExpr>,
        args: Vec<TypedExpr>,
    },
    Array(Vec<TypedExpr>),
    Index(Box<TypedExpr>, Box<TypedExpr>),
    Var(TypedVar),
    Infix {
        left: Box<TypedExpr>,
        right: Box<TypedExpr>,
        op: TypedInfixOp,
    },
}

#[derive(Debug, Clone)]
pub enum TypedStmt {
    Let {
        var: Rc<LocalTypedVar>,
        expr: TypedExpr,
    },
    Assign {
        dst: TypedExpr,
        op: Option<(ArithOp, NumericType)>,
        src: TypedExpr,
    },
    Expr(TypedExpr),
    Return(TypedExpr),
    If(TypedIf),
    While {
        cond: TypedExpr,
        block: TypedBlock,
    },
}

#[derive(Debug, Clone)]
pub struct TypedIf {
    pub cond: TypedExpr,
    pub if_block: TypedBlock,
    pub else_: TypedElse,
}

#[derive(Debug, Clone)]
pub enum TypedElse {
    If(Box<TypedIf>),
    Block(TypedBlock),
    Nothing,
}

#[derive(Debug, Clone)]
pub struct TypedBlock {
    pub stmts: Vec<TypedStmt>,
    pub diverges: bool,
}

#[derive(Debug, Clone)]
pub struct TypedFun {
    pub name: Ident,
    pub params: Vec<Rc<LocalTypedVar>>,
    pub returns: Type,
    pub block: TypedBlock,
}
