use crate::{interner::InternedStr, lexer::Span};

#[derive(Debug, Clone)]
pub struct AstNode<T> {
    pub id: u32,
    pub span: Span,
    pub node: Box<T>,
}

#[derive(Debug, Clone)]
pub struct Ident {
    pub str: InternedStr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Paren(AstNode<Expr>),
    Infix {
        left: AstNode<Expr>,
        op: InfixOp,
        right: AstNode<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfixOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Mod,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    VarDecl {
        name: Ident,
        expr: Option<AstNode<Expr>>,
    },
    Assign {
        name: Ident,
        expr: AstNode<Expr>,
    }
}

#[derive(Debug)]
pub struct Block {
    pub stmts: Vec<AstNode<Stmt>>,
}

#[derive(Debug)]
pub enum Type {
    Name(Ident),
}

#[derive(Debug)]
pub enum Decl {
    Func {
        name: Ident,
        params: Vec<(Ident, AstNode<Type>)>,
        returns: Option<AstNode<Type>>,
        block: AstNode<Block>,
    }
}