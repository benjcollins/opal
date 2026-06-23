use crate::{ast::InfixOp, ty::NumericMetaType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalId(pub u32);

#[derive(Debug, Clone)]
pub enum Expr {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Unit,
    Local(LocalId),
    Global(String),
    Call(Box<Expr>, Vec<Expr>),
    Infix {
        left: Box<Expr>,
        op: InfixOp,
        ty: NumericMetaType,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone)]
pub enum Stmt {
    VarDecl { local: LocalId, value: Expr },
    Expr(Expr),
    Return(Expr),
}

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}
