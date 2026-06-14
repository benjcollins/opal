use crate::lexer::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Bool {
        value: bool,
        span: Span,
    },
    Int {
        value: i64,
        span: Span,
    },
    Float {
        value: f64,
        span: Span,
    },
    String {
        value: String,
        span: Span,
    },
    Unit {
        open_paren: Span,
        close_paren: Span,
    },
    Var {
        name: String,
        span: Span,
    },
    Call {
        fun: Box<Expr>,
        open_paren: Span,
        args: Vec<(Expr, Option<Span>)>,
        close_paren: Span,
    },
    Parens {
        open_paren: Span,
        expr: Box<Expr>,
        close_paren: Span,
    },
    Infix {
        left: Box<Expr>,
        op: InfixOp,
        op_span: Span,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InfixOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    VarDecl {
        var: Span,
        name: String,
        name_span: Span,
        equals: Span,
        value: Expr,
        semicolon: Span,
    },
    Expr {
        expr: Expr,
        semicolon: Span,
    },
    Return {
        return_: Span,
        expr: Option<Expr>,
        semicolon: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub open_brace: Span,
    pub stmts: Vec<Stmt>,
    pub close_brace: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Type {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub name_span: Span,
    pub colon: Span,
    pub ty: Type,
    pub comma: Option<Span>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Returns {
    pub arrow: Span,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Decl {
    Fun {
        fun: Span,
        name: String,
        name_span: Span,
        open_paren: Span,
        params: Vec<Param>,
        close_paren: Span,
        returns: Option<Returns>,
        body: Block,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct File {
    pub module_keyword: Span,
    pub module_name: String,
    pub module_name_span: Span,
    pub semicolon: Span,
    pub decls: Vec<Decl>,
}
