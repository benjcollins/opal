use crate::{intern::InternedStr, lexer::Span, parser::Separated};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident {
    pub str: InternedStr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Lit {
    Int { value: i64, span: Span },
    Float { value: f64, span: Span },
    Bool { value: bool, span: Span },
    Str { value: String, span: Span },
    Unit { open_paren: Span, close_paren: Span },
}

#[derive(Debug, Clone)]
pub enum Expr {
    Lit(Lit),
    Call {
        fun: Box<Expr>,
        open_paren: Span,
        args: Separated<Expr>,
        close_paren: Span,
    },
    Paren {
        open_paren: Span,
        expr: Box<Expr>,
        close_paren: Span,
    },
    Var(Ident),
    ListElements {
        open_bracket: Span,
        elements: Separated<Expr>,
        close_bracket: Span,
    },
    ListDefaultLength {
        open_bracket: Span,
        default: Box<Expr>,
        semicolon: Span,
        length: Box<Expr>,
        close_bracket: Span,
    },
    Index {
        expr: Box<Expr>,
        open_bracket: Span,
        index: Box<Expr>,
        close_bracket: Span,
    },
    Prefix {
        op: PrefixOp,
        op_span: Span,
        expr: Box<Expr>,
    },
    FunType {
        fun: Span,
        open_paren: Span,
        params: Separated<Expr>,
        close_paren: Span,
        returns: Option<(Span, Box<Expr>)>,
    },
    Infix {
        left: Box<Expr>,
        op: InfixOp,
        op_span: Span,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfixOp {
    Arith(ArithOp),
    Comp(CompOp),
    Equality(EqualityOp),
    Logical(LogicalOp),
    Bitwise(BitwiseOp),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefixOp {
    Negative,
    Positive,
    LogicalNot,
    BitwiseNot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitwiseOp {
    And,
    Or,
    XOr,
    ShiftLeft,
    ShiftRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EqualityOp {
    Equal,
    NotEqual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompOp {
    Greater,
    Less,
    LessEqual,
    GreaterEqual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalOp {
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOp {
    Bitwise(BitwiseOp),
    Arith(ArithOp),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Var { name: Ident, ty: Option<Expr>, expr: Expr },
    Assign { dst: Expr, op: Option<AssignOp>, src: Expr },
    Break,
    Continue,
    Return(Option<Expr>),
    Expr(Expr),
    If(If),
    While { cond: Expr, block: Block },
}

#[derive(Debug, Clone)]
pub struct If {
    pub cond: Expr,
    pub if_block: Block,
    pub else_: Else,
}

#[derive(Debug, Clone)]
pub enum Else {
    If(Box<If>),
    Block(Block),
    Nothing,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug)]
pub struct Generics {
    pub open_bracket: Span,
    pub args: Separated<Ident>,
    pub close_bracket: Span,
}

#[derive(Debug)]
pub struct Fun {
    pub name: Ident,
    pub generics: Option<Generics>,
    pub params: Separated<(Ident, Expr)>,
    pub returns: Option<Expr>,
    pub block: Block,
}

#[derive(Debug)]
pub enum ModuleItem {
    Fun(Fun),
}

#[derive(Debug)]
pub struct Module {
    pub name: Ident,
    pub items: Vec<ModuleItem>,
}
