use std::rc::Rc;

use crate::{
    ast::{ArithOp, BitwiseOp, CompOp, EqualityOp, Ident, Lit, LogicalOp},
    ty::{NumericType, Type},
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

#[derive(Debug, Clone, Copy)]
pub enum TypedInfixOp {
    Arith(ArithOp, NumericType),
    Comp(CompOp, NumericType),
    Equality(EqualityOp),
    Logical(LogicalOp),
    Bitwise(BitwiseOp),
}

#[derive(Debug, Clone, Copy)]
pub enum TypedAssignOp {
    Arith(ArithOp, NumericType),
    Bitwise(BitwiseOp),
}

#[derive(Debug, Clone, Copy)]
pub enum TypedPrefixOp {
    Negative(NumericType),
    Positive(NumericType),
    BitwiseNot,
    LogicalNot,
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
    Prefix(TypedPrefixOp, Box<TypedExpr>),
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
        op: Option<TypedAssignOp>,
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
